//! Same-origin HTTP transport. Session credential becomes an HttpOnly cookie.
use cantos_api::{
    ApiError, ErrorCode, RevisionResponse, SaveRevisionRequest, SessionRequest, SessionResponse,
};
use gloo_net::http::{Request, Response};

#[derive(Clone, Copy)]
pub struct StudioHttp;

#[derive(Clone, Copy)]
pub struct StudioContext {
    pub api: StudioHttp,
}

fn unavailable() -> ApiError {
    ApiError {
        code: ErrorCode::Unavailable,
        current_revision: None,
        issues: vec![],
    }
}

async fn decode<T: serde::de::DeserializeOwned>(response: Response) -> Result<T, ApiError> {
    if response.ok() {
        response.json().await.map_err(|_| unavailable())
    } else {
        Err(response.json().await.unwrap_or_else(|_| unavailable()))
    }
}

impl StudioHttp {
    pub async fn login(&self, token: String) -> Result<SessionResponse, ApiError> {
        let request = Request::post("/api/v1/session")
            .json(&SessionRequest { token })
            .map_err(|_| unavailable())?;
        decode(request.send().await.map_err(|_| unavailable())?).await
    }
    pub async fn read(&self, script: &str) -> Result<RevisionResponse, ApiError> {
        // Paths only accept a canonical UUID; avoid interpreting arbitrary user input as a URL.
        if script.len() != 36 || !script.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
            return Err(ApiError {
                code: ErrorCode::InvalidRequest,
                current_revision: None,
                issues: vec![],
            });
        }
        decode(
            Request::get(&format!("/api/v1/scripts/{script}/head"))
                .send()
                .await
                .map_err(|_| unavailable())?,
        )
        .await
    }
    pub async fn save(
        &self,
        script: &str,
        request: &SaveRevisionRequest,
    ) -> Result<RevisionResponse, ApiError> {
        if script.len() != 36 || !script.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
            return Err(ApiError {
                code: ErrorCode::InvalidRequest,
                current_revision: None,
                issues: vec![],
            });
        }
        let request = Request::post(&format!("/api/v1/scripts/{script}/revisions"))
            .json(request)
            .map_err(|_| unavailable())?;
        decode(request.send().await.map_err(|_| unavailable())?).await
    }
}
