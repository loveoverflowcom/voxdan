use cantos_api::RevisionResponse;
use cantos_server::script_ir::read_canonical_script;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[test]
fn studio_wire_fixtures_match_schema_and_independent_complete_export() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../contracts/schema/studio/v1.schema.json"
    ))
    .unwrap();
    for (name, bytes) in [
        (
            "Revision",
            include_str!("../../../contracts/fixtures/studio/v1/revision.json"),
        ),
        (
            "ApiError",
            include_str!("../../../contracts/fixtures/studio/v1/stale-revision.json"),
        ),
    ] {
        let schema = json!({"$ref":format!("#/$defs/{name}"),"$defs":schema["$defs"]});
        jsonschema::validator_for(&schema)
            .unwrap()
            .validate(&serde_json::from_str::<Value>(bytes).unwrap())
            .unwrap();
    }
    let revision: RevisionResponse = serde_json::from_str(include_str!(
        "../../../contracts/fixtures/studio/v1/revision.json"
    ))
    .unwrap();
    let content = read_canonical_script(revision.script_json.as_bytes()).unwrap();
    assert_eq!(
        content.content_digest().to_string(),
        revision.content_digest
    );
    let mut hash = Sha256::new();
    hash.update(b"cantos/script-export/e1\n");
    hash.update(revision.script_json.as_bytes());
    assert_eq!(
        format!("sir-e1:sha256:{:x}", hash.finalize()),
        revision.export_digest
    );
}
