//! Pure adaptation review state. A proposal never silently replaces an accepted revision.
use cantos_api::{
    AcceptAdaptationRequest, AdaptationProviderMetadata, AdaptationRunResponse, AdaptationStatus,
    ApiError, CancelAdaptationRequest, ErrorCode, FieldIssue, ImportOutcome, ImportResponse,
    RevisionResponse, StartAdaptationRequest,
};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewStatus {
    Idle,
    LoadingSource,
    SourceReady,
    Starting,
    LoadingRun,
    RunOpened,
    DraftChanged,
    Accepting,
    Accepted,
    Cancelling,
    LoadingAccepted,
    AcceptedOpened,
    Error(ErrorCode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mutation {
    Start(StartAdaptationRequest),
    Accept {
        run_id: String,
        script_id: String,
        request: AcceptAdaptationRequest,
    },
    Cancel {
        run_id: String,
        request: CancelAdaptationRequest,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdaptationIntent {
    pub actor: String,
    pub mutation: Mutation,
}

impl AdaptationIntent {
    pub fn operation_id(&self) -> &str {
        match &self.mutation {
            Mutation::Start(request) => &request.operation_id,
            Mutation::Accept { request, .. } => &request.operation_id,
            Mutation::Cancel { request, .. } => &request.operation_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdaptationReview {
    pub actor: String,
    pub source_id: String,
    pub script_id: String,
    pub expected_revision: String,
    pub rights_authorization: bool,
    pub provider: Option<AdaptationProviderMetadata>,
    pub source: Option<Arc<ImportResponse>>,
    pub run_id: String,
    pub run: Option<Arc<AdaptationRunResponse>>,
    pub draft: String,
    pub draft_actor: String,
    pub draft_changed: bool,
    pub reviewed_findings: bool,
    pub accepted: Option<Arc<RevisionResponse>>,
    pub pending: Option<AdaptationIntent>,
    pub ambiguous: bool,
    pub busy: bool,
    pub ticket: u64,
    pub issues: Vec<FieldIssue>,
    pub status: ReviewStatus,
}

impl Default for AdaptationReview {
    fn default() -> Self {
        Self {
            actor: String::new(),
            source_id: String::new(),
            script_id: String::new(),
            expected_revision: "0".into(),
            rights_authorization: false,
            provider: None,
            source: None,
            run_id: String::new(),
            run: None,
            draft: String::new(),
            draft_actor: String::new(),
            draft_changed: false,
            reviewed_findings: false,
            accepted: None,
            pending: None,
            ambiguous: false,
            busy: false,
            ticket: 0,
            issues: vec![],
            status: ReviewStatus::Idle,
        }
    }
}

pub fn is_run_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

impl AdaptationReview {
    pub fn blocked(&self) -> bool {
        self.busy || self.pending.is_some()
    }

    pub fn signed_in_as(&self, actor: String) -> Self {
        if actor == self.actor {
            return self.clone();
        }
        let mut next = self.clone();
        next.actor = actor;
        next.source = None;
        next.run = None;
        next.accepted = None;
        next.reviewed_findings = false;
        next.rights_authorization = false;
        next.provider = None;
        next.ticket = next.ticket.saturating_add(1);
        next.busy = false;
        next.issues.clear();
        next.status = if next.pending.is_some() {
            ReviewStatus::Error(ErrorCode::Forbidden)
        } else {
            ReviewStatus::Idle
        };
        next
    }

    pub fn provider_observed(&self, provider: Option<AdaptationProviderMetadata>) -> Self {
        let mut next = self.clone();
        if next.provider != provider {
            next.provider = provider;
            next.rights_authorization = false;
        }
        next
    }

    pub fn edit_inputs(&self, edit: impl FnOnce(&mut Self)) -> Self {
        if self.blocked() || self.run.is_some() {
            return self.clone();
        }
        let mut next = self.clone();
        let previous_source = next.source_id.clone();
        edit(&mut next);
        if next.source_id != previous_source {
            next.source = None;
            next.rights_authorization = false;
        }
        next.issues.clear();
        next.status = ReviewStatus::Idle;
        next
    }

    pub fn select_run(&self, id: String) -> Self {
        if self.blocked() || self.draft_changed || self.run_id == id {
            return self.clone();
        }
        let mut next = self.clone();
        next.run_id = id;
        next.run = None;
        next.source = None;
        next.accepted = None;
        next.draft.clear();
        next.draft_actor.clear();
        next.reviewed_findings = false;
        next.issues.clear();
        next.status = ReviewStatus::Idle;
        next
    }

    /// Explicit discard/new-run action; never available while an outcome is unresolved.
    pub fn new_run(&self) -> Self {
        if self.blocked() {
            return self.clone();
        }
        Self {
            actor: self.actor.clone(),
            source_id: self.source_id.clone(),
            script_id: self.script_id.clone(),
            expected_revision: self.expected_revision.clone(),
            ticket: self.ticket.saturating_add(1),
            ..Self::default()
        }
    }

    pub fn start_source_read(&self) -> Option<Self> {
        if self.blocked()
            || self.actor.is_empty()
            || !crate::import::is_import_source_id(&self.source_id)
        {
            return None;
        }
        let mut next = self.clone();
        next.ticket = self.ticket.checked_add(1)?;
        next.busy = true;
        next.status = ReviewStatus::LoadingSource;
        next.issues.clear();
        Some(next)
    }

    pub fn source_loaded(&self, ticket: u64, actor: &str, source: ImportResponse) -> Self {
        if ticket != self.ticket || actor != self.actor || source.id != self.source_id {
            return self.clone();
        }
        let mut next = self.clone();
        next.busy = false;
        if source.imported_by != actor {
            next.status = ReviewStatus::Error(ErrorCode::Forbidden);
            return next;
        }
        if self.run.as_ref().is_some_and(|run| {
            source.sha256 != run.source_sha256
                || !matches!(&source.outcome, ImportOutcome::Parsed { extraction } if extraction.extractor_version == run.extractor_version)
        }) {
            next.status = ReviewStatus::Error(ErrorCode::CorruptRevision);
            return next;
        }
        next.source = Some(Arc::new(source));
        next.status = ReviewStatus::SourceReady;
        next
    }

    pub fn start_run(&self, operation_id: String) -> Option<(Self, AdaptationIntent)> {
        if !self.can_start() {
            return None;
        }
        let expected_revision = self.expected_revision.parse().ok()?;
        let expected_provider = self.provider.clone()?;
        let intent = AdaptationIntent {
            actor: self.actor.clone(),
            mutation: Mutation::Start(StartAdaptationRequest {
                operation_id,
                source_id: self.source_id.clone(),
                script_id: self.script_id.clone(),
                expected_revision,
                rights_authorization: true,
                expected_provider,
            }),
        };
        Some((self.sending(intent.clone()), intent))
    }

    pub fn can_start(&self) -> bool {
        !self.blocked()
            && self.run.is_none()
            && !self.actor.is_empty()
            && self.rights_authorization
            && self.provider.is_some()
            && is_run_id(&self.script_id)
            && self.expected_revision.parse::<u64>().is_ok()
            && self.source.as_ref().is_some_and(|source| {
                source.imported_by == self.actor
                    && source.id == self.source_id
                    && matches!(source.outcome, ImportOutcome::Parsed { .. })
            })
    }

    pub fn edit_draft(&self, draft: String) -> Self {
        if self.draft_actor != self.actor
            || self.run.as_ref().is_none_or(|run| {
                !matches!(
                    run.status,
                    AdaptationStatus::Succeeded | AdaptationStatus::Accepted
                )
            })
        {
            return self.clone();
        }
        let mut next = self.clone();
        next.draft = draft;
        next.draft_changed = self
            .run
            .as_ref()
            .and_then(|run| run.proposal.as_ref())
            .is_some_and(|proposal| proposal.script_json != next.draft);
        next.reviewed_findings = false;
        if !self.blocked() {
            next.status = ReviewStatus::DraftChanged;
        }
        next
    }

    pub fn begin_composition(&self) -> Self {
        let mut next = self.clone();
        if self.draft_actor == self.actor
            && self.run.as_ref().is_some_and(|run| run.proposal.is_some())
        {
            next.draft_changed = true;
            next.reviewed_findings = false;
        }
        next
    }

    pub fn review_findings(&self, reviewed: bool) -> Self {
        let mut next = self.clone();
        if !self.blocked() && self.draft_actor == self.actor {
            next.reviewed_findings = reviewed;
        }
        next
    }

    pub fn start_accept(&self, operation_id: String) -> Option<(Self, AdaptationIntent)> {
        if !self.can_accept() {
            return None;
        }
        let run = self.run.as_ref()?;
        let intent = AdaptationIntent {
            actor: self.actor.clone(),
            mutation: Mutation::Accept {
                run_id: run.id.clone(),
                script_id: run.request.script_id.clone(),
                request: AcceptAdaptationRequest {
                    operation_id,
                    expected_revision: run.request.expected_revision,
                    script_json: self.draft.clone(),
                    reviewed_findings: true,
                },
            },
        };
        Some((self.sending(intent.clone()), intent))
    }

    pub fn can_accept(&self) -> bool {
        !self.blocked()
            && !self.actor.is_empty()
            && self.draft_actor == self.actor
            && self.reviewed_findings
            && !self.draft.trim().is_empty()
            && self.run.as_ref().is_some_and(|run| {
                run.created_by == self.actor
                    && run.status == AdaptationStatus::Succeeded
                    && self.source_matches_run(run)
            })
    }

    pub fn source_matches_run(&self, run: &AdaptationRunResponse) -> bool {
        self.source.as_ref().is_some_and(|source| {
            source.id == run.request.source_id
                && source.imported_by == self.actor
                && source.sha256 == run.source_sha256
                && matches!(&source.outcome, ImportOutcome::Parsed { extraction } if extraction.extractor_version == run.extractor_version)
        })
    }

    pub fn start_cancel(&self, operation_id: String) -> Option<(Self, AdaptationIntent)> {
        if !self.can_cancel() {
            return None;
        }
        let run = self.run.as_ref()?;
        let intent = AdaptationIntent {
            actor: self.actor.clone(),
            mutation: Mutation::Cancel {
                run_id: run.id.clone(),
                request: CancelAdaptationRequest { operation_id },
            },
        };
        Some((self.sending(intent.clone()), intent))
    }

    pub fn can_cancel(&self) -> bool {
        !self.blocked()
            && !self.actor.is_empty()
            && self.run.as_ref().is_some_and(|run| {
                run.created_by == self.actor
                    && matches!(
                        run.status,
                        AdaptationStatus::Queued
                            | AdaptationStatus::Running
                            | AdaptationStatus::Ambiguous
                    )
            })
    }

    fn sending(&self, intent: AdaptationIntent) -> Self {
        let mut next = self.clone();
        next.status = match &intent.mutation {
            Mutation::Start(_) => ReviewStatus::Starting,
            Mutation::Accept { .. } => ReviewStatus::Accepting,
            Mutation::Cancel { .. } => ReviewStatus::Cancelling,
        };
        next.pending = Some(intent);
        next.busy = true;
        next.issues.clear();
        next
    }

    pub fn retry(&self) -> Option<(Self, AdaptationIntent)> {
        let intent = self.pending.clone()?;
        if !self.can_retry() {
            return None;
        }
        Some((self.sending(intent.clone()), intent))
    }

    pub fn can_retry(&self) -> bool {
        !self.busy
            && self
                .pending
                .as_ref()
                .is_some_and(|intent| intent.actor == self.actor)
    }

    pub fn mutated(&self, intent: &AdaptationIntent, run: AdaptationRunResponse) -> Self {
        if self.pending.as_ref() != Some(intent) {
            return self.clone();
        }
        let matches = match &intent.mutation {
            Mutation::Start(request) => run.request == *request,
            Mutation::Accept { .. } => false,
            Mutation::Cancel { run_id, .. } => run.id == *run_id,
        };
        if run.created_by != intent.actor || !matches {
            let mut next = self.clone();
            next.busy = false;
            next.ambiguous = true;
            next.status = ReviewStatus::Error(if run.created_by == intent.actor {
                ErrorCode::CorruptRevision
            } else {
                ErrorCode::Forbidden
            });
            return next;
        }
        let mut next = self.clone();
        if next.actor != intent.actor {
            next.busy = false;
            next.ambiguous = true;
            next.status = ReviewStatus::Error(ErrorCode::Forbidden);
            return next;
        }
        next.pending = None;
        next.ambiguous = false;
        next.busy = false;
        next.admit_run(run)
    }

    /// Acceptance uses the existing revision receipt contract, not a guessed run response.
    pub fn accepted_result(&self, intent: &AdaptationIntent, revision: RevisionResponse) -> Self {
        if self.pending.as_ref() != Some(intent) {
            return self.clone();
        }
        let Mutation::Accept {
            run_id,
            script_id,
            request,
        } = &intent.mutation
        else {
            return self.clone();
        };
        let mut next = self.clone();
        next.busy = false;
        if revision.accepted_by != intent.actor
            || revision.script_id != *script_id
            || revision.revision != request.expected_revision.saturating_add(1)
        {
            next.ambiguous = true;
            next.status = ReviewStatus::Error(if revision.accepted_by == intent.actor {
                ErrorCode::CorruptRevision
            } else {
                ErrorCode::Forbidden
            });
            return next;
        }
        if self.actor != intent.actor {
            next.ambiguous = true;
            next.status = ReviewStatus::Error(ErrorCode::Forbidden);
            return next;
        }
        next.pending = None;
        next.ambiguous = false;
        if let Some(run) = &self.run {
            if run.id != *run_id {
                next.pending = Some(intent.clone());
                next.ambiguous = true;
                next.status = ReviewStatus::Error(ErrorCode::CorruptRevision);
                return next;
            }
            let mut run = run.as_ref().clone();
            run.status = AdaptationStatus::Accepted;
            run.accepted_revision = Some(revision.clone());
            next.run = Some(Arc::new(run));
        }
        next.accepted = Some(Arc::new(revision));
        next.status = ReviewStatus::Accepted;
        next
    }

    pub fn start_read(&self) -> Option<Self> {
        if !self.can_read() {
            return None;
        }
        let mut next = self.clone();
        next.busy = true;
        next.ticket = self.ticket.checked_add(1)?;
        next.issues.clear();
        next.status = ReviewStatus::LoadingRun;
        Some(next)
    }

    pub fn can_read(&self) -> bool {
        !self.blocked() && !self.actor.is_empty() && is_run_id(&self.run_id)
    }

    pub fn loaded(&self, ticket: u64, actor: &str, run: AdaptationRunResponse) -> Self {
        if ticket != self.ticket || actor != self.actor || run.id != self.run_id {
            return self.clone();
        }
        let mut next = self.clone();
        next.busy = false;
        if run.created_by != actor {
            next.status = ReviewStatus::Error(ErrorCode::Forbidden);
            return next;
        }
        next.admit_run(run)
    }

    fn admit_run(&self, run: AdaptationRunResponse) -> Self {
        let mut next = self.clone();
        next.run_id = run.id.clone();
        next.source_id = run.request.source_id.clone();
        next.script_id = run.request.script_id.clone();
        next.expected_revision = run.request.expected_revision.to_string();
        next.rights_authorization = run.request.rights_authorization;
        if !self.draft_changed || self.draft_actor != self.actor {
            next.draft = run
                .proposal
                .as_ref()
                .map(|proposal| proposal.script_json.clone())
                .unwrap_or_default();
            next.draft_actor = self.actor.clone();
            next.draft_changed = false;
        }
        next.accepted = run.accepted_revision.clone().map(Arc::new);
        next.status = if run.status == AdaptationStatus::Accepted {
            ReviewStatus::Accepted
        } else {
            ReviewStatus::RunOpened
        };
        next.run = Some(Arc::new(run));
        next
    }

    pub fn failed(&self, error: &ApiError) -> Self {
        let mut next = self.clone();
        next.busy = false;
        next.issues = error.issues.clone();
        next.status = ReviewStatus::Error(error.code.clone());
        if matches!(
            error.code,
            ErrorCode::Unavailable | ErrorCode::CorruptRevision
        ) {
            next.ambiguous = next.pending.is_some();
        } else if !(self.ambiguous
            && matches!(
                error.code,
                ErrorCode::Unauthenticated | ErrorCode::Forbidden | ErrorCode::NotFound
            ))
        {
            next.pending = None;
            next.ambiguous = false;
        }
        next
    }

    pub fn read_failed(&self, ticket: u64, actor: &str, error: &ApiError) -> Self {
        if ticket != self.ticket || actor != self.actor {
            return self.clone();
        }
        self.failed(error)
    }

    pub fn start_accepted_read(&self) -> Option<Self> {
        if self.blocked() || self.actor.is_empty() || self.accepted.is_none() {
            return None;
        }
        let mut next = self.clone();
        next.busy = true;
        next.ticket = next.ticket.checked_add(1)?;
        next.status = ReviewStatus::LoadingAccepted;
        Some(next)
    }

    pub fn accepted_loaded(&self, ticket: u64, actor: &str, revision: RevisionResponse) -> Self {
        if ticket != self.ticket || actor != self.actor {
            return self.clone();
        }
        let Some(accepted) = &self.accepted else {
            return self.clone();
        };
        if &revision != accepted.as_ref() {
            return self.read_failed(
                ticket,
                actor,
                &ApiError {
                    code: ErrorCode::CorruptRevision,
                    current_revision: None,
                    issues: vec![],
                },
            );
        }
        let mut next = self.clone();
        next.busy = false;
        next.status = ReviewStatus::AcceptedOpened;
        next
    }
}

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(target_arch = "wasm32")]
pub use browser::ScriptAdaptation;

#[cfg(test)]
mod tests;
