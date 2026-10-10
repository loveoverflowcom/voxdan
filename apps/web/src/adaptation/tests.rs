use super::*;
use cantos_api::{
    AdaptationConfig, AdaptationProposal, AdaptationProviderMetadata, Extraction, ImportMetadata,
};

const SCRIPT: &str = "00000000-0000-4000-8000-000000000001";
const RUN: &str = "00000000-0000-4000-8000-000000000002";
const SOURCE: &str = "src_00000000000040008000000000000001";

fn source() -> ImportResponse {
    ImportResponse {
        id: SOURCE.into(),
        imported_by: "alice".into(),
        recorded_at: "2026-10-10T10:00:00Z".into(),
        sha256: "test-checksum".into(),
        byte_len: 20,
        metadata: ImportMetadata {
            operation_id: "import-op".into(),
            file_name: "Mưa cuối sân khấu.txt".into(),
            format: cantos_api::ImportFormat::Txt,
            reference: "Original synthetic chapter".into(),
            rights_holder: None,
            permission_evidence: None,
            usage_scope: None,
        },
        outcome: ImportOutcome::Parsed {
            extraction: Extraction {
                extractor_version: "test-extraction".into(),
                blocks: vec![],
                warnings: vec![],
                script_json: None,
            },
        },
        original_text: Some("An: Ngày mai, mình có diễn tiếp không?".into()),
    }
}

fn ready() -> AdaptationReview {
    AdaptationReview {
        actor: "alice".into(),
        source_id: SOURCE.into(),
        script_id: SCRIPT.into(),
        rights_authorization: true,
        provider: Some(provider()),
        source: Some(Arc::new(source())),
        ..AdaptationReview::default()
    }
}

fn provider() -> AdaptationProviderMetadata {
    AdaptationProviderMetadata {
        provider: "test-double".into(),
        endpoint: "http://127.0.0.1:11434".into(),
        model: "contract-test".into(),
        prompt_version: "test-1".into(),
        contract_version: "test-1".into(),
        local_model_digest: None,
        config: AdaptationConfig {
            temperature_milli: 0,
            seed: 1,
            num_context: 1024,
            num_predict: 512,
            timeout_seconds: 1,
        },
    }
}

fn run(request: StartAdaptationRequest) -> AdaptationRunResponse {
    AdaptationRunResponse {
        id: RUN.into(),
        created_by: "alice".into(),
        recorded_at: "2026-10-10T10:00:00Z".into(),
        updated_at: "2026-10-10T10:01:00Z".into(),
        request,
        source_sha256: "test-checksum".into(),
        extractor_version: "test-extraction".into(),
        input_content_digest: None,
        input_export_digest: None,
        generation_record_id: "gen_test".into(),
        rights_record_id: "rights_test".into(),
        provider: provider(),
        status: AdaptationStatus::Succeeded,
        proposal: Some(AdaptationProposal {
            script_json: "{\"synthetic\":true}".into(),
            findings: vec![],
            coverage: vec![],
        }),
        problem: None,
        usage: None,
        cost: None,
        accepted_revision: None,
    }
}

fn opened() -> AdaptationReview {
    let (sending, intent) = ready().start_run("start-op".into()).unwrap();
    let Mutation::Start(request) = &intent.mutation else {
        panic!("start request");
    };
    sending.mutated(&intent, run(request.clone()))
}

fn error(code: ErrorCode) -> ApiError {
    ApiError {
        code,
        current_revision: None,
        issues: vec![],
    }
}

#[test]
fn lost_start_retries_exact_snapshot_without_new_generation_identity() {
    let (sending, intent) = ready().start_run("original-op".into()).unwrap();
    assert!(sending.start_run("duplicate".into()).is_none());
    let unavailable = sending.failed(&error(ErrorCode::Unavailable));
    assert_eq!(
        unavailable
            .edit_inputs(|state| state.source_id = "different".into())
            .source_id,
        SOURCE
    );
    let (retrying, retried) = unavailable.retry().unwrap();
    assert_eq!(retried, intent);
    let Mutation::Start(request) = &retried.mutation else {
        panic!("start request");
    };
    let success = retrying.mutated(&retried, run(request.clone()));
    assert_eq!(success.pending, None);
    assert_eq!(
        success.source.unwrap().original_text,
        source().original_text
    );
    assert_eq!(success.run_id, RUN);
}

