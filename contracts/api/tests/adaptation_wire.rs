use cantos_api::{AdaptationFindingCode, AdaptationStatus, StartAdaptationRequest};
use serde_json::{json, Value};

// Independently authored destination assertion; no backend serializer produced this fixture.
const START: &str = r#"{
  "operation_id":"00000000-0000-4000-8000-000000000003",
  "source_id":"src_synthetic", "script_id":"00000000-0000-4000-8000-000000000004",
  "expected_revision":0, "rights_authorization":true,
  "expected_provider":{
    "provider":"ollama-local", "endpoint":"http://127.0.0.1:11434", "model":"fixture:latest",
    "local_model_digest":null,
    "prompt_version":"cantos-radio-adapt-1", "contract_version":"cantos-adaptation-1",
    "config":{"temperature_milli":200,"seed":0,"num_context":8192,"num_predict":4096,"timeout_seconds":120}
  }
}"#;

#[test]
fn start_round_trip_retains_explicit_destination_model_and_scope_assertion() {
    let request: StartAdaptationRequest = serde_json::from_str(START).unwrap();
    assert_eq!(request.expected_provider.endpoint, "http://127.0.0.1:11434");
    assert_eq!(request.expected_provider.model, "fixture:latest");
    assert!(request.rights_authorization);
    assert_eq!(
        serde_json::to_value(request).unwrap(),
        serde_json::from_str::<Value>(START).unwrap()
    );
    let mut raw: Value = serde_json::from_str(START).unwrap();
    raw.as_object_mut().unwrap().remove("expected_provider");
    assert!(serde_json::from_value::<StartAdaptationRequest>(raw).is_err());
    let mut raw: Value = serde_json::from_str(START).unwrap();
    raw["approved"] = json!(true);
    assert!(serde_json::from_value::<StartAdaptationRequest>(raw).is_err());
}

#[test]
fn status_and_review_findings_are_closed_and_have_independent_wire_spellings() {
    for (value, expected) in [
        ("queued", AdaptationStatus::Queued),
        ("running", AdaptationStatus::Running),
        ("succeeded", AdaptationStatus::Succeeded),
        ("invalid_output", AdaptationStatus::InvalidOutput),
        ("failed", AdaptationStatus::Failed),
        ("ambiguous", AdaptationStatus::Ambiguous),
        ("cancelled", AdaptationStatus::Cancelled),
        ("accepted", AdaptationStatus::Accepted),
    ] {
        assert_eq!(
            serde_json::from_value::<AdaptationStatus>(json!(value)).unwrap(),
            expected
        );
        assert_eq!(serde_json::to_value(expected).unwrap(), json!(value));
    }
    assert_eq!(
        serde_json::from_value::<AdaptationFindingCode>(json!("unresolved_speaker")).unwrap(),
        AdaptationFindingCode::UnresolvedSpeaker
    );
    assert!(serde_json::from_value::<AdaptationStatus>(json!("publication_approved")).is_err());
    assert!(serde_json::from_value::<AdaptationFindingCode>(json!("rights_cleared")).is_err());
}
