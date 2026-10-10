//! Same-origin HTTP transport. Session credential becomes an HttpOnly cookie.
use cantos_api::{
    AcceptAdaptationRequest, AdaptationProviderResponse, AdaptationRunResponse, ApiError,
    CancelAdaptationRequest, ErrorCode, ImportRequest, ImportResponse, RevisionResponse,
    SaveRevisionRequest, SessionRequest, SessionResponse, StartAdaptationRequest,
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
    pub async fn adaptation_provider(&self) -> Result<AdaptationProviderResponse, ApiError> {
        decode(
            Request::get("/api/v1/adaptations/provider")
                .send()
                .await
                .map_err(|_| unavailable())?,
        )
        .await
    }

    pub async fn start_adaptation(
        &self,
        request: &StartAdaptationRequest,
    ) -> Result<AdaptationRunResponse, ApiError> {
        let request = Request::post("/api/v1/adaptations")
            .json(request)
            .map_err(|_| unavailable())?;
        decode(request.send().await.map_err(|_| unavailable())?).await
    }

    pub async fn read_adaptation(&self, id: &str) -> Result<AdaptationRunResponse, ApiError> {
        decode(
            Request::get(&adaptation_path(id)?)
                .send()
                .await
                .map_err(|_| unavailable())?,
        )
        .await
    }

    pub async fn accept_adaptation(
        &self,
        id: &str,
        request: &AcceptAdaptationRequest,
    ) -> Result<RevisionResponse, ApiError> {
        let request = Request::post(&format!("{}/accept", adaptation_path(id)?))
            .json(request)
            .map_err(|_| unavailable())?;
        decode(request.send().await.map_err(|_| unavailable())?).await
    }

    pub async fn cancel_adaptation(
        &self,
        id: &str,
        request: &CancelAdaptationRequest,
    ) -> Result<AdaptationRunResponse, ApiError> {
        let request = Request::post(&format!("{}/cancel", adaptation_path(id)?))
            .json(request)
            .map_err(|_| unavailable())?;
        decode(request.send().await.map_err(|_| unavailable())?).await
    }

    pub async fn read_revision(
        &self,
        script: &str,
        revision: u64,
    ) -> Result<RevisionResponse, ApiError> {
        if !crate::adaptation::is_run_id(script) || revision == 0 {
            return Err(ApiError {
                code: ErrorCode::InvalidRequest,
                current_revision: None,
                issues: vec![],
            });
        }
        decode(
            Request::get(&format!("/api/v1/scripts/{script}/revisions/{revision}"))
                .send()
                .await
                .map_err(|_| unavailable())?,
        )
        .await
    }
    pub async fn import(&self, request: &ImportRequest) -> Result<ImportResponse, ApiError> {
        let request = Request::post("/api/v1/imports")
            .json(request)
            .map_err(|_| unavailable())?;
        decode(request.send().await.map_err(|_| unavailable())?).await
    }

    pub async fn read_import(&self, id: &str) -> Result<ImportResponse, ApiError> {
        let path = import_path(id)?;
        decode(
            Request::get(&path)
                .send()
                .await
                .map_err(|_| unavailable())?,
        )
        .await
    }

    pub fn original_import_path(&self, id: &str) -> Result<String, ApiError> {
        Ok(format!("{}/original", import_path(id)?))
    }
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

fn adaptation_path(id: &str) -> Result<String, ApiError> {
    if !crate::adaptation::is_run_id(id) {
        return Err(ApiError {
            code: ErrorCode::InvalidRequest,
            current_revision: None,
            issues: vec![],
        });
    }
    Ok(format!("/api/v1/adaptations/{id}"))
}

fn import_path(id: &str) -> Result<String, ApiError> {
    if !crate::import::is_import_source_id(id) {
        return Err(ApiError {
            code: ErrorCode::InvalidRequest,
            current_revision: None,
            issues: vec![],
        });
    }
    Ok(format!("/api/v1/imports/{id}"))
}