#[test]
fn changed_actor_and_cookie_cannot_admit_a_private_proposal() {
    let (sending, intent) = ready().start_run("original-op".into()).unwrap();
    let Mutation::Start(request) = &intent.mutation else {
        panic!("start request");
    };
    let mut wrong_cookie = run(request.clone());
    wrong_cookie.created_by = "bob".into();
    let rejected = sending.mutated(&intent, wrong_cookie);
    assert_eq!(rejected.status, ReviewStatus::Error(ErrorCode::Forbidden));
    assert_eq!(rejected.pending, Some(intent.clone()));
    assert!(rejected.run.is_none());
    let changed = sending.signed_in_as("bob".into());
    assert!(changed.retry().is_none());
    let hidden = changed.mutated(&intent, run(request.clone()));
    assert!(hidden.run.is_none());
    assert_eq!(hidden.pending, Some(intent.clone()));
    assert_eq!(
        hidden.signed_in_as("alice".into()).retry().unwrap().1,
        intent
    );
}

#[test]
fn stale_reads_and_actor_changes_never_replace_local_text() {
    let edited = opened().edit_draft("Mưa vẫn rơi; An chưa nói tiếp.".into());
    let reading = edited.start_read().unwrap();
    let response = reading.run.as_ref().unwrap().as_ref().clone();
    let refresh = reading.loaded(reading.ticket, "alice", response.clone());
    assert_eq!(refresh.draft, edited.draft);
    assert!(refresh.draft_changed);
    assert_eq!(
        reading
            .loaded(reading.ticket - 1, "alice", response.clone())
            .status,
        ReviewStatus::LoadingRun
    );
    let changed = reading.signed_in_as("bob".into());
    assert!(changed
        .loaded(reading.ticket, "alice", response)
        .run
        .is_none());
    assert!(changed.accepted.is_none());
    assert_eq!(changed.draft, edited.draft);
    assert_ne!(changed.draft_actor, changed.actor);
}

#[test]
fn acceptance_needs_source_comparison_and_explicit_review_of_current_text() {
    let proposal = opened();
    assert!(proposal.start_accept("op".into()).is_none());
    let reviewed = proposal.review_findings(true);
    assert!(reviewed.start_accept("op".into()).is_some());
    assert!(reviewed
        .edit_draft("Edited dialogue".into())
        .start_accept("op".into())
        .is_none());
    let mut wrong_source = reviewed.clone();
    Arc::make_mut(wrong_source.source.as_mut().unwrap()).sha256 = "other-checksum".into();
    assert!(wrong_source.start_accept("op".into()).is_none());
    assert!(reviewed.new_run().run.is_none());
    assert!(!reviewed.new_run().rights_authorization);
}

#[test]
fn accepted_snapshot_and_later_typing_stay_separate_and_reopen_is_pinned() {
    let reviewed = opened().review_findings(true);
    let (sending, intent) = reviewed.start_accept("accept-op".into()).unwrap();
    let edited = sending.edit_draft("Later unsaved editing".into());
    let mut response = reviewed.run.as_ref().unwrap().as_ref().clone();
    response.status = AdaptationStatus::Accepted;
    response.accepted_revision = Some(RevisionResponse {
        script_id: SCRIPT.into(),
        revision: 1,
        accepted_by: "alice".into(),
        accepted_at: "2026-10-10T10:02:00Z".into(),
        content_digest: "content".into(),
        export_digest: "export".into(),
        script_json: "canonical accepted snapshot".into(),
    });
    let accepted = edited.accepted_result(&intent, response.accepted_revision.unwrap());
    assert_eq!(accepted.draft, edited.draft);
    assert_eq!(accepted.status, ReviewStatus::Accepted);
    assert!(accepted.pending.is_none());
    let reading = accepted.start_accepted_read().unwrap();
    let accepted_revision = accepted.accepted.as_ref().unwrap().as_ref().clone();
    assert_eq!(
        reading
            .accepted_loaded(reading.ticket, "alice", accepted_revision.clone())
            .status,
        ReviewStatus::AcceptedOpened
    );
    let mut wrong = accepted_revision;
    wrong.revision = 2;
    assert_eq!(
        reading
            .accepted_loaded(reading.ticket, "alice", wrong)
            .status,
        ReviewStatus::Error(ErrorCode::CorruptRevision)
    );
}

#[test]
fn stale_or_invalid_acceptance_keeps_draft_and_original_run_base() {
    let edited = opened()
        .edit_draft("Sửa lời thoại chưa được lưu".into())
        .review_findings(true);
    let (sending, _) = edited.start_accept("accept-op".into()).unwrap();
    for code in [
        ErrorCode::StaleRevision,
        ErrorCode::InvalidScript,
        ErrorCode::EvidenceUnavailable,
    ] {
        let failed = sending.failed(&error(code.clone()));
        assert_eq!(failed.status, ReviewStatus::Error(code));
        assert_eq!(failed.draft, edited.draft);
        assert_eq!(failed.run.as_ref().unwrap().request.expected_revision, 0);
        assert_eq!(failed.pending, None);
        assert!(failed.accepted.is_none());
    }
}

