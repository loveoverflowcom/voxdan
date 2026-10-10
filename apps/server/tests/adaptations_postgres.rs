//! Independent persistence/HTTP oracles; only the disposable-cluster runner enables these.
//! All content is original synthetic Vietnamese text, never a private manuscript.
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use cantos_api::{
    AcceptAdaptationRequest, AdaptationConfig, AdaptationProviderMetadata, AdaptationRunResponse,
    AdaptationStatus, AdaptationUsage, CancelAdaptationRequest, ImportFormat, ImportMetadata,
    ImportRequest, ImportResponse, SaveRevisionRequest, StartAdaptationRequest,
};
use cantos_server::{
    adaptation::provider::{
        AdaptationProvider, OllamaProvider, ProviderFailure, ProviderFuture, ProviderOutput,
        ProviderRequest,
    },
    http::{router, AppState},
    postgres::{local_config, migrate, token_hash, Store, StoreError},
    script_ir::read_script,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
use tokio_postgres::{Client, Config, NoTls};
use tower::ServiceExt;
use uuid::Uuid;

const ALICE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BOB: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ORIGIN: &str = "http://127.0.0.1:8080";
const ORIGINAL: &str = "Đêm xuống, đèn Vọng Đài vẫn sáng.\r\n\r\nMai: Chúng mình sẽ chờ ở đây.\r\n\r\n— Ai đã mở cửa?\r\n";

#[derive(Clone)]
enum FixtureResult {
    Output(Vec<u8>),
    Rejected,
    MalformedResponse,
    OutputTooLarge,
    Ambiguous,
}

/// This is a deterministic contract double, never live AI evidence.
struct FixtureProvider {
    result: FixtureResult,
    gated: bool,
    entered: Semaphore,
    resume: Semaphore,
    calls: AtomicUsize,
    prompts: Mutex<Vec<String>>,
    timeout_seconds: u32,
}

impl FixtureProvider {
    fn new(result: FixtureResult, gated: bool, timeout_seconds: u32) -> Arc<Self> {
        Arc::new(Self {
            result,
            gated,
            entered: Semaphore::new(0),
            resume: Semaphore::new(0),
            calls: AtomicUsize::new(0),
            prompts: Mutex::new(vec![]),
            timeout_seconds,
        })
    }

    async fn dispatched(&self) {
        bounded(self.entered.acquire()).await.unwrap().forget();
    }

    fn release(&self) {
        self.resume.add_permits(1);
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl AdaptationProvider for FixtureProvider {
    fn metadata(&self) -> AdaptationProviderMetadata {
        AdaptationProviderMetadata {
            provider: "deterministic-integration-double".into(),
            endpoint: "http://127.0.0.1:fixture-only".into(),
            model: "synthetic-vietnamese-1".into(),
            prompt_version: "cantos-radio-adapt-1".into(),
            contract_version: "cantos-adaptation-1".into(),
            local_model_digest: None,
            config: AdaptationConfig {
                temperature_milli: 100,
                seed: 7,
                num_context: 8192,
                num_predict: 1024,
                timeout_seconds: self.timeout_seconds,
            },
        }
    }

    fn generate<'a>(&'a self, request: &'a ProviderRequest) -> ProviderFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.prompts.lock().unwrap().push(request.prompt.clone());
            self.entered.add_permits(1);
            if self.gated {
                self.resume.acquire().await.unwrap().forget();
            }
            match &self.result {
                FixtureResult::Output(bytes) => Ok(ProviderOutput {
                    bytes: bytes.clone(),
                    usage: Some(AdaptationUsage {
                        input_tokens: Some(87),
                        output_tokens: Some(123),
                    }),
                    cost: None,
                    problem: None,
                }),
                FixtureResult::Rejected => Err(ProviderFailure::Rejected),
                FixtureResult::MalformedResponse => Err(ProviderFailure::MalformedResponse),
                FixtureResult::OutputTooLarge => Err(ProviderFailure::OutputTooLarge),
                FixtureResult::Ambiguous => Err(ProviderFailure::Ambiguous),
            }
        })
    }
}

fn provider_document() -> Vec<u8> {
    json!({
        "contract_version": "cantos-adaptation-1",
        "title": "Ngọn đèn Vọng Đài",
        "characters": [{"name":"Mai","personality":"Chưa xác định"}],
        "scenes": [{
            "title":"Bên cánh cửa",
            "lines":[
                {"speaker":"Người dẫn chuyện","text":"Đêm xuống, đèn Vọng Đài vẫn sáng.","source_blocks":[0],"delivery":{"emotion":"calm","intensity_permille":300}},
                {"speaker":"Mai","text":"Chúng mình sẽ chờ ở đây.","source_blocks":[1],"delivery":{"emotion":"neutral","intensity_permille":300}},
                {"speaker":null,"text":"Ai đã mở cửa?","source_blocks":[2],"delivery":{"emotion":"neutral","intensity_permille":450}}
            ],
            "cues":[
                {"kind":"ambience","description":"Không khí đêm yên tĩnh","line_index":0,"edge":"start","source_blocks":[0]},
                {"kind":"music","description":"Nhạc nền cần tác giả duyệt","line_index":1,"edge":"start","source_blocks":[1]},
                {"kind":"sfx","description":"Tiếng cửa cần xác minh","line_index":2,"edge":"end","source_blocks":[2]}
            ],
            "pacing_note":"Nhịp chậm; khoảng dừng cần biên tập viên quyết định"
        }],
        "omitted_blocks":[],
        "review_notes":["Chưa xác định người đặt câu hỏi; không suy đoán danh tính"]
    }).to_string().into_bytes()
}

struct Harness {
    admin: Client,
    config: Config,
    store: Store,
    app: Router,
    database: String,
}

async fn connect(config: &Config) -> Client {
    let (client, connection) = config.connect(NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!(
                "test connection closed: {}",
                error.code().map(|code| code.code()).unwrap_or("transport")
            );
        }
    });
    client
}

impl Harness {
    async fn new(name: Option<&str>) -> Self {
        let url = env::var("CANTOS_TEST_CLUSTER_URL").expect("use scripts/test_postgres.py");
        let mut config = local_config(&url).unwrap();
        assert_eq!(config.get_user(), Some("cantos_test_admin"));
        assert_eq!(config.get_dbname(), Some("postgres"));
        let cluster = connect(&config).await;
        let database = name
            .map(str::to_owned)
            .unwrap_or_else(|| format!("cantos_test_{}", Uuid::new_v4().simple()));
        assert!(
            database.starts_with("cantos_test_")
                && database
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        );
        cluster
            .batch_execute(&format!("CREATE DATABASE {database}"))
            .await
            .unwrap();
        config.dbname(&database);
        migrate(&config).await.unwrap();
        let admin = connect(&config).await;
        admin.batch_execute("GRANT USAGE ON SCHEMA public TO cantos_app; GRANT SELECT ON actors,sessions,script_members,script_evidence,script_revisions,revision_evidence,scripts,source_records,script_reviews,script_review_operations TO cantos_app; GRANT INSERT ON scripts,script_revisions,revision_evidence,script_evidence,source_records TO cantos_app; GRANT UPDATE(head_revision) ON scripts TO cantos_app; GRANT UPDATE(revoked) ON sessions TO cantos_app; INSERT INTO actors(id) VALUES('alice'),('bob');").await.unwrap();
        admin.batch_execute("GRANT SELECT,INSERT ON adaptation_runs,adaptation_attempts,adaptation_observations,adaptation_proposals,adaptation_cancellations,adaptation_acceptances TO cantos_app; GRANT UPDATE(status,problem,dispatch_deadline,updated_at) ON adaptation_runs TO cantos_app;").await.unwrap();
        for (actor, token) in [("alice", ALICE), ("bob", BOB)] {
            admin.execute("INSERT INTO sessions(token_hash,actor_id,expires_at) VALUES($1,$2,CURRENT_TIMESTAMP+interval '1 hour')", &[&token_hash(token), &actor]).await.unwrap();
        }
        let mut app_config = config.clone();
        app_config
            .user("cantos_app")
            .application_name("cantos-adaptation-test");
        let store = Store::new(app_config).unwrap();
        let app = make_router(store.clone());
        Self {
            admin,
            config,
            store,
            app,
            database,
        }
    }

