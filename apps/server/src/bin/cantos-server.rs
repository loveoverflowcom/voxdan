use cantos_api::AdaptationConfig;
use cantos_server::adaptation::provider::OllamaProvider;
use cantos_server::{
    http::{router, AppState},
    postgres::{local_config, Store},
};
use std::{env, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("CANTOS_ENV").ok().as_deref() != Some("development") {
        return Err("this host supports CANTOS_ENV=development only".into());
    }
    let address: SocketAddr = env::var("CANTOS_BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("development host must bind loopback".into());
    }
    let config =
        local_config(&env::var("DATABASE_URL")?).map_err(|_| "invalid local PostgreSQL config")?;
    let mut store = Store::new(config).map_err(|_| "database pool unavailable")?;
    if let Ok(model) = env::var("CANTOS_ADAPTATION_MODEL") {
        let endpoint = env::var("CANTOS_ADAPTATION_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:11434".into());
        let provider = OllamaProvider::connect(
            &endpoint,
            &model,
            AdaptationConfig {
                temperature_milli: 200,
                seed: 0,
                num_context: 8192,
                num_predict: 4096,
                timeout_seconds: 120,
            },
        )
        .await
        .map_err(|_| "local adaptation runtime could not verify cloud-disabled execution and pinned local model")?;
        store = store.with_adaptation_provider(Arc::new(provider));
    }
    let state = AppState {
        store,
        web_origin: env::var("CANTOS_WEB_ORIGIN").unwrap_or_else(|_| format!("http://{address}")),
    };
    let dist = env::var("CANTOS_WEB_DIST").unwrap_or_else(|_| "apps/web/dist".into());
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("Cantos Studio development host: http://{address}");
    axum::serve(listener, router(state, &dist))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
