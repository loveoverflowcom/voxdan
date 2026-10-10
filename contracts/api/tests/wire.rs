use cantos_api::{ApiError, ErrorCode, RevisionResponse};

#[test]
fn shared_consumer_decodes_the_committed_v1_revision_and_error_fixtures() {
    let bytes = include_str!("../../fixtures/studio/v1/revision.json");
    let revision: RevisionResponse = serde_json::from_str(bytes).unwrap();
    assert_eq!(revision.revision, 1);
    assert!(revision.script_json.contains("Người dẫn chuyện"));
    let encoded = serde_json::to_value(revision).unwrap();
    assert_eq!(
        encoded,
        serde_json::from_str::<serde_json::Value>(bytes).unwrap()
    );
    let error: ApiError =
        serde_json::from_str(include_str!("../../fixtures/studio/v1/stale-revision.json")).unwrap();
    assert_eq!(error.code, ErrorCode::StaleRevision);
    assert_eq!(error.current_revision, Some(2));
}

#[test]
fn editorial_extensions_round_trip_without_changing_the_revision_contract() {
    let history: cantos_api::HistoryResponse =
        serde_json::from_str(include_str!("../../fixtures/studio/v1/history.json")).unwrap();
    let review: cantos_api::ReviewResponse =
        serde_json::from_str(include_str!("../../fixtures/studio/v1/review.json")).unwrap();
    assert_eq!(history.reviews, vec![review]);
    assert_eq!(history.revisions[0].revision, 1);
    assert_eq!(history.next_after, None);
    let request = serde_json::json!({"operation_id":"00000000-0000-4000-8000-000000000003", "revision":1, "reviewed_by":"forged"});
    assert!(serde_json::from_value::<cantos_api::ReviewRequest>(request).is_err());
}