    fn provider(mut self, provider: Arc<FixtureProvider>) -> Self {
        self.store = self.store.with_adaptation_provider(provider);
        self.app = make_router(self.store.clone());
        self
    }

    async fn run(&self, request: StartAdaptationRequest) -> AdaptationRunResponse {
        self.store.start_adaptation(ALICE, request).await.unwrap()
    }

    async fn settled(&self, run: &str) -> AdaptationRunResponse {
        bounded(async {
            loop {
                let value = self.store.load_adaptation(ALICE, run).await.unwrap();
                if !matches!(
                    value.status,
                    AdaptationStatus::Queued | AdaptationStatus::Running
                ) {
                    return value;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
    }

    async fn counts(&self) -> (i64, i64, i64, i64, i64) {
        let row = self.admin.query_one("SELECT (SELECT count(*) FROM adaptation_runs),(SELECT count(*) FROM adaptation_attempts),(SELECT count(*) FROM adaptation_proposals),(SELECT count(*) FROM adaptation_acceptances),(SELECT count(*) FROM script_revisions)", &[]).await.unwrap();
        (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4))
    }

    async fn import(&self, token: &str) -> ImportResponse {
        self.store
            .import_manuscript(
                token,
                ImportRequest {
                    metadata: ImportMetadata {
                        operation_id: Uuid::new_v4().to_string(),
                        file_name: "Vọng Đài — synthetic.txt".into(),
                        format: ImportFormat::Txt,
                        reference: "Original synthetic adaptation integration fixture".into(),
                        rights_holder: Some("Synthetic test author".into()),
                        permission_evidence: Some(
                            "Local tests only; no publication clearance".into(),
                        ),
                        usage_scope: Some("private local adaptation".into()),
                    },
                    original_bytes: ORIGINAL.as_bytes().to_vec(),
                },
            )
            .await
            .unwrap()
    }

    async fn http(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(format!("/api/v1{path}"))
            .header("origin", ORIGIN)
            .header("content-type", "application/json")
            .header("x-actor-id", "alice");
        if let Some(token) = token {
            request = request.header("cookie", format!("cantos_session={token}"));
        }
        let response = self
            .app
            .clone()
            .oneshot(
                request
                    .body(Body::from(
                        body.map(|value| value.to_string()).unwrap_or_default(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    async fn originals_unchanged(&self, source: &ImportResponse) {
        assert_eq!(
            self.store.original_import(ALICE, &source.id).await.unwrap(),
            ORIGINAL.as_bytes()
        );
        assert_eq!(
            self.store.load_import(ALICE, &source.id).await.unwrap(),
            *source
        );
        assert_eq!(
            source.sha256,
            format!("{:x}", Sha256::digest(ORIGINAL.as_bytes()))
        );
    }
}

fn make_router(store: Store) -> Router {
    router(
        AppState {
            store,
            web_origin: ORIGIN.into(),
        },
        "apps/web/dist",
    )
}

fn operation() -> String {
    Uuid::new_v4().to_string()
}

fn start(source: &ImportResponse, script: &str, expected_revision: u64) -> StartAdaptationRequest {
    StartAdaptationRequest {
        operation_id: operation(),
        source_id: source.id.clone(),
        script_id: script.into(),
        expected_revision,
        rights_authorization: true,
        expected_provider: FixtureProvider::new(FixtureResult::Rejected, false, 120).metadata(),
    }
}

fn accept(run: &AdaptationRunResponse) -> AcceptAdaptationRequest {
    AcceptAdaptationRequest {
        operation_id: operation(),
        expected_revision: run.request.expected_revision,
        script_json: run.proposal.as_ref().unwrap().script_json.clone(),
        reviewed_findings: true,
    }
}

async fn expire_dispatch(h: &Harness, run: &str) {
    // Test owner injects the post-crash clock condition; application never has this privilege.
    h.admin
        .batch_execute("ALTER TABLE adaptation_runs DISABLE TRIGGER adaptation_run_guard")
        .await
        .unwrap();
    let result = h.admin.execute("UPDATE adaptation_runs SET dispatch_deadline=CURRENT_TIMESTAMP-interval '1 second' WHERE id=$1", &[&run]).await;
    h.admin
        .batch_execute("ALTER TABLE adaptation_runs ENABLE TRIGGER adaptation_run_guard")
        .await
        .unwrap();
    assert_eq!(result.unwrap(), 1);
}

async fn blocked_adaptation_connections(admin: &Client, expected: i64) {
    bounded(async {
        loop {
            let waiting: i64 = admin.query_one("SELECT count(*) FROM pg_stat_activity WHERE application_name='cantos-adaptation-test' AND wait_event_type='Lock'", &[]).await.unwrap().get(0);
            if waiting >= expected { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await;
}

async fn bounded<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("adaptation integration barrier timed out")
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn valid_proposal_pins_source_unknown_cost_and_uncertainty_before_explicit_acceptance() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let script = operation();
    let request = start(&source, &script, 0);
    let started = h.run(request.clone()).await;
    let run = h.settled(&started.id).await;
    assert_eq!(run.status, AdaptationStatus::Succeeded);
    assert_eq!(run.request, request);
    assert_eq!(run.created_by, "alice");
    assert_eq!(
        run.source_sha256,
        format!("{:x}", Sha256::digest(ORIGINAL.as_bytes()))
    );
    assert_eq!(run.extractor_version, "cantos-import-1");
    assert_eq!(run.provider, provider.metadata());
    assert_eq!(run.input_content_digest, None);
    assert_eq!(run.input_export_digest, None);
    assert_eq!(
        run.usage,
        Some(AdaptationUsage {
            input_tokens: Some(87),
            output_tokens: Some(123)
        })
    );
    assert_eq!(
        run.cost, None,
        "unknown provider cost is never reported as zero"
    );
    assert_eq!(run.accepted_revision, None);
    assert_eq!(provider.calls(), 1);
    assert_eq!(h.counts().await, (1, 1, 1, 0, 0));
    let proposal = run.proposal.as_ref().unwrap();
    let script_value: Value = serde_json::from_str(&proposal.script_json).unwrap();
    let admitted = read_script(proposal.script_json.as_bytes()).unwrap();
    assert_eq!(admitted.export_bytes(), proposal.script_json.as_bytes());
    assert_eq!(script_value["provenance"][0]["source_record_id"], source.id);
    assert!(script_value["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["generation_record_id"] == run.generation_record_id));
    assert_eq!(
        proposal
            .coverage
            .iter()
            .map(|item| item.block)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert!(proposal
        .findings
        .iter()
        .any(|finding| serde_json::to_value(finding).unwrap()["code"] == "unresolved_speaker"));
    assert!(proposal
        .findings
        .iter()
        .any(|finding| serde_json::to_value(finding).unwrap()["code"] == "source_warning"));
    assert!(proposal
        .findings
        .iter()
        .any(|finding| serde_json::to_value(finding).unwrap()["code"]
            == "unsupported_performance_control"));
    h.originals_unchanged(&source).await;
    let reviews: i64 = h
        .admin
        .query_one("SELECT count(*) FROM script_reviews", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        reviews, 0,
        "proposal does not grant editorial/publication approval"
    );
    let mut not_reviewed = accept(&run);
    not_reviewed.reviewed_findings = false;
    assert!(matches!(
        h.store
            .accept_adaptation(ALICE, &run.id, not_reviewed)
            .await,
        Err(StoreError::InvalidRequestFields(_))
    ));
    assert_eq!(h.counts().await, (1, 1, 1, 0, 0));
    let intent = accept(&run);
    let saved = h
        .store
        .accept_adaptation(ALICE, &run.id, intent.clone())
        .await
        .unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(saved.script_id, script);
    assert_eq!(saved.accepted_by, "alice");
    assert_eq!(saved.script_json, proposal.script_json);
    assert_eq!(
        h.store
            .accept_adaptation(ALICE, &run.id, intent)
            .await
            .unwrap(),
        saved
    );
    let accepted = h.store.load_adaptation(ALICE, &run.id).await.unwrap();
    assert_eq!(accepted.status, AdaptationStatus::Accepted);
    assert_eq!(accepted.accepted_revision, Some(saved.clone()));
    assert_eq!(accepted.proposal, run.proposal);
    assert_eq!(h.store.load(ALICE, &script, Some(1)).await.unwrap(), saved);
    assert_eq!(h.counts().await, (1, 1, 1, 1, 1));
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn concurrent_duplicates_dispatch_once_and_changed_operation_never_mutates_frozen_input() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), true, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let request = start(&source, &operation(), 0);
    let (first, second) = tokio::join!(
        h.store.start_adaptation(ALICE, request.clone()),
        h.store.start_adaptation(ALICE, request.clone())
    );
    let first = first.unwrap();
    assert_eq!(first.id, second.unwrap().id);
    provider.dispatched().await;
    assert_eq!(provider.calls(), 1);
    let frozen_before: String = h
        .admin
        .query_one(
            "SELECT frozen_input::text FROM adaptation_runs WHERE id=$1",
            &[&first.id],
        )
        .await
        .unwrap()
        .get(0);
    for variant in 0..3 {
        let mut changed = request.clone();
        match variant {
            0 => changed.script_id = operation(),
            1 => changed.expected_revision = 1,
            2 => changed.rights_authorization = false,
            _ => unreachable!(),
        }
        assert!(matches!(
            h.store.start_adaptation(ALICE, changed).await,
            Err(StoreError::OperationReused) | Err(StoreError::InvalidRequestFields(_))
        ));
    }
    assert_eq!(
        h.store
            .start_adaptation(ALICE, request.clone())
            .await
            .unwrap()
            .id,
        first.id
    );
    assert_eq!(h.counts().await, (1, 1, 0, 0, 0));
    provider.release();
    let run = h.settled(&first.id).await;
    assert_eq!(run.status, AdaptationStatus::Succeeded);
    let frozen_after: String = h
        .admin
        .query_one(
            "SELECT frozen_input::text FROM adaptation_runs WHERE id=$1",
            &[&first.id],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(frozen_before, frozen_after);
    assert_eq!(provider.calls(), 1);
    assert_eq!(h.store.start_adaptation(ALICE, request).await.unwrap(), run);
    assert_eq!(h.counts().await, (1, 1, 1, 0, 0));
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn wrong_owner_anonymous_revoked_and_expired_sessions_cannot_generate_read_or_accept() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), true, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let request = start(&source, &operation(), 0);
    assert_eq!(
        h.http("POST", "/adaptations", None, Some(json!(request)))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert!(matches!(
        h.store.start_adaptation(BOB, request.clone()).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(h.counts().await, (0, 0, 0, 0, 0));
    let started = h.run(request).await;
    provider.dispatched().await;
    let path = format!("/adaptations/{}", started.id);
    assert_eq!(
        h.http("GET", &path, Some(BOB), None).await.0,
        StatusCode::NOT_FOUND
    );
    h.admin
        .execute(
            "UPDATE sessions SET revoked=true WHERE actor_id='alice'",
            &[],
        )
        .await
        .unwrap();
    assert_eq!(
        h.http("GET", &path, Some(ALICE), None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert!(matches!(
        h.store
            .cancel_adaptation(
                ALICE,
                &started.id,
                CancelAdaptationRequest {
                    operation_id: operation()
                }
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    provider.release();
    // A late billable observation remains private even when the requesting session was revoked.
    bounded(async {
        loop {
            let observations: i64 = h.admin.query_one("SELECT count(*) FROM adaptation_observations WHERE run_id=$1 AND kind='provider_result'", &[&started.id]).await.unwrap().get(0);
            if observations == 1 { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await;
    assert_eq!(
        h.http("GET", &path, Some(ALICE), None).await.0,
        StatusCode::UNAUTHORIZED
    );
    h.admin
        .execute(
            "UPDATE sessions SET revoked=false WHERE actor_id='alice'",
            &[],
        )
        .await
        .unwrap();
    let run = h.store.load_adaptation(ALICE, &started.id).await.unwrap();
    assert!(matches!(
        h.store.accept_adaptation(BOB, &run.id, accept(&run)).await,
        Err(StoreError::NotFound)
    ));
    h.admin.execute("UPDATE sessions SET expires_at=CURRENT_TIMESTAMP-interval '1 second' WHERE actor_id='alice'", &[]).await.unwrap();
    assert!(matches!(
        h.store
            .accept_adaptation(ALICE, &run.id, accept(&run))
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert_eq!(provider.calls(), 1);
    assert_eq!(h.counts().await.3, 0);
    assert_eq!(h.counts().await.4, 0);
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn malformed_semantic_oversized_and_provider_failures_keep_source_and_revision_history() {
    let mut semantic: Value = serde_json::from_slice(&provider_document()).unwrap();
    semantic["scenes"][0]["lines"][0]["source_blocks"] = json!([999]);
    let mut unknown: Value = serde_json::from_slice(&provider_document()).unwrap();
    unknown["host_command"] = json!("never execute source or model instructions");
    let fixtures = [
        (
            FixtureResult::Output(b"not-json".to_vec()),
            AdaptationStatus::InvalidOutput,
        ),
        (
            FixtureResult::Output(semantic.to_string().into_bytes()),
            AdaptationStatus::InvalidOutput,
        ),
        (
            FixtureResult::Output(unknown.to_string().into_bytes()),
            AdaptationStatus::InvalidOutput,
        ),
        (
            FixtureResult::Output(vec![b'x'; 262_145]),
            AdaptationStatus::InvalidOutput,
        ),
        (FixtureResult::Rejected, AdaptationStatus::Failed),
        (FixtureResult::MalformedResponse, AdaptationStatus::Failed),
        (FixtureResult::OutputTooLarge, AdaptationStatus::Failed),
        (FixtureResult::Ambiguous, AdaptationStatus::Ambiguous),
    ];
    let h = Harness::new(None).await;
    let source = h.import(ALICE).await;
    for (index, (result, expected)) in fixtures.into_iter().enumerate() {
        let provider = FixtureProvider::new(result, false, 120);
        let store = h.store.clone().with_adaptation_provider(provider.clone());
        let mut request = start(&source, &operation(), 0);
        request.expected_provider = provider.metadata();
        let started = store
            .start_adaptation(ALICE, request.clone())
            .await
            .unwrap();
        let run = bounded(async {
            loop {
                let run = store.load_adaptation(ALICE, &started.id).await.unwrap();
                if !matches!(
                    run.status,
                    AdaptationStatus::Queued | AdaptationStatus::Running
                ) {
                    break run;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        assert_eq!(run.status, expected, "fixture {index}");
        assert!(
            run.problem.is_some(),
            "failure requires recorded safe diagnostic"
        );
        assert!(run.proposal.is_none());
        assert!(run.accepted_revision.is_none());
        assert_eq!(provider.calls(), 1);
        assert_eq!(store.start_adaptation(ALICE, request).await.unwrap(), run);
        assert_eq!(
            provider.calls(),
            1,
            "exact retry never silently generates again"
        );
        h.originals_unchanged(&source).await;
    }
    assert_eq!(h.counts().await, (8, 8, 0, 0, 0));
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn cancellation_is_idempotent_and_late_provider_result_cannot_publish_a_proposal() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), true, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let started = h.run(start(&source, &operation(), 0)).await;
    provider.dispatched().await;
    let cancel = CancelAdaptationRequest {
        operation_id: operation(),
    };
    let cancelled = h
        .store
        .cancel_adaptation(ALICE, &started.id, cancel.clone())
        .await
        .unwrap();
    assert_eq!(cancelled.status, AdaptationStatus::Cancelled);
    assert_eq!(
        h.store
            .cancel_adaptation(ALICE, &started.id, cancel.clone())
            .await
            .unwrap(),
        cancelled
    );
    let other = h.run(start(&source, &operation(), 0)).await;
    assert!(matches!(
        h.store.cancel_adaptation(ALICE, &other.id, cancel).await,
        Err(StoreError::OperationReused)
    ));
    let other_cancel = h
        .store
        .cancel_adaptation(
            ALICE,
            &other.id,
            CancelAdaptationRequest {
                operation_id: operation(),
            },
        )
        .await
        .unwrap();
    assert_eq!(other_cancel.status, AdaptationStatus::Cancelled);
    provider.release();
    bounded(async {
        loop {
            let observations: i64 = h.admin.query_one("SELECT count(*) FROM adaptation_observations WHERE run_id=$1 AND kind='provider_result'", &[&started.id]).await.unwrap().get(0);
            if observations == 1 { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await;
    let late = h.store.load_adaptation(ALICE, &started.id).await.unwrap();
    assert_eq!(late.status, AdaptationStatus::Cancelled);
    assert!(late.proposal.is_none());
    assert_eq!(
        late.usage,
        Some(AdaptationUsage {
            input_tokens: Some(87),
            output_tokens: Some(123)
        })
    );
    assert_eq!(late.cost, None);
    assert_eq!(
        provider.calls(),
        1,
        "cancelled queued contender never reaches provider"
    );
    assert_eq!(h.counts().await, (2, 1, 0, 0, 0));
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn timeout_expired_dispatch_and_reopened_store_are_ambiguous_without_automatic_regeneration()
{
    for timeout in [true, false] {
        let provider = FixtureProvider::new(
            FixtureResult::Output(provider_document()),
            true,
            if timeout { 1 } else { 120 },
        );
        let h = Harness::new(None).await.provider(provider.clone());
        let source = h.import(ALICE).await;
        let mut request = start(&source, &operation(), 0);
        request.expected_provider = provider.metadata();
        let started = h.run(request.clone()).await;
        provider.dispatched().await;
        if !timeout {
            expire_dispatch(&h, &started.id).await;
        }
        let run = h.settled(&started.id).await;
        assert_eq!(run.status, AdaptationStatus::Ambiguous);
        assert!(run.proposal.is_none());
        assert!(run.problem.is_some());
        let mut config = h.config.clone();
        config.user("cantos_app");
        let reopened = Store::new(config)
            .unwrap()
            .with_adaptation_provider(provider.clone());
        assert_eq!(reopened.load_adaptation(ALICE, &run.id).await.unwrap(), run);
        assert_eq!(
            reopened.start_adaptation(ALICE, request).await.unwrap(),
            run
        );
        assert_eq!(provider.calls(), 1);
        if !timeout {
            provider.release();
            bounded(async {
                loop {
                    let count: i64 = h.admin.query_one("SELECT count(*) FROM adaptation_observations WHERE run_id=$1 AND kind='provider_result'", &[&run.id]).await.unwrap().get(0);
                    if count == 1 { break; }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }).await;
        }
        assert_eq!(
            h.store
                .load_adaptation(ALICE, &run.id)
                .await
                .unwrap()
                .status,
            AdaptationStatus::Ambiguous
        );
        assert_eq!(h.counts().await, (1, 1, 0, 0, 0));
        h.originals_unchanged(&source).await;
    }
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn failed_run_creation_and_failed_terminal_proposal_commit_are_atomic_and_recoverable() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), true, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let request = start(&source, &operation(), 0);
    let registry_before: i64 = h
        .admin
        .query_one("SELECT count(*) FROM script_evidence", &[])
        .await
        .unwrap()
        .get(0);
    h.admin.batch_execute("CREATE TRIGGER injected_adaptation_create BEFORE INSERT ON adaptation_runs FOR EACH ROW EXECUTE FUNCTION reject_settled_change()").await.unwrap();
    assert!(matches!(
        h.store.start_adaptation(ALICE, request.clone()).await,
        Err(StoreError::Unavailable)
    ));
    assert_eq!(h.counts().await, (0, 0, 0, 0, 0));
    let registry_after: i64 = h
        .admin
        .query_one("SELECT count(*) FROM script_evidence", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        registry_before, registry_after,
        "run and pending evidence commit together"
    );
    assert_eq!(provider.calls(), 0);
    h.admin
        .batch_execute("DROP TRIGGER injected_adaptation_create ON adaptation_runs")
        .await
        .unwrap();
    let started = h.run(request.clone()).await;
    provider.dispatched().await;
    let mut blocker = connect(&h.config).await;
    let tx = blocker.transaction().await.unwrap();
    tx.query_one("SELECT pg_advisory_xact_lock(123456789)", &[])
        .await
        .unwrap();
    h.admin.batch_execute("CREATE FUNCTION crash_adaptation_proposal() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_advisory_xact_lock(123456789); PERFORM pg_terminate_backend(pg_backend_pid()); RETURN NEW; END $$; CREATE TRIGGER crash_adaptation_proposal_before_commit AFTER INSERT ON adaptation_proposals FOR EACH ROW EXECUTE FUNCTION crash_adaptation_proposal()").await.unwrap();
    provider.release();
    blocked_adaptation_connections(&h.admin, 1).await;
    assert_eq!(
        h.counts().await,
        (1, 1, 0, 0, 0),
        "uncommitted result is invisible at the trigger barrier"
    );
    tx.commit().await.unwrap();
    // Wait on PostgreSQL's terminated connection after the explicit trigger barrier.
    bounded(async {
        loop {
            let active: i64 = h.admin.query_one("SELECT count(*) FROM pg_stat_activity WHERE application_name='cantos-adaptation-test' AND state='active'", &[]).await.unwrap().get(0);
            if active == 0 { break; }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await;
    // A row-count oracle sees no half proposal/status commit. The same request cannot re-dispatch.
    assert_eq!(h.counts().await, (1, 1, 0, 0, 0));
    h.admin.batch_execute("DROP TRIGGER crash_adaptation_proposal_before_commit ON adaptation_proposals; DROP FUNCTION crash_adaptation_proposal()").await.unwrap();
    expire_dispatch(&h, &started.id).await;
    let recovered = h.store.load_adaptation(ALICE, &started.id).await.unwrap();
    assert_eq!(recovered.status, AdaptationStatus::Ambiguous);
    assert_eq!(
        h.store.start_adaptation(ALICE, request).await.unwrap(),
        recovered
    );
    assert_eq!(provider.calls(), 1);
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn stale_acceptance_keeps_pinned_base_and_newer_revision_while_concurrent_exact_accept_replays(
) {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let script = operation();
    let first = h.run(start(&source, &script, 0)).await;
    let first = h.settled(&first.id).await;
    let base = h
        .store
        .accept_adaptation(ALICE, &first.id, accept(&first))
        .await
        .unwrap();
    let next = h.run(start(&source, &script, 1)).await;
    let next = h.settled(&next.id).await;
    assert_eq!(
        next.input_content_digest.as_deref(),
        Some(base.content_digest.as_str())
    );
    assert_eq!(
        next.input_export_digest.as_deref(),
        Some(base.export_digest.as_str())
    );
    let mut value: Value = serde_json::from_str(&base.script_json).unwrap();
    value["episode"]["acts"][0]["scenes"][0]["dialogues"][1]["text"] =
        json!("Chúng mình sẽ chờ đến sáng.");
    let edited = h
        .store
        .save(
            ALICE,
            &script,
            SaveRevisionRequest {
                expected_revision: 1,
                operation_id: operation(),
                script_json: value.to_string(),
            },
        )
        .await
        .unwrap();
    assert!(matches!(
        h.store
            .accept_adaptation(ALICE, &next.id, accept(&next))
            .await,
        Err(StoreError::StaleRevision(2))
    ));
    assert_eq!(h.store.load(ALICE, &script, Some(1)).await.unwrap(), base);
    assert_eq!(h.store.load(ALICE, &script, None).await.unwrap(), edited);
    assert_eq!(
        h.store.load_adaptation(ALICE, &next.id).await.unwrap(),
        next
    );
    assert_eq!(h.counts().await, (2, 2, 2, 1, 2));
    let final_run = h.run(start(&source, &script, 2)).await;
    let final_run = h.settled(&final_run.id).await;
    let intent = accept(&final_run);
    let (first_save, second_save) = tokio::join!(
        h.store
            .accept_adaptation(ALICE, &final_run.id, intent.clone()),
        h.store
            .accept_adaptation(ALICE, &final_run.id, intent.clone())
    );
    let saved = first_save.unwrap();
    assert_eq!(second_save.unwrap(), saved);
    assert_eq!(saved.revision, 3);
    let mut changed = intent;
    let mut script_value: Value = serde_json::from_str(&changed.script_json).unwrap();
    script_value["episode"]["acts"][0]["scenes"][0]["dialogues"][1]["text"] =
        json!("Một lời khác cần phiên bản khác.");
    changed.script_json = script_value.to_string();
    assert!(matches!(
        h.store
            .accept_adaptation(ALICE, &final_run.id, changed)
            .await,
        Err(StoreError::OperationReused)
    ));
    assert_eq!(h.store.load(ALICE, &script, Some(1)).await.unwrap(), base);
    assert_eq!(h.store.load(ALICE, &script, Some(2)).await.unwrap(), edited);
    assert_eq!(h.store.load(ALICE, &script, None).await.unwrap(), saved);
    assert_eq!(h.counts().await, (3, 3, 3, 2, 3));
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn failed_acceptance_commit_rolls_back_revision_head_and_receipt_then_exact_retry_recovers() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(None).await.provider(provider);
    let source = h.import(ALICE).await;
    let started = h.run(start(&source, &operation(), 0)).await;
    let run = h.settled(&started.id).await;
    let intent = accept(&run);
    h.admin.batch_execute("CREATE TRIGGER injected_acceptance_insert BEFORE INSERT ON adaptation_acceptances FOR EACH ROW EXECUTE FUNCTION reject_settled_change()").await.unwrap();
    assert!(matches!(
        h.store
            .accept_adaptation(ALICE, &run.id, intent.clone())
            .await,
        Err(StoreError::Unavailable)
    ));
    assert_eq!(h.counts().await, (1, 1, 1, 0, 0));
    let scripts: i64 = h
        .admin
        .query_one("SELECT count(*) FROM scripts", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(scripts, 0, "provisional script insertion rolls back too");
    assert_eq!(h.store.load_adaptation(ALICE, &run.id).await.unwrap(), run);
    h.admin
        .batch_execute("DROP TRIGGER injected_acceptance_insert ON adaptation_acceptances")
        .await
        .unwrap();
    let saved = h
        .store
        .accept_adaptation(ALICE, &run.id, intent.clone())
        .await
        .unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(
        h.store
            .accept_adaptation(ALICE, &run.id, intent)
            .await
            .unwrap(),
        saved
    );
    assert_eq!(h.counts().await, (1, 1, 1, 1, 1));
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn settled_proposals_attempts_acceptances_and_revisions_are_immutable_for_application_and_owner(
) {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(None).await.provider(provider);
    let source = h.import(ALICE).await;
    let started = h.run(start(&source, &operation(), 0)).await;
    let run = h.settled(&started.id).await;
    let saved = h
        .store
        .accept_adaptation(ALICE, &run.id, accept(&run))
        .await
        .unwrap();
    let mut app_config = h.config.clone();
    app_config.user("cantos_app");
    let app = connect(&app_config).await;
    for sql in [
        "UPDATE adaptation_runs SET frozen_input=frozen_input",
        "DELETE FROM adaptation_runs",
        "TRUNCATE adaptation_runs CASCADE",
        "UPDATE adaptation_proposals SET proposal=proposal",
        "DELETE FROM adaptation_proposals",
        "UPDATE adaptation_attempts SET dispatched_at=dispatched_at",
        "UPDATE adaptation_observations SET observation=observation",
        "UPDATE adaptation_acceptances SET request=request",
        "UPDATE script_revisions SET canonical_export=canonical_export",
        "ALTER TABLE adaptation_runs DISABLE TRIGGER ALL",
    ] {
        assert_eq!(
            app.batch_execute(sql)
                .await
                .unwrap_err()
                .code()
                .unwrap()
                .code(),
            "42501",
            "{sql}"
        );
    }
    assert_eq!(
        app.batch_execute("UPDATE adaptation_runs SET status='succeeded' WHERE status='accepted'")
            .await
            .unwrap_err()
            .code()
            .unwrap()
            .code(),
        "55000"
    );
    for sql in [
        "UPDATE adaptation_runs SET frozen_input=frozen_input || '{\"forged_fact\":true}'::jsonb",
        "DELETE FROM adaptation_runs",
        "TRUNCATE adaptation_runs CASCADE",
        "UPDATE adaptation_proposals SET proposal=proposal",
        "DELETE FROM adaptation_proposals",
        "TRUNCATE adaptation_proposals CASCADE",
        "UPDATE adaptation_acceptances SET request=request",
        "UPDATE adaptation_attempts SET dispatched_at=dispatched_at",
        "UPDATE adaptation_observations SET observation=observation",
    ] {
        assert_eq!(
            h.admin
                .batch_execute(sql)
                .await
                .unwrap_err()
                .code()
                .unwrap()
                .code(),
            "55000",
            "{sql}"
        );
    }
    assert_eq!(
        h.store
            .load(ALICE, &saved.script_id, Some(1))
            .await
            .unwrap(),
        saved
    );
    h.admin
        .batch_execute(
            "ALTER TABLE adaptation_proposals DISABLE TRIGGER adaptation_proposal_immutable",
        )
        .await
        .unwrap();
    h.admin.execute("UPDATE adaptation_proposals SET proposal=jsonb_set(proposal,'{script_json}','\"{}\"'::jsonb) WHERE run_id=$1", &[&run.id]).await.unwrap();
    h.admin
        .batch_execute(
            "ALTER TABLE adaptation_proposals ENABLE TRIGGER adaptation_proposal_immutable",
        )
        .await
        .unwrap();
    assert!(matches!(
        h.store.load_adaptation(ALICE, &run.id).await,
        Err(StoreError::CorruptRevision)
    ));
    assert_eq!(
        h.store
            .load(ALICE, &saved.script_id, Some(1))
            .await
            .unwrap(),
        saved
    );
    h.originals_unchanged(&source).await;
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn deactivation_and_clock_expiry_while_waiting_for_start_lock_are_rechecked_before_commit() {
    for expired in [false, true] {
        let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
        let h = Harness::new(None).await.provider(provider.clone());
        let source = h.import(ALICE).await;
        let mut blocker = connect(&h.config).await;
        let tx = blocker.transaction().await.unwrap();
        tx.query_one(
            "SELECT pg_advisory_xact_lock(hashtextextended('adaptation:alice',0))",
            &[],
        )
        .await
        .unwrap();
        if expired {
            h.admin.execute("UPDATE sessions SET expires_at=clock_timestamp()+interval '200 milliseconds' WHERE actor_id='alice'", &[]).await.unwrap();
        }
        let store = h.store.clone();
        let intent = start(&source, &operation(), 0);
        let task = tokio::spawn(async move { store.start_adaptation(ALICE, intent).await });
        blocked_adaptation_connections(&h.admin, 1).await;
        if expired {
            // Database time advances after admission and before the second check at commit.
            h.admin
                .query_one("SELECT pg_sleep(0.4)", &[])
                .await
                .unwrap();
        } else {
            h.admin
                .execute("UPDATE actors SET active=false WHERE id='alice'", &[])
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
        assert!(matches!(
            task.await.unwrap(),
            Err(StoreError::Unauthenticated)
        ));
        assert_eq!(h.counts().await, (0, 0, 0, 0, 0));
        assert_eq!(provider.calls(), 0);
    }
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn actual_local_ollama_transport_records_truncated_envelope_usage_and_never_retries() {
    let requests = Arc::new(Mutex::new(vec![]));
    let observations = requests.clone();
    let status_probes = Arc::new(AtomicUsize::new(0));
    let status_observer = status_probes.clone();
    let show_probes = Arc::new(Mutex::new(vec![]));
    let show_observer = show_probes.clone();
    let content = String::from_utf8(provider_document()).unwrap();
    let app = Router::new().route(
        "/api/chat",
        axum::routing::post(move |axum::Json(request): axum::Json<Value>| {
            let observations = observations.clone();
            let content = content.clone();
            async move {
                observations.lock().unwrap().push(request.clone());
                axum::Json(json!({
                    "model":"fixture-test-double", "message":{"role":"assistant","content":content},
                    "done":true,"done_reason":"length","prompt_eval_count":87,"eval_count":123
                }))
            }
        }),
    ).route("/api/status", axum::routing::get(move || {
        status_observer.fetch_add(1, Ordering::SeqCst);
        async { axum::Json(json!({"cloud":{"disabled":true,"source":"env"}})) }
    })).route("/api/show", axum::routing::post(move |axum::Json(request): axum::Json<Value>| {
        show_observer.lock().unwrap().push(request);
        async { axum::Json(local_show()) }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let h = Harness::new(None).await;
    let source = h.import(ALICE).await;
    let config = AdaptationConfig {
        temperature_milli: 200,
        seed: 0,
        num_context: 8192,
        num_predict: 4096,
        timeout_seconds: 120,
    };
    let provider = Arc::new(
        OllamaProvider::connect(&format!("http://{address}"), "fixture-test-double", config)
            .await
            .unwrap(),
    );
    let metadata = provider.metadata();
    let store = h.store.clone().with_adaptation_provider(provider);
    let mut request = start(&source, &operation(), 0);
    request.expected_provider = metadata;
    let started = store
        .start_adaptation(ALICE, request.clone())
        .await
        .unwrap();
    let run = bounded(async {
        loop {
            let run = store.load_adaptation(ALICE, &started.id).await.unwrap();
            if !matches!(
                run.status,
                AdaptationStatus::Queued | AdaptationStatus::Running
            ) {
                break run;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    assert_eq!(run.status, AdaptationStatus::InvalidOutput);
    assert_eq!(
        run.usage,
        Some(AdaptationUsage {
            input_tokens: Some(87),
            output_tokens: Some(123)
        })
    );
    assert_eq!(run.cost, None);
    assert!(run.proposal.is_none());
    assert_eq!(store.start_adaptation(ALICE, request).await.unwrap(), run);
    {
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["model"], "fixture-test-double");
        assert_eq!(requests[0]["stream"], false);
        assert_eq!(requests[0]["keep_alive"], 0);
        assert_eq!(requests[0]["messages"][0]["role"], "system");
        assert_eq!(requests[0]["messages"][1]["role"], "user");
        let prompt: Value =
            serde_json::from_str(requests[0]["messages"][1]["content"].as_str().unwrap()).unwrap();
        assert_eq!(prompt["source"]["id"], source.id);
        assert_eq!(prompt["source"]["sha256"], source.sha256);
        assert_eq!(prompt["source"]["blocks"][2]["text"], "— Ai đã mở cửa?");
    }
    assert_eq!(h.counts().await, (1, 1, 0, 0, 0));
    assert_eq!(status_probes.load(Ordering::SeqCst), 2);
    assert_eq!(
        *show_probes.lock().unwrap(),
        vec![json!({"model":"fixture-test-double","verbose":false}); 2]
    );
    server.abort();
}

fn local_show() -> Value {
    json!({
        "details":{"format":"gguf","parameter_size":"1.0B"},
        "model_info":{"general.architecture":"llama"},
        "modelfile": format!("FROM /synthetic/blobs/sha256-{}", "1".repeat(64))
    })
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn changed_cloud_alias_or_local_weights_fail_before_manuscript_chat_dispatch() {
    for unsafe_mode in [1, 2, 3] {
        let mode = Arc::new(AtomicUsize::new(0));
        let status_mode = mode.clone();
        let show_mode = mode.clone();
        let chats = Arc::new(AtomicUsize::new(0));
        let chat_observer = chats.clone();
        let app = Router::new()
            .route(
                "/api/status",
                axum::routing::get(move || {
                    let disabled = status_mode.load(Ordering::SeqCst) != 1;
                    async move { axum::Json(json!({"cloud":{"disabled":disabled,"source":"env"}})) }
                }),
            )
            .route(
                "/api/show",
                axum::routing::post(move |axum::Json(request): axum::Json<Value>| {
                    assert_eq!(
                        request,
                        json!({"model":"fixture-test-double","verbose":false})
                    );
                    let mut value = local_show();
                    match show_mode.load(Ordering::SeqCst) {
                        2 => {
                            value["remote_model"] = json!("remote-aliased-model");
                            value["remote_host"] = json!("https://example.invalid");
                        }
                        3 => {
                            value["modelfile"] =
                                json!(format!("FROM /synthetic/blobs/sha256-{}", "2".repeat(64)))
                        }
                        _ => (),
                    }
                    async move { axum::Json(value) }
                }),
            )
            .route(
                "/api/chat",
                axum::routing::post(move || {
                    chat_observer.fetch_add(1, Ordering::SeqCst);
                    async { axum::Json(json!({"error":"manuscript_must_never_reach_this_route"})) }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let config = AdaptationConfig {
            temperature_milli: 200,
            seed: 0,
            num_context: 8192,
            num_predict: 4096,
            timeout_seconds: 120,
        };
        let provider = Arc::new(
            OllamaProvider::connect(&format!("http://{address}"), "fixture-test-double", config)
                .await
                .unwrap(),
        );
        let metadata = provider.metadata();
        assert!(metadata.local_model_digest.is_some());
        mode.store(unsafe_mode, Ordering::SeqCst);
        let h = Harness::new(None).await;
        let source = h.import(ALICE).await;
        let store = h.store.clone().with_adaptation_provider(provider);
        let mut intent = start(&source, &operation(), 0);
        intent.expected_provider = metadata;
        let started = store.start_adaptation(ALICE, intent.clone()).await.unwrap();
        let run = bounded(async {
            loop {
                let value = store.load_adaptation(ALICE, &started.id).await.unwrap();
                if !matches!(
                    value.status,
                    AdaptationStatus::Queued | AdaptationStatus::Running
                ) {
                    break value;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        assert_eq!(run.status, AdaptationStatus::Failed);
        assert_eq!(run.problem.as_ref().unwrap().code, "provider_rejected");
        assert_eq!(run.proposal, None);
        assert_eq!(run.usage, None);
        assert_eq!(run.cost, None);
        assert_eq!(store.start_adaptation(ALICE, intent).await.unwrap(), run);
        assert_eq!(
            chats.load(Ordering::SeqCst),
            0,
            "cloud/alias/weight change cannot receive source"
        );
        assert_eq!(h.counts().await, (1, 1, 0, 0, 0));
        h.originals_unchanged(&source).await;
        server.abort();
    }
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn adaptation_restart_fixture_preserves_exact_accepted_proposal_and_export() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(Some("cantos_test_adaptation_restart"))
        .await
        .provider(provider);
    let source = h.import(ALICE).await;
    let script = operation();
    let request = start(&source, &script, 0);
    let started = h.run(request.clone()).await;
    let run = h.settled(&started.id).await;
    let acceptance = accept(&run);
    let saved = h
        .store
        .accept_adaptation(ALICE, &run.id, acceptance.clone())
        .await
        .unwrap();
    let accepted = h.store.load_adaptation(ALICE, &run.id).await.unwrap();
    fs::create_dir_all("../../target/adaptation-evidence").unwrap();
    fs::write(
        "../../target/adaptation-evidence/provider-document.json",
        provider_document(),
    )
    .unwrap();
    fs::write("../../target/adaptation-evidence/restart.json", json!({
        "database":h.database,"script":script,"token":ALICE,
        "request":SaveRevisionRequest { expected_revision:0,operation_id:acceptance.operation_id.clone(),script_json:acceptance.script_json.clone() },
        "response":saved,"source":source,"start_request":request,"accept_request":acceptance,"run":accepted
    }).to_string()).unwrap();
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn bounded_adaptation_point_reads_keep_authorized_postgresql_index_baseline() {
    // Declared exploratory budget: warm p95 <=250ms, concurrency one, 1,000 private proposals.
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let seeded_at = Instant::now();
    let mut last = None;
    for _ in 0..1000 {
        let started = h.run(start(&source, &operation(), 0)).await;
        last = Some(h.settled(&started.id).await);
    }
    let seed_ms = seeded_at.elapsed().as_secs_f64() * 1000.0;
    let run = last.unwrap();
    assert_eq!(provider.calls(), 1000);
    h.admin.batch_execute("ANALYZE adaptation_runs; ANALYZE adaptation_proposals; ANALYZE adaptation_observations; ANALYZE source_records; ANALYZE sessions; ANALYZE actors").await.unwrap();
    let mut config = h.config.clone();
    config.user("cantos_app");
    let app = connect(&config).await;
    let point_plan: Vec<String> = app.query("EXPLAIN (ANALYZE, BUFFERS) SELECT id,frozen_input,status FROM adaptation_runs WHERE id=$1 AND owner_id=$2 FOR UPDATE", &[&run.id,&"alice"]).await.unwrap().into_iter().map(|row|row.get(0)).collect();
    let replay_plan: Vec<String> = app.query("EXPLAIN (ANALYZE, BUFFERS) SELECT id FROM adaptation_runs WHERE owner_id=$1 AND operation_id=$2", &[&"alice",&run.request.operation_id]).await.unwrap().into_iter().map(|row|row.get(0)).collect();
    assert!(point_plan
        .iter()
        .any(|line| line.contains("adaptation_runs_pkey")));
    assert!(replay_plan
        .iter()
        .any(|line| line.contains("adaptation_runs_owner_id_operation_id_key")));
    let path = format!("/adaptations/{}", run.id);
    for _ in 0..5 {
        assert_eq!(
            h.http("GET", &path, Some(ALICE), None).await.0,
            StatusCode::OK
        );
    }
    let mut timings_ms = vec![];
    for _ in 0..30 {
        let began = Instant::now();
        let (status, response) = h.http("GET", &path, Some(ALICE), None).await;
        timings_ms.push(began.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_value::<AdaptationRunResponse>(response).unwrap(),
            run
        );
    }
    timings_ms.sort_by(f64::total_cmp);
    let p50_ms = timings_ms[14];
    let p95_ms = timings_ms[28];
    let p99_ms = timings_ms[29];
    let pg_version: String = app.query_one("SELECT version()", &[]).await.unwrap().get(0);
    fs::create_dir_all("../../target/adaptation-evidence").unwrap();
    fs::write("../../target/adaptation-evidence/point-lookups.json", serde_json::to_string_pretty(&json!({
        "postgresql":pg_version,"database":h.database,"fixture_kind":"original synthetic Vietnamese source and deterministic provider contract double",
        "proposal_count":1000,"source_bytes":ORIGINAL.len(),"proposal_bytes":run.proposal.as_ref().unwrap().script_json.len(),"seed_ms":seed_ms,
        "concurrency":1,"warm_up_reads":5,"samples":30,"target_warm_p95_ms":250,
        "p50_ms":p50_ms,"p95_ms":p95_ms,"p99_ms":p99_ms,"timings_ms":timings_ms,
        "point_plan":point_plan,"replay_plan":replay_plan,
        "dbsp":"defer: actor-authorized indexed point lookups; no repeated aggregate consumer",
        "limits":"warm in-process HTTP Router with real application-role PostgreSQL; no socket, cold cache, high concurrency or production workload"
    })).unwrap()).unwrap();
    assert!(
        p95_ms <= 250.0,
        "declared warm p95 exceeded 250ms: {p95_ms}"
    );
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn authorized_provider_snapshot_mismatch_has_no_records_and_queued_recovery_never_changes_destination(
) {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), true, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    for variant in 0..11 {
        let mut request = start(&source, &operation(), 0);
        let metadata = &mut request.expected_provider;
        match variant {
            0 => metadata.provider.push_str("-changed"),
            1 => metadata.endpoint = "http://127.0.0.1:9999".into(),
            2 => metadata.model.push_str("-changed"),
            3 => metadata.prompt_version.push_str("-changed"),
            4 => metadata.contract_version.push_str("-changed"),
            5 => metadata.config.temperature_milli += 1,
            6 => metadata.config.seed += 1,
            7 => metadata.config.num_context += 1,
            8 => metadata.config.num_predict += 1,
            9 => metadata.config.timeout_seconds += 1,
            10 => metadata.local_model_digest = Some("sha256:unreviewed-weight-snapshot".into()),
            _ => unreachable!(),
        }
        match h.store.start_adaptation(ALICE, request).await {
            Err(StoreError::InvalidRequestFields(issues)) => assert_eq!(
                json!(issues),
                json!([{"path":"/expected_provider","rule":"provider_configuration_changed"}])
            ),
            result => panic!("provider mismatch was not rejected: {result:?}"),
        }
        assert_eq!(h.counts().await, (0, 0, 0, 0, 0));
        assert_eq!(provider.calls(), 0);
    }
    let request = start(&source, &operation(), 0);
    let occupier = h.run(request.clone()).await;
    provider.dispatched().await;
    let mut changed = request.clone();
    changed.expected_provider.model.push_str("-changed");
    assert!(matches!(
        h.store.start_adaptation(ALICE, changed).await,
        Err(StoreError::OperationReused)
    ));
    let queued = h.run(start(&source, &operation(), 0)).await;
    assert_eq!(queued.status, AdaptationStatus::Queued);
    let altered_provider =
        FixtureProvider::new(FixtureResult::Output(provider_document()), false, 121);
    let mut app_config = h.config.clone();
    app_config.user("cantos_app");
    let reopened = Store::new(app_config)
        .unwrap()
        .with_adaptation_provider(altered_provider.clone());
    let failed = bounded(async {
        loop {
            let value = reopened.load_adaptation(ALICE, &queued.id).await.unwrap();
            if value.status != AdaptationStatus::Queued {
                break value;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    assert_eq!(failed.status, AdaptationStatus::Failed);
    assert_eq!(
        failed.problem.as_ref().unwrap().code,
        "provider_configuration_changed"
    );
    assert_eq!(failed.provider, queued.provider);
    assert_eq!(failed.request, queued.request);
    assert_eq!(altered_provider.calls(), 0);
    provider.release();
    let settled = h.settled(&occupier.id).await;
    assert_eq!(
        reopened.start_adaptation(ALICE, request).await.unwrap(),
        settled
    );
    assert_eq!(provider.calls(), 1);
    assert_eq!(altered_provider.calls(), 0);
    assert_eq!(h.counts().await, (2, 1, 1, 0, 0));
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn queued_target_edit_fails_before_dispatch_and_keeps_immutable_base_revision() {
    let quick = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let mut h = Harness::new(None).await.provider(quick);
    let source = h.import(ALICE).await;
    let target = operation();
    let base_run = h.run(start(&source, &target, 0)).await;
    let base_run = h.settled(&base_run.id).await;
    let base = h
        .store
        .accept_adaptation(ALICE, &base_run.id, accept(&base_run))
        .await
        .unwrap();
    let gated = FixtureProvider::new(FixtureResult::Output(provider_document()), true, 120);
    h = h.provider(gated.clone());
    let occupier = h.run(start(&source, &operation(), 0)).await;
    gated.dispatched().await;
    let queued = h.run(start(&source, &target, 1)).await;
    assert_eq!(queued.status, AdaptationStatus::Queued);
    let mut value: Value = serde_json::from_str(&base.script_json).unwrap();
    value["episode"]["acts"][0]["scenes"][0]["dialogues"][1]["text"] =
        json!("Chúng mình trở về trước bình minh.");
    let edited = h
        .store
        .save(
            ALICE,
            &target,
            SaveRevisionRequest {
                operation_id: operation(),
                expected_revision: 1,
                script_json: value.to_string(),
            },
        )
        .await
        .unwrap();
    gated.release();
    assert_eq!(
        h.settled(&occupier.id).await.status,
        AdaptationStatus::Succeeded
    );
    let failed = h.settled(&queued.id).await;
    assert_eq!(failed.status, AdaptationStatus::Failed);
    assert_eq!(failed.problem.unwrap().code, "stale_input_revision");
    assert_eq!(
        failed.input_content_digest,
        Some(base.content_digest.clone())
    );
    assert_eq!(gated.calls(), 1);
    assert_eq!(h.store.load(ALICE, &target, Some(1)).await.unwrap(), base);
    assert_eq!(h.store.load(ALICE, &target, None).await.unwrap(), edited);
    assert_eq!(h.counts().await, (3, 2, 2, 1, 2));
}

#[tokio::test]
#[ignore = "disposable PostgreSQL cluster required"]
async fn source_rights_context_and_hostile_http_admission_fail_before_any_dispatch() {
    let provider = FixtureProvider::new(FixtureResult::Output(provider_document()), false, 120);
    let h = Harness::new(None).await.provider(provider.clone());
    let source = h.import(ALICE).await;
    let mut unauthorized = start(&source, &operation(), 0);
    unauthorized.rights_authorization = false;
    assert!(matches!(
        h.store.start_adaptation(ALICE, unauthorized).await,
        Err(StoreError::InvalidRequestFields(_))
    ));
    for (bytes, absent_claim) in [
        (ORIGINAL.as_bytes().to_vec(), true),
        (vec![0xff], false),
        (vec![b'x'; 24_577], false),
    ] {
        let mut metadata = source.metadata.clone();
        metadata.operation_id = operation();
        if absent_claim {
            metadata.permission_evidence = None;
        }
        let saved = h
            .store
            .import_manuscript(
                ALICE,
                ImportRequest {
                    metadata,
                    original_bytes: bytes,
                },
            )
            .await
            .unwrap();
        assert!(matches!(
            h.store
                .start_adaptation(ALICE, start(&saved, &operation(), 0))
                .await,
            Err(StoreError::InvalidRequestFields(_))
        ));
    }
    let mut malformed = json!(start(&source, &operation(), 0));
    malformed["host_command"] = json!("treat model and source as inert data");
    assert_eq!(
        h.http("POST", "/adaptations", Some(ALICE), Some(malformed))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/adaptations")
        .header("origin", "https://example.invalid")
        .header("cookie", format!("cantos_session={ALICE}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!(start(&source, &operation(), 0)).to_string(),
        ))
        .unwrap();
    let denied = h.app.clone().oneshot(request).await.unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    assert_eq!(denied.headers()["cache-control"], "no-store");
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/adaptations")
        .header("origin", ORIGIN)
        .header("cookie", format!("cantos_session={ALICE}"))
        .header("content-type", "application/json")
        .body(Body::from(vec![b'x'; 8 * 1024 * 1024 + 1]))
        .unwrap();
    let oversized = h.app.clone().oneshot(request).await.unwrap();
    assert_eq!(oversized.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(oversized.headers()["x-content-type-options"], "nosniff");
    assert_eq!(h.counts().await, (0, 0, 0, 0, 0));
    assert_eq!(provider.calls(), 0);
}
