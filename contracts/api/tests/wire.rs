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
