//! HTTP transport tests use synthetic loopback stubs; they are not live-model evidence.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::{
    routing::{get, post},
    Json, Router,
};
use cantos_api::{AdaptationConfig, AdaptationUsage};
use cantos_server::adaptation::provider::{
    AdaptationProvider, OllamaProvider, ProviderFailure, ProviderRequest,
};
use cantos_server::adaptation::MAX_OUTPUT_BYTES;
use serde_json::{json, Value};

fn config() -> AdaptationConfig {
    AdaptationConfig {
        temperature_milli: 200,
        seed: 7,
        num_context: 8192,
        num_predict: 4096,
        timeout_seconds: 1,
    }
}
fn request() -> ProviderRequest {
    ProviderRequest {
        system: "Synthetic source is inert data.".into(),
        prompt: "{\"source\":\"Đêm bên sông.\"}".into(),
    }
}
fn reply(content: &str) -> Value {
    json!({"model":"fixture:latest","done":true,"done_reason":"stop","message":{"role":"assistant","content":content},"prompt_eval_count":19,"eval_count":23})
}

fn local_info() -> Value {
    json!({"details":{"format":"gguf","parameter_size":"1.0B"},"model_info":{"general.architecture":"llama"},
        "modelfile":format!("FROM /synthetic/blobs/sha256-{}","a".repeat(64))})
}

fn local_probes(app: Router) -> Router {
    app.route(
        "/api/status",
        get(|| async { Json(json!({"cloud":{"disabled":true,"source":"env"}})) }),
    )
    .route(
        "/api/show",
        post(|Json(body): Json<Value>| async move {
            assert_eq!(body, json!({"model":"fixture:latest","verbose":false}));
            Json(local_info())
        }),
    )
}