#[test]
fn ambiguous_acceptance_survives_expiry_and_uses_original_actor_and_json() {
    let (sending, intent) = opened()
        .review_findings(true)
        .start_accept("accept-op".into())
        .unwrap();
    let unavailable = sending
        .failed(&error(ErrorCode::Unavailable))
        .edit_draft("Newer local JSON".into());
    let expired = unavailable.failed(&error(ErrorCode::Unauthenticated));
    assert_eq!(expired.pending, Some(intent.clone()));
    assert_eq!(expired.retry().unwrap().1, intent);
    assert!(expired.signed_in_as("bob".into()).retry().is_none());
    assert_eq!(expired.draft, "Newer local JSON");
}

#[test]
fn reopening_a_run_checks_the_immutable_source_extraction_binding() {
    let opened = opened();
    let reading = opened.start_source_read().unwrap();
    let mut wrong = source();
    let ImportOutcome::Parsed { extraction } = &mut wrong.outcome else {
        panic!("parsed");
    };
    extraction.extractor_version = "different-version".into();
    assert_eq!(
        reading.source_loaded(reading.ticket, "alice", wrong).status,
        ReviewStatus::Error(ErrorCode::CorruptRevision)
    );
    let changed = reading.signed_in_as("bob".into());
    assert!(changed
        .source_loaded(reading.ticket, "alice", source())
        .source
        .is_none());
}

#[test]
fn canonical_run_path_grammar_cannot_interpret_an_arbitrary_url() {
    assert!(is_run_id(RUN));
    for id in [
        "../../private",
        "000000000000040008000000000000001",
        "00000000-0000-4000-8000-00000000000G",
        "00000000-0000-4000-8000-000000000001/accept",
    ] {
        assert!(!is_run_id(id));
    }
}

#[test]
fn actual_acceptance_revision_wire_contract_is_admitted_without_a_run_wrapper() {
    let literal = r#"{"script_id":"00000000-0000-4000-8000-000000000001","revision":1,"accepted_by":"alice","accepted_at":"2026-10-10T10:02:00Z","content_digest":"content","export_digest":"export","script_json":"accepted JSON"}"#;
    let revision: RevisionResponse = serde_json::from_str(literal).unwrap();
    assert!(serde_json::from_str::<AdaptationRunResponse>(literal).is_err());
    let (sending, intent) = opened()
        .review_findings(true)
        .start_accept("accept-op".into())
        .unwrap();
    let accepted = sending.accepted_result(&intent, revision.clone());
    assert_eq!(accepted.status, ReviewStatus::Accepted);
    assert_eq!(accepted.accepted.as_deref(), Some(&revision));
    assert_eq!(
        accepted.run.as_ref().unwrap().status,
        AdaptationStatus::Accepted
    );
    assert!(accepted.pending.is_none());
}

#[test]
fn provider_changes_reset_permission_and_exact_retry_keeps_original_destination() {
    let original = ready();
    let mut changed = provider();
    changed.model = "different-model".into();
    let changed_state = original.provider_observed(Some(changed.clone()));
    assert!(!changed_state.rights_authorization);
    assert!(!changed_state.can_start());
    assert!(!original.provider_observed(None).can_start());
    let mut changed_weights = provider();
    changed_weights.local_model_digest = Some("local-model:sha256:different-weights".into());
    assert!(
        !original
            .provider_observed(Some(changed_weights))
            .rights_authorization
    );
    let (sending, intent) = original.start_run("original-op".into()).unwrap();
    let observed_new_provider = sending
        .failed(&error(ErrorCode::Unavailable))
        .provider_observed(Some(changed));
    let (_, retried) = observed_new_provider.retry().unwrap();
    assert_eq!(retried, intent);
    let Mutation::Start(request) = retried.mutation else {
        panic!("start request");
    };
    assert_eq!(request.expected_provider, provider());
}

#[test]
fn composition_during_refresh_and_acceptance_preserves_local_text() {
    let reviewed = opened().review_findings(true);
    let (sending, intent) = reviewed.start_accept("accept-op".into()).unwrap();
    let composing = sending.begin_composition();
    let revision: RevisionResponse = serde_json::from_str(r#"{"script_id":"00000000-0000-4000-8000-000000000001","revision":1,"accepted_by":"alice","accepted_at":"2026-10-10T10:02:00Z","content_digest":"content","export_digest":"export","script_json":"accepted JSON"}"#).unwrap();
    let accepted = composing.accepted_result(&intent, revision);
    let committed = accepted.edit_draft("Mình vẫn diễn tiếp nhé?".into());
    assert_eq!(committed.draft, "Mình vẫn diễn tiếp nhé?");
    assert_eq!(
        committed.accepted.as_ref().unwrap().script_json,
        "accepted JSON"
    );
    assert!(committed.start_accept("new-op".into()).is_none());
}
