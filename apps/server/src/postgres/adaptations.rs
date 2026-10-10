//! Private, immutable adaptation inputs and fenced provider dispatch on the existing store.
use std::{sync::Arc, time::Duration};

use cantos_api::{
    AcceptAdaptationRequest, AdaptationCost, AdaptationProblem, AdaptationProposal,
    AdaptationProviderMetadata, AdaptationRunResponse, AdaptationStatus, AdaptationUsage,
    CancelAdaptationRequest, FieldIssue, ImportOutcome, ImportResponse, RevisionResponse,
    SaveRevisionRequest, StartAdaptationRequest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_postgres::Row;
use uuid::Uuid;

use super::{
    authenticate, authorize, checked_number, decode_revision, export_digest, normalized_uuid,
    save_in_transaction, Store, StoreError, REVISION_COLUMNS,
};
use crate::{
    adaptation::provider::{AdaptationProvider, ProviderFailure, ProviderOutput},
    adaptation::{
        admit_edited_proposal, admit_output, prepare_request, validate_request_budget,
        TrustedBinding,
    },
    revisions::Action,
    script_ir::read_canonical_script,
};

const RUN_COLUMNS: &str = "id,owner_id,operation_id,source_id,script_id,frozen_input::text AS frozen_input,input_digest,status,problem::text AS problem,to_char(recorded_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS recorded_at,to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS updated_at";

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FrozenAdaptation {
    request: StartAdaptationRequest,
    source: ImportResponse,
    base: Option<RevisionResponse>,
    provider: AdaptationProviderMetadata,
    generation_record_id: String,
    rights_record_id: String,
}

impl FrozenAdaptation {
    fn binding(&self, id: &str) -> TrustedBinding {
        TrustedBinding {
            source_id: self.source.id.clone(),
            source_sha256: self.source.sha256.clone(),
            generation_record_id: self.generation_record_id.clone(),
            rights_record_id: self.rights_record_id.clone(),
            id_namespace: id.replace('-', ""),
            base_script_json: self.base.as_ref().map(|base| base.script_json.clone()),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    problem: Option<AdaptationProblem>,
    usage: Option<AdaptationUsage>,
    cost: Option<AdaptationCost>,
    output_sha256: Option<String>,
    output_byte_len: Option<u64>,
}

fn problem(code: &str) -> AdaptationProblem {
    AdaptationProblem {
        code: code.into(),
        issues: vec![],
    }
}

fn typed_digest<T: Serialize>(label: &[u8], value: &T) -> Result<String, StoreError> {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update(serde_json::to_vec(value).map_err(|_| StoreError::CorruptRevision)?);
    Ok(format!("{:x}", digest.finalize()))
}

fn status_name(status: AdaptationStatus) -> &'static str {
    match status {
        AdaptationStatus::Queued => "queued",
        AdaptationStatus::Running => "running",
        AdaptationStatus::Succeeded => "succeeded",
        AdaptationStatus::InvalidOutput => "invalid_output",
        AdaptationStatus::Failed => "failed",
        AdaptationStatus::Ambiguous => "ambiguous",
        AdaptationStatus::Cancelled => "cancelled",
        AdaptationStatus::Accepted => "accepted",
    }
}

fn decode_status(status: &str) -> Result<AdaptationStatus, StoreError> {
    match status {
        "queued" => Ok(AdaptationStatus::Queued),
        "running" => Ok(AdaptationStatus::Running),
        "succeeded" => Ok(AdaptationStatus::Succeeded),
        "invalid_output" => Ok(AdaptationStatus::InvalidOutput),
        "failed" => Ok(AdaptationStatus::Failed),
        "ambiguous" => Ok(AdaptationStatus::Ambiguous),
        "cancelled" => Ok(AdaptationStatus::Cancelled),
        "accepted" => Ok(AdaptationStatus::Accepted),
        _ => Err(StoreError::CorruptRevision),
    }
}

fn decode_frozen(row: &Row) -> Result<FrozenAdaptation, StoreError> {
    let frozen: FrozenAdaptation = serde_json::from_str(&row.get::<_, String>("frozen_input"))
        .map_err(|_| StoreError::CorruptRevision)?;
    if typed_digest(b"cantos/adaptation-input/a1\n", &frozen)?
        != row.get::<_, String>("input_digest")
        || frozen.request.operation_id != row.get::<_, String>("operation_id")
        || frozen.request.source_id != row.get::<_, String>("source_id")
        || frozen.request.script_id != row.get::<_, String>("script_id")
        || frozen.source.imported_by != row.get::<_, String>("owner_id")
        || frozen.source.id != frozen.request.source_id
        || !frozen.request.rights_authorization
        || frozen.request.expected_provider != frozen.provider
        || frozen.generation_record_id
            != format!("gen_{}", row.get::<_, String>("id").replace('-', ""))
        || frozen.rights_record_id
            != format!("rights_{}", row.get::<_, String>("id").replace('-', ""))
    {
        return Err(StoreError::CorruptRevision);
    }
    normalized_uuid(&frozen.request.operation_id).map_err(|_| StoreError::CorruptRevision)?;
    normalized_uuid(&frozen.request.script_id).map_err(|_| StoreError::CorruptRevision)?;
    match &frozen.base {
        Some(base) => {
            let script = read_canonical_script(base.script_json.as_bytes())
                .map_err(|_| StoreError::CorruptRevision)?;
            if base.script_id != frozen.request.script_id
                || base.revision != frozen.request.expected_revision
                || base.content_digest != script.content_digest().to_string()
                || base.export_digest != export_digest(base.script_json.as_bytes())
            {
                return Err(StoreError::CorruptRevision);
            }
        }
        None if frozen.request.expected_revision != 0 => return Err(StoreError::CorruptRevision),
        None => (),
    }
    Ok(frozen)
}

async fn source_in_transaction(
    tx: &deadpool_postgres::Transaction<'_>,
    actor: &str,
    id: &str,
) -> Result<ImportResponse, StoreError> {
    let query = format!(
        "SELECT {} FROM source_records WHERE owner_id=$1 AND imported_by=$1 AND id=$2",
        super::imports::IMPORT_COLUMNS
    );
    let row = tx
        .query_opt(&query, &[&actor, &id])
        .await?
        .ok_or(StoreError::NotFound)?;
    super::imports::decode_import(&row).map(|(source, _)| source)
}

async fn expire_dispatch(
    tx: &deadpool_postgres::Transaction<'_>,
    id: &str,
) -> Result<(), StoreError> {
    let reason = serde_json::to_string(&problem("dispatch_outcome_unknown"))
        .map_err(|_| StoreError::Unavailable)?;
    let changed = tx.execute("UPDATE adaptation_runs SET status='ambiguous',problem=$2::text::jsonb,dispatch_deadline=NULL,updated_at=clock_timestamp() WHERE id=$1 AND status='running' AND dispatch_deadline<=clock_timestamp()", &[&id,&reason]).await?;
    if changed == 1 {
        let observation = Observation {
            problem: Some(problem("dispatch_outcome_unknown")),
            usage: None,
            cost: None,
            output_sha256: None,
            output_byte_len: None,
        };
        let json = serde_json::to_string(&observation).map_err(|_| StoreError::Unavailable)?;
        tx.execute("INSERT INTO adaptation_observations(run_id,kind,observation) VALUES($1,'interrupted',$2::text::jsonb)", &[&id,&json]).await?;
    }
    Ok(())
}

async fn run_in_transaction(
    tx: &deadpool_postgres::Transaction<'_>,
    actor: &str,
    id: &str,
) -> Result<(AdaptationRunResponse, FrozenAdaptation), StoreError> {
    let query =
        format!("SELECT {RUN_COLUMNS} FROM adaptation_runs WHERE id=$1 AND owner_id=$2 FOR UPDATE");
    // Lock before expiry so cancellation/completion/read recovery have one transition order.
    let initial = tx
        .query_opt(&query, &[&id, &actor])
        .await?
        .ok_or(StoreError::NotFound)?;
    let frozen = decode_frozen(&initial)?;
    if source_in_transaction(tx, actor, &frozen.source.id).await? != frozen.source {
        return Err(StoreError::CorruptRevision);
    }
    expire_dispatch(tx, id).await?;
    let row = tx.query_one(&query, &[&id, &actor]).await?;
    let status = decode_status(&row.get::<_, String>("status"))?;
    let proposal = match tx
        .query_opt(
            "SELECT proposal::text,digest FROM adaptation_proposals WHERE run_id=$1",
            &[&id],
        )
        .await?
    {
        Some(row) => {
            let proposal: AdaptationProposal = serde_json::from_str(&row.get::<_, String>(0))
                .map_err(|_| StoreError::CorruptRevision)?;
            if typed_digest(b"cantos/adaptation-proposal/p1\n", &proposal)?
                != row.get::<_, String>(1)
            {
                return Err(StoreError::CorruptRevision);
            }
            read_canonical_script(proposal.script_json.as_bytes())
                .map_err(|_| StoreError::CorruptRevision)?;
            admit_edited_proposal(&proposal.script_json, &frozen.source, &frozen.binding(id))
                .map_err(|_| StoreError::CorruptRevision)?;
            Some(proposal)
        }
        None => None,
    };
    if matches!(
        status,
        AdaptationStatus::Succeeded | AdaptationStatus::Accepted
    ) && proposal.is_none()
    {
        return Err(StoreError::CorruptRevision);
    }
    let observation = tx.query_opt("SELECT observation::text FROM adaptation_observations WHERE run_id=$1 AND kind='provider_result'", &[&id]).await?.map(|row| serde_json::from_str::<Observation>(&row.get::<_,String>(0))).transpose().map_err(|_| StoreError::CorruptRevision)?;
    let accepted = tx
        .query_opt(
            "SELECT script_id,revision FROM adaptation_acceptances WHERE run_id=$1",
            &[&id],
        )
        .await?;
    let accepted_revision = if let Some(accepted) = accepted {
        let query = format!(
            "SELECT {REVISION_COLUMNS} FROM script_revisions WHERE script_id=$1 AND revision=$2"
        );
        Some(decode_revision(
            &tx.query_one(
                &query,
                &[&accepted.get::<_, String>(0), &accepted.get::<_, i64>(1)],
            )
            .await?,
        )?)
    } else {
        None
    };
    if (status == AdaptationStatus::Accepted) != accepted_revision.is_some() {
        return Err(StoreError::CorruptRevision);
    }
    let run_problem = row
        .get::<_, Option<String>>("problem")
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|_| StoreError::CorruptRevision)?;
    let response = AdaptationRunResponse {
        id: id.into(),
        created_by: actor.into(),
        recorded_at: row.get("recorded_at"),
        updated_at: row.get("updated_at"),
        request: frozen.request.clone(),
        source_sha256: frozen.source.sha256.clone(),
        extractor_version: match &frozen.source.outcome {
            ImportOutcome::Parsed { extraction } => extraction.extractor_version.clone(),
            ImportOutcome::Failed { .. } => return Err(StoreError::CorruptRevision),
        },
        input_content_digest: frozen.base.as_ref().map(|base| base.content_digest.clone()),
        input_export_digest: frozen.base.as_ref().map(|base| base.export_digest.clone()),
        generation_record_id: frozen.generation_record_id.clone(),
        rights_record_id: frozen.rights_record_id.clone(),
        provider: frozen.provider.clone(),
        status,
        proposal,
        problem: run_problem,
        usage: observation.as_ref().and_then(|value| value.usage.clone()),
        cost: observation.and_then(|value| value.cost),
        accepted_revision,
    };
    Ok((response, frozen))
}

impl Store {
    /// Configuration is explicit and local. Unconfigured hosts cannot dispatch generation.
    pub fn with_adaptation_provider(mut self, provider: Arc<dyn AdaptationProvider>) -> Self {
        self.adaptation_provider = Some(provider);
        self
    }

    pub fn adaptation_provider_metadata(&self) -> Option<AdaptationProviderMetadata> {
        self.adaptation_provider
            .as_ref()
            .map(|provider| provider.metadata())
    }

    pub async fn start_adaptation(
        &self,
        token: &str,
        request: StartAdaptationRequest,
    ) -> Result<AdaptationRunResponse, StoreError> {
        normalized_uuid(&request.operation_id)?;
        normalized_uuid(&request.script_id)?;
        checked_number(request.expected_revision)?;
        if !request.rights_authorization {
            return Err(StoreError::InvalidRequestFields(vec![FieldIssue {
                path: "/rights_authorization".into(),
                rule: "required_authorization".into(),
            }]));
        }
        let mut client = self.connection().await?;
        let tx = client.transaction().await?;
        let actor = authenticate(&tx, token).await?;
        tx.query_one(
            "SELECT pg_advisory_xact_lock(hashtextextended('adaptation:' || $1::text,0))",
            &[&actor],
        )
        .await?;
        authenticate(&tx, token).await?;
        if let Some(row) = tx
            .query_opt(
                "SELECT id FROM adaptation_runs WHERE owner_id=$1 AND operation_id=$2",
                &[&actor, &request.operation_id],
            )
            .await?
        {
            let id: String = row.get(0);
            let (response, frozen) = run_in_transaction(&tx, &actor, &id).await?;
            if frozen.request != request {
                return Err(StoreError::OperationReused);
            }
            authenticate(&tx, token).await?;
            tx.commit().await?;
            // A duplicate delivery observes its first run; never starts another attempt.
            self.schedule_queued(token, &response);
            return Ok(response);
        }
        // The authorization names the destination, model and effective configuration the
        // creator reviewed. A host configuration change requires a fresh explicit decision.
        let provider = self
            .adaptation_provider_metadata()
            .ok_or(StoreError::Unavailable)?;
        if request.expected_provider != provider {
            return Err(StoreError::InvalidRequestFields(vec![FieldIssue {
                path: "/expected_provider".into(),
                rule: "provider_configuration_changed".into(),
            }]));
        }
        let source = source_in_transaction(&tx, &actor, &request.source_id).await?;
        let mut issues = Vec::new();
        for (value, path) in [
            (&source.metadata.rights_holder, "/source/rights_holder"),
            (
                &source.metadata.permission_evidence,
                "/source/permission_evidence",
            ),
            (&source.metadata.usage_scope, "/source/usage_scope"),
        ] {
            if value.as_deref().is_none_or(|value| value.trim().is_empty()) {
                issues.push(FieldIssue {
                    path: path.into(),
                    rule: "required_claim".into(),
                });
            }
        }
        if !issues.is_empty() {
            return Err(StoreError::InvalidRequestFields(issues));
        }
        let base = if tx
            .query_opt("SELECT id FROM scripts WHERE id=$1", &[&request.script_id])
            .await?
            .is_some()
        {
            let (owner, head) =
                authorize(&tx, &actor, &request.script_id, Action::Review, false).await?;
            if owner != actor {
                return Err(StoreError::Forbidden);
            }
            if head != request.expected_revision {
                return Err(StoreError::StaleRevision(head));
            }
            let query=format!("SELECT {REVISION_COLUMNS} FROM script_revisions WHERE script_id=$1 AND revision=$2");
            Some(decode_revision(
                &tx.query_one(&query, &[&request.script_id, &checked_number(head)?])
                    .await?,
            )?)
        } else {
            if request.expected_revision != 0 {
                return Err(StoreError::NotFound);
            }
            None
        };
        let prepared =
            prepare_request(&source, base.as_ref().map(|base| base.script_json.as_str()))
                .map_err(|value| StoreError::InvalidRequestFields(value.issues))?;
        validate_request_budget(&prepared, &provider.config)
            .map_err(|value| StoreError::InvalidRequestFields(value.issues))?;
        if !(1..=180).contains(&provider.config.timeout_seconds) {
            return Err(StoreError::InvalidRequest);
        }
        let id = Uuid::new_v4();
        let frozen = FrozenAdaptation {
            request: request.clone(),
            source,
            base,
            provider,
            generation_record_id: format!("gen_{}", id.simple()),
            rights_record_id: format!("rights_{}", id.simple()),
        };
        for (kind,evidence,description) in [("generation",&frozen.generation_record_id,"Local adaptation attempt; execution is not editorial or publication approval"),("rights",&frozen.rights_record_id,"Creator source/adaptation-use assertion pinned to source checksum; legal adjudication and publication eligibility remain unknown")] {
            tx.execute("INSERT INTO script_evidence(owner_id,kind,id,description) VALUES($1,$2,$3,$4)", &[&actor,&kind,&evidence,&description]).await?;
        }
        let json = serde_json::to_string(&frozen).map_err(|_| StoreError::Unavailable)?;
        let digest = typed_digest(b"cantos/adaptation-input/a1\n", &frozen)?;
        let id = id.to_string();
        tx.execute("INSERT INTO adaptation_runs(id,owner_id,operation_id,source_id,script_id,frozen_input,input_digest,status) VALUES($1,$2,$3,$4,$5,$6::text::jsonb,$7,'queued')", &[&id,&actor,&request.operation_id,&request.source_id,&request.script_id,&json,&digest]).await?;
        let (response, _) = run_in_transaction(&tx, &actor, &id).await?;
        authenticate(&tx, token).await?;
        tx.commit().await?;
        self.schedule_queued(token, &response);
        Ok(response)
    }

    fn schedule_queued(&self, token: &str, response: &AdaptationRunResponse) {
        if response.status == AdaptationStatus::Queued && self.adaptation_provider.is_some() {
            // Bound task count as well as calls. Other queued runs recover on explicit refresh.
            if let Ok(permit) = self.adaptation_slots.clone().try_acquire_owned() {
                let store = self.clone();
                let token = token.to_owned();
                let id = response.id.clone();
                tokio::spawn(async move {
                    let _ = store.dispatch_adaptation(&token, &id, permit).await;
                });
            }
        }
    }

    pub async fn load_adaptation(
        &self,
        token: &str,
        id: &str,
    ) -> Result<AdaptationRunResponse, StoreError> {
        normalized_uuid(id)?;
        let mut client = self.connection().await?;
        let tx = client.transaction().await?;
        let actor = authenticate(&tx, token).await?;
        let (response, _) = run_in_transaction(&tx, &actor, id).await?;
        authenticate(&tx, token).await?;
        tx.commit().await?;
        // A queued run has no dispatch intent, so authenticated recovery can claim it safely.
        self.schedule_queued(token, &response);
        Ok(response)
    }

    async fn dispatch_adaptation(
        &self,
        token: &str,
        id: &str,
        _permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<(), StoreError> {
        let provider = self
            .adaptation_provider
            .as_ref()
            .ok_or(StoreError::Unavailable)?
            .clone();
        let mut client = self.connection().await?;
        let tx = client.transaction().await?;
        let actor = authenticate(&tx, token).await?;
        let (response, frozen) = run_in_transaction(&tx, &actor, id).await?;
        authenticate(&tx, token).await?;
        if response.status != AdaptationStatus::Queued {
            return Ok(());
        }
        if provider.metadata() != frozen.provider {
            let reason = serde_json::to_string(&problem("provider_configuration_changed"))
                .map_err(|_| StoreError::Unavailable)?;
            tx.execute("UPDATE adaptation_runs SET status='failed',problem=$2::text::jsonb,updated_at=clock_timestamp() WHERE id=$1 AND status='queued'", &[&id,&reason]).await?;
            tx.commit().await?;
            return Ok(());
        }
        // A queued run may have waited through another save or access change. Inspect only
        // current authorization/head facts; generation still uses the immutable pinned body.
        let target_exists = tx
            .query_opt(
                "SELECT id FROM scripts WHERE id=$1",
                &[&frozen.request.script_id],
            )
            .await?
            .is_some();
        let target_problem = if target_exists {
            match authorize(
                &tx,
                &actor,
                &frozen.request.script_id,
                Action::Review,
                false,
            )
            .await
            {
                Ok((owner, head)) if owner == actor && head == frozen.request.expected_revision => {
                    None
                }
                Ok(_) => Some(problem("stale_input_revision")),
                Err(StoreError::NotFound | StoreError::Forbidden) => {
                    Some(problem("target_unavailable"))
                }
                Err(error) => return Err(error),
            }
        } else if frozen.base.is_some() {
            Some(problem("target_unavailable"))
        } else {
            None
        };
        if let Some(reason) = target_problem {
            let reason = serde_json::to_string(&reason).map_err(|_| StoreError::Unavailable)?;
            tx.execute("UPDATE adaptation_runs SET status='failed',problem=$2::text::jsonb,updated_at=clock_timestamp() WHERE id=$1 AND status='queued'", &[&id, &reason]).await?;
            authenticate(&tx, token).await?;
            tx.commit().await?;
            return Ok(());
        }
        let request = prepare_request(
            &frozen.source,
            frozen.base.as_ref().map(|base| base.script_json.as_str()),
        )
        .map_err(|_| StoreError::CorruptRevision)?;
        validate_request_budget(&request, &frozen.provider.config)
            .map_err(|_| StoreError::CorruptRevision)?;
        authenticate(&tx, token).await?;
        let timeout_seconds = i64::from(frozen.provider.config.timeout_seconds);
        tx.execute("INSERT INTO adaptation_attempts(run_id) VALUES($1)", &[&id])
            .await?;
        tx.execute("UPDATE adaptation_runs SET status='running',dispatch_deadline=clock_timestamp()+($2::bigint * interval '1 second'),updated_at=clock_timestamp() WHERE id=$1 AND status='queued'", &[&id,&timeout_seconds]).await?;
        tx.commit().await?;
        // No transaction or lock spans the untrusted provider call. A timeout is ambiguous.
        let result = match tokio::time::timeout(
            Duration::from_secs(timeout_seconds as u64),
            provider.generate(&request),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(ProviderFailure::Ambiguous),
        };
        self.finish_adaptation(&actor, id, &frozen, result).await
    }

    async fn finish_adaptation(
        &self,
        actor: &str,
        id: &str,
        frozen: &FrozenAdaptation,
        result: Result<ProviderOutput, ProviderFailure>,
    ) -> Result<(), StoreError> {
        let (status, proposal, observation) = match result {
            Ok(output) => {
                let hash = format!("{:x}", Sha256::digest(&output.bytes));
                let admitted = match output.problem {
                    Some(error) => Err(problem(error.code())),
                    None => admit_output(&output.bytes, &frozen.source, &frozen.binding(id)),
                };
                let (status, proposal, problem) = match admitted {
                    Ok(proposal) => (AdaptationStatus::Succeeded, Some(proposal), None),
                    Err(error) => (AdaptationStatus::InvalidOutput, None, Some(error)),
                };
                (
                    status,
                    proposal,
                    Observation {
                        problem,
                        usage: output.usage,
                        cost: output.cost,
                        output_sha256: Some(hash),
                        output_byte_len: Some(output.bytes.len() as u64),
                    },
                )
            }
            Err(error) => {
                let status = if error.outcome_unknown() {
                    AdaptationStatus::Ambiguous
                } else {
                    AdaptationStatus::Failed
                };
                (
                    status,
                    None,
                    Observation {
                        problem: Some(problem(error.code())),
                        usage: None,
                        cost: None,
                        output_sha256: None,
                        output_byte_len: None,
                    },
                )
            }
        };
        let mut client = self.connection().await?;
        let tx = client.transaction().await?;
        let (response, _) = run_in_transaction(&tx, actor, id).await?;
        let json = serde_json::to_string(&observation).map_err(|_| StoreError::Unavailable)?;
        tx.execute("INSERT INTO adaptation_observations(run_id,kind,observation) VALUES($1,'provider_result',$2::text::jsonb) ON CONFLICT DO NOTHING", &[&id,&json]).await?;
        if response.status == AdaptationStatus::Running {
            let reason = observation
                .problem
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(|_| StoreError::Unavailable)?;
            let changed=tx.execute("UPDATE adaptation_runs SET status=$2,problem=$3::text::jsonb,dispatch_deadline=NULL,updated_at=clock_timestamp() WHERE id=$1 AND status='running' AND dispatch_deadline>clock_timestamp()", &[&id,&status_name(status),&reason]).await?;
            if changed == 1 {
                if let Some(proposal) = proposal {
                    let json =
                        serde_json::to_string(&proposal).map_err(|_| StoreError::Unavailable)?;
                    let digest = typed_digest(b"cantos/adaptation-proposal/p1\n", &proposal)?;
                    tx.execute("INSERT INTO adaptation_proposals(run_id,proposal,digest) VALUES($1,$2::text::jsonb,$3)", &[&id,&json,&digest]).await?;
                }
            } else {
                // The call completed at the deadline boundary. Preserve its observation,
                // classify the attempt as unknown, and do not retain an acceptable proposal.
                expire_dispatch(&tx, id).await?;
            }
        }
        // Late cancelled/expired results retain usage but cannot create or accept a proposal.
        tx.commit().await?;
        Ok(())
    }

    pub async fn cancel_adaptation(
        &self,
        token: &str,
        id: &str,
        request: CancelAdaptationRequest,
    ) -> Result<AdaptationRunResponse, StoreError> {
        normalized_uuid(id)?;
        normalized_uuid(&request.operation_id)?;
        let mut client = self.connection().await?;
        let tx = client.transaction().await?;
        let actor = authenticate(&tx, token).await?;
        tx.query_one(
            "SELECT pg_advisory_xact_lock(hashtextextended('adaptation-cancel:' || $1::text,0))",
            &[&actor],
        )
        .await?;
        let (response, _) = run_in_transaction(&tx, &actor, id).await?;
        authenticate(&tx, token).await?;
        if let Some(row) = tx
            .query_opt(
                "SELECT run_id FROM adaptation_cancellations WHERE owner_id=$1 AND operation_id=$2",
                &[&actor, &request.operation_id],
            )
            .await?
        {
            if row.get::<_, String>(0) != id {
                return Err(StoreError::OperationReused);
            }
        } else {
            if response.status == AdaptationStatus::Accepted {
                return Err(StoreError::InvalidRequest);
            }
            if response.status != AdaptationStatus::Cancelled {
                tx.execute("UPDATE adaptation_runs SET status='cancelled',dispatch_deadline=NULL,updated_at=clock_timestamp() WHERE id=$1", &[&id]).await?;
            }
            tx.execute("INSERT INTO adaptation_cancellations(owner_id,operation_id,run_id) VALUES($1,$2,$3)", &[&actor,&request.operation_id,&id]).await?;
        }
        let (response, _) = run_in_transaction(&tx, &actor, id).await?;
        tx.commit().await?;
        Ok(response)
    }

    pub async fn accept_adaptation(
        &self,
        token: &str,
        id: &str,
        request: AcceptAdaptationRequest,
    ) -> Result<RevisionResponse, StoreError> {
        normalized_uuid(id)?;
        normalized_uuid(&request.operation_id)?;
        checked_number(request.expected_revision)?;
        if !request.reviewed_findings {
            return Err(StoreError::InvalidRequestFields(vec![FieldIssue {
                path: "/reviewed_findings".into(),
                rule: "required_review".into(),
            }]));
        }
        let mut client = self.connection().await?;
        let tx = client.transaction().await?;
        let actor = authenticate(&tx, token).await?;
        tx.query_one(
            "SELECT pg_advisory_xact_lock(hashtextextended('adaptation-accept:' || $1::text,0))",
            &[&actor],
        )
        .await?;
        let (response, frozen) = run_in_transaction(&tx, &actor, id).await?;
        authenticate(&tx, token).await?;
        let prior = tx.query_opt(
            "SELECT run_id,request::text FROM adaptation_acceptances WHERE owner_id=$1 AND operation_id=$2",
            &[&actor, &request.operation_id],
        ).await?;
        if let Some(row) = prior {
            let prior: AcceptAdaptationRequest = serde_json::from_str(&row.get::<_, String>(1))
                .map_err(|_| StoreError::CorruptRevision)?;
            if row.get::<_, String>(0) != id || prior != request {
                return Err(StoreError::OperationReused);
            }
            let saved = response
                .accepted_revision
                .ok_or(StoreError::CorruptRevision)?;
            authenticate(&tx, token).await?;
            tx.commit().await?;
            return Ok(saved);
        }
        if response.status != AdaptationStatus::Succeeded
            || request.expected_revision != frozen.request.expected_revision
        {
            return Err(StoreError::InvalidRequest);
        }
        let content =
            admit_edited_proposal(&request.script_json, &frozen.source, &frozen.binding(id))
                .map_err(|value| StoreError::InvalidRequestFields(value.issues))?;
        let saved = save_in_transaction(
            &tx,
            &actor,
            &frozen.request.script_id,
            &SaveRevisionRequest {
                expected_revision: request.expected_revision,
                operation_id: request.operation_id.clone(),
                script_json: request.script_json.clone(),
            },
            &content,
        )
        .await?;
        let json = serde_json::to_string(&request).map_err(|_| StoreError::Unavailable)?;
        tx.execute("INSERT INTO adaptation_acceptances(run_id,owner_id,operation_id,request,script_id,revision) VALUES($1,$2,$3,$4::text::jsonb,$5,$6)", &[&id,&actor,&request.operation_id,&json,&saved.script_id,&checked_number(saved.revision)?]).await?;
        tx.execute("UPDATE adaptation_runs SET status='accepted',updated_at=clock_timestamp() WHERE id=$1 AND status='succeeded'", &[&id]).await?;
        authenticate(&tx, token).await?;
        tx.commit().await?;
        Ok(saved)
    }
}