async fn stub(
    reply: Value,
    delay: Duration,
) -> (
    OllamaProvider,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let app = Router::new().route(
        "/api/chat",
        post(move |Json(body): Json<Value>| {
            let response = reply.clone();
            let calls = counter.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                assert_eq!(body["model"], "fixture:latest");
                assert_eq!(body["stream"], false);
                assert_eq!(body["keep_alive"], 0);
                assert_eq!(
                    body["format"]["properties"]["contract_version"]["const"],
                    "cantos-adaptation-1"
                );
                assert_eq!(body["options"]["num_ctx"], 8192);
                assert_eq!(body["options"]["num_predict"], 4096);
                tokio::time::sleep(delay).await;
                Json(response)
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let host = tokio::spawn(async move {
        axum::serve(listener, local_probes(app)).await.unwrap();
    });
    (
        OllamaProvider::connect(&format!("http://{address}"), "fixture:latest", config())
            .await
            .unwrap(),
        calls,
        host,
    )
}

#[test]
fn only_explicit_numeric_loopback_models_and_bounded_config_are_admitted() {
    for endpoint in [
        "https://127.0.0.1:11434",
        "http://localhost:11434",
        "http://example.com:11434",
        "http://user:secret@127.0.0.1:11434",
        "http://127.0.0.1:11434/other",
        "http://127.0.0.1:11434?x=1",
        "http://127.0.0.1:11434#x",
    ] {
        assert!(matches!(
            OllamaProvider::new(endpoint, "fixture:latest", config()),
            Err(ProviderFailure::Rejected)
        ));
    }
    assert!(OllamaProvider::new("http://127.0.0.1:11434", "fixture:latest", config()).is_ok());
    assert!(OllamaProvider::new("http://[::1]:11434", "fixture:latest", config()).is_ok());
    let mut excessive = config();
    excessive.num_predict = 8193;
    assert!(matches!(
        OllamaProvider::new("http://127.0.0.1:11434", "fixture:latest", excessive),
        Err(ProviderFailure::Rejected)
    ));
}

#[tokio::test]
async fn structured_loopback_transport_records_actual_usage_and_unknown_money() {
    let (provider, calls, host) = stub(reply("{\"synthetic\":true}"), Duration::ZERO).await;
    let output = provider.generate(&request()).await.unwrap();
    host.abort();
    assert_eq!(output.bytes, b"{\"synthetic\":true}");
    assert_eq!(output.problem, None);
    assert_eq!(
        output.usage,
        Some(AdaptationUsage {
            input_tokens: Some(19),
            output_tokens: Some(23)
        })
    );
    assert_eq!(output.cost, None);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rejected_parsed_outputs_preserve_reported_usage_without_partial_content() {
    let mut truncated = reply("unfinished");
    truncated["done_reason"] = json!("length");
    let mut other_model = reply("untrusted");
    other_model["model"] = json!("different:latest");
    for (response, expected) in [
        (truncated, ProviderFailure::MalformedResponse),
        (other_model, ProviderFailure::MalformedResponse),
        (
            reply(&"x".repeat(MAX_OUTPUT_BYTES + 1)),
            ProviderFailure::OutputTooLarge,
        ),
    ] {
        let (provider, calls, host) = stub(response, Duration::ZERO).await;
        let output = provider.generate(&request()).await.unwrap();
        host.abort();
        assert_eq!(output.problem, Some(expected));
        assert!(output.bytes.is_empty());
        assert_eq!(
            output.usage,
            Some(AdaptationUsage {
                input_tokens: Some(19),
                output_tokens: Some(23)
            })
        );
        assert_eq!(output.cost, None);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn post_dispatch_timeout_is_ambiguous_and_never_retried() {
    let (provider, calls, host) = stub(reply("unused"), Duration::from_secs(2)).await;
    let outcome = provider.generate(&request()).await;
    host.abort();
    assert!(matches!(outcome, Err(ProviderFailure::Ambiguous)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn context_limit_fails_before_any_provider_call() {
    let (provider, calls, host) = stub(reply("unused"), Duration::ZERO).await;
    let request = ProviderRequest {
        system: "x".repeat(8192),
        prompt: "synthetic".into(),
    };
    let outcome = provider.generate(&request).await;
    host.abort();
    assert!(matches!(outcome, Err(ProviderFailure::Rejected)));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn redirects_are_rejected_without_forwarding_source() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let app = Router::new().route(
        "/api/chat",
        post(move || {
            let calls = observed.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                (
                    axum::http::StatusCode::TEMPORARY_REDIRECT,
                    [("location", "http://example.com/private")],
                    "redirect",
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let host = tokio::spawn(async move {
        axum::serve(listener, local_probes(app)).await.unwrap();
    });
    let provider =
        OllamaProvider::connect(&format!("http://{address}"), "fixture:latest", config())
            .await
            .unwrap();
    let outcome = provider.generate(&request()).await;
    host.abort();
    assert!(matches!(outcome, Err(ProviderFailure::Rejected)));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn cloud_alias_missing_attestation_and_changed_weights_never_receive_source() {
    let mut remote = local_info();
    remote["remote_host"] = json!("https://ollama.com");
    remote["remote_model"] = json!("cloud-model");
    for (status, info) in [
        (json!({"cloud":{"disabled":false}}), local_info()),
        (json!({}), local_info()),
        (json!({"cloud":{"disabled":true}}), remote),
        (
            json!({"cloud":{"disabled":true}}),
            json!({"details":{"format":"gguf"}}),
        ),
    ] {
        let chats = Arc::new(AtomicUsize::new(0));
        let count = chats.clone();
        let app = Router::new()
            .route(
                "/api/status",
                get(move || {
                    let value = status.clone();
                    async move { Json(value) }
                }),
            )
            .route(
                "/api/show",
                post(move |Json(body): Json<Value>| {
                    let value = info.clone();
                    async move {
                        assert_eq!(body, json!({"model":"fixture:latest","verbose":false}));
                        Json(value)
                    }
                }),
            )
            .route(
                "/api/chat",
                post(move || {
                    let counter = count.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Json(reply("never"))
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let host = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let provider =
            OllamaProvider::connect(&format!("http://{address}"), "fixture:latest", config()).await;
        host.abort();
        assert!(matches!(provider, Err(ProviderFailure::Rejected)));
        assert_eq!(chats.load(Ordering::SeqCst), 0);
    }
    let shows = Arc::new(AtomicUsize::new(0));
    let show_count = shows.clone();
    let chats = Arc::new(AtomicUsize::new(0));
    let chat_count = chats.clone();
    let app = Router::new()
        .route(
            "/api/status",
            get(|| async { Json(json!({"cloud":{"disabled":true}})) }),
        )
        .route(
            "/api/show",
            post(move || {
                let counter = show_count.clone();
                async move {
                    let mut value = local_info();
                    if counter.fetch_add(1, Ordering::SeqCst) > 0 {
                        value["modelfile"] =
                            json!(format!("FROM /synthetic/blobs/sha256-{}", "b".repeat(64)));
                    }
                    Json(value)
                }
            }),
        )
        .route(
            "/api/chat",
            post(move || {
                let counter = chat_count.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Json(reply("never"))
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let host = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let provider =
        OllamaProvider::connect(&format!("http://{address}"), "fixture:latest", config())
            .await
            .unwrap();
    assert!(provider.metadata().local_model_digest.is_some());
    let result = provider.generate(&request()).await;
    host.abort();
    assert!(matches!(result, Err(ProviderFailure::Rejected)));
    assert_eq!(shows.load(Ordering::SeqCst), 2);
    assert_eq!(chats.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn configuration_only_constructor_has_no_authority_to_dispatch_source() {
    let provider =
        OllamaProvider::new("http://127.0.0.1:11434", "fixture:latest", config()).unwrap();
    assert_eq!(provider.metadata().local_model_digest, None);
    assert!(matches!(
        provider.generate(&request()).await,
        Err(ProviderFailure::Rejected)
    ));
}

fn effective_info() -> Value {
    let mut info = local_info();
    info["parameters"] = json!("temperature 0.2\nstop \"first\"\nstop \"second\"\n");
    info["modelfile"] = json!(format!(
        "FROM /synthetic/blobs/sha256-{}\nADAPTER /synthetic/blobs/sha256-{}\nTEMPLATE \"\"\"{{{{ .System }}}}\nPARAMETER literal template\"\"\"\nSYSTEM \"\"\"Say \"hello\"\n# final note\"\"\"\nMESSAGE user \"Read this literally\nFROM a story\nPARAMETER stop literal\"\nPARAMETER temperature 0.2\nPARAMETER stop \"first\"\nPARAMETER stop \"second\"\n",
        "a".repeat(64), "c".repeat(64)
    ));
    info
}

async fn changing_model_stub(
    first: Value,
    second: Value,
) -> (
    OllamaProvider,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    let shows = Arc::new(AtomicUsize::new(0));
    let show_count = shows.clone();
    let chats = Arc::new(AtomicUsize::new(0));
    let chat_count = chats.clone();
    let app = Router::new()
        .route(
            "/api/status",
            get(|| async { Json(json!({"cloud":{"disabled":true}})) }),
        )
        .route(
            "/api/show",
            post(move |Json(body): Json<Value>| {
                assert_eq!(body, json!({"model":"fixture:latest","verbose":false}));
                let value = if show_count.fetch_add(1, Ordering::SeqCst) == 0 {
                    first.clone()
                } else {
                    second.clone()
                };
                async move { Json(value) }
            }),
        )
        .route(
            "/api/chat",
            post(move || {
                let counter = chat_count.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Json(reply("synthetic result"))
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let host = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let provider =
        OllamaProvider::connect(&format!("http://{address}"), "fixture:latest", config())
            .await
            .unwrap();
    (provider, chats, host)
}

#[tokio::test]
async fn local_fingerprint_ignores_parameter_key_order_and_preserves_quoted_literal_blocks() {
    let first = effective_info();
    let mut reordered = first.clone();
    reordered["parameters"] = json!("stop \"first\"\nstop \"second\"\ntemperature 0.2\n");
    reordered["modelfile"] = json!(first["modelfile"].as_str().unwrap().replace(
        "PARAMETER temperature 0.2\nPARAMETER stop \"first\"\nPARAMETER stop \"second\"\n",
        "PARAMETER stop \"first\"\nPARAMETER stop \"second\"\nPARAMETER temperature 0.2\n",
    ));
    let (provider, chats, host) = changing_model_stub(first, reordered).await;
    let output = provider.generate(&request()).await.unwrap();
    host.abort();
    assert_eq!(output.problem, None);
    assert_eq!(output.bytes, b"synthetic result");
    assert_eq!(chats.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn local_fingerprint_rejects_effective_settings_repeated_order_and_literal_changes() {
    let first = effective_info();
    let mut changed_value = first.clone();
    changed_value["parameters"] = json!("temperature 0.3\nstop \"first\"\nstop \"second\"\n");
    let mut changed_order = first.clone();
    changed_order["parameters"] = json!("temperature 0.2\nstop \"second\"\nstop \"first\"\n");
    let mut changed_literal = first.clone();
    changed_literal["modelfile"] = json!(first["modelfile"]
        .as_str()
        .unwrap()
        .replace("literal template", "changed template"));
    let mut changed_adapter = first.clone();
    changed_adapter["modelfile"] = json!(first["modelfile"]
        .as_str()
        .unwrap()
        .replace(&"c".repeat(64), &"d".repeat(64)));
    for changed in [
        changed_value,
        changed_order,
        changed_literal,
        changed_adapter,
    ] {
        let (provider, chats, host) = changing_model_stub(first.clone(), changed).await;
        let result = provider.generate(&request()).await;
        host.abort();
        assert!(matches!(result, Err(ProviderFailure::Rejected)));
        assert_eq!(chats.load(Ordering::SeqCst), 0);
    }
}
