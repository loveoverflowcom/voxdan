//! Imperative local-only provider boundary. Responses remain untrusted until domain admission.
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use cantos_api::{AdaptationConfig, AdaptationCost, AdaptationProviderMetadata, AdaptationUsage};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{CONTRACT_VERSION, MAX_OUTPUT_BYTES, MAX_REQUEST_BYTES, PROMPT_VERSION};

#[derive(Clone, Debug)]
pub struct ProviderRequest {
    pub system: String,
    pub prompt: String,
}

#[derive(Clone, Debug)]
pub struct ProviderOutput {
    pub bytes: Vec<u8>,
    pub usage: Option<AdaptationUsage>,
    pub cost: Option<AdaptationCost>,
    /// Parsed rejected output can still carry actual provider-reported usage.
    pub problem: Option<ProviderFailure>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderFailure {
    Unavailable,
    Rejected,
    MalformedResponse,
    OutputTooLarge,
    Ambiguous,
}

impl ProviderFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "provider_unavailable",
            Self::Rejected => "provider_rejected",
            Self::MalformedResponse => "provider_malformed_response",
            Self::OutputTooLarge => "provider_output_too_large",
            Self::Ambiguous => "provider_outcome_unknown",
        }
    }

    pub fn outcome_unknown(self) -> bool {
        matches!(self, Self::Ambiguous)
    }
}

pub type ProviderFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ProviderOutput, ProviderFailure>> + Send + 'a>>;

/// A real local adapter and injected test adapters share this narrow capability port.
pub trait AdaptationProvider: Send + Sync {
    fn metadata(&self) -> AdaptationProviderMetadata;
    fn generate<'a>(&'a self, request: &'a ProviderRequest) -> ProviderFuture<'a>;
}

pub struct OllamaProvider {
    client: reqwest::Client,
    metadata: AdaptationProviderMetadata,
}

impl OllamaProvider {
    /// Only numeric loopback HTTP is admitted. No DNS, proxy, redirect, credentials or model pull.
    pub fn new(
        endpoint: &str,
        model: &str,
        config: AdaptationConfig,
    ) -> Result<Self, ProviderFailure> {
        let url = reqwest::Url::parse(endpoint).map_err(|_| ProviderFailure::Rejected)?;
        if url.scheme() != "http"
            || !matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "::1"))
            || url.port().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !matches!(url.path(), "" | "/")
            || model.is_empty()
            || model.len() > 128
            || !model
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-:/".contains(&byte))
            || config.temperature_milli > 1000
            || !(1024..=16384).contains(&config.num_context)
            || !(128..=8192).contains(&config.num_predict)
            || !(1..=180).contains(&config.timeout_seconds)
        {
            return Err(ProviderFailure::Rejected);
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(u64::from(config.timeout_seconds)))
            .build()
            .map_err(|_| ProviderFailure::Unavailable)?;
        Ok(Self {
            client,
            metadata: AdaptationProviderMetadata {
                provider: "ollama-local".into(),
                endpoint: endpoint.trim_end_matches('/').into(),
                model: model.into(),
                local_model_digest: None,
                prompt_version: PROMPT_VERSION.into(),
                contract_version: CONTRACT_VERSION.into(),
                config,
            },
        })
    }

    /// Safe discovery sends no source text. Unknown/older runtimes and cloud aliases fail closed.
    pub async fn connect(
        endpoint: &str,
        model: &str,
        config: AdaptationConfig,
    ) -> Result<Self, ProviderFailure> {
        let mut provider = Self::new(endpoint, model, config)?;
        provider.metadata.local_model_digest = Some(provider.verify_local_model().await?);
        Ok(provider)
    }

    async fn probe_json(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<serde_json::Value, ProviderFailure> {
        const MAX_PROBE_BYTES: usize = 256 * 1024;
        let mut response = request
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map_err(|_| ProviderFailure::Unavailable)?;
        if !response.status().is_success() {
            return Err(ProviderFailure::Rejected);
        }
        if response
            .content_length()
            .is_some_and(|size| size > MAX_PROBE_BYTES as u64)
        {
            return Err(ProviderFailure::Rejected);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| ProviderFailure::Unavailable)?
        {
            if chunk.len() > MAX_PROBE_BYTES.saturating_sub(bytes.len()) {
                return Err(ProviderFailure::Rejected);
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|_| ProviderFailure::Rejected)
    }

    async fn verify_local_model(&self) -> Result<String, ProviderFailure> {
        // Numeric loopback is transport locality, not inference locality: Ollama can proxy cloud.
        let status = self
            .probe_json(
                self.client
                    .get(format!("{}/api/status", self.metadata.endpoint)),
            )
            .await?;
        if status["cloud"]["disabled"] != serde_json::Value::Bool(true) {
            return Err(ProviderFailure::Rejected);
        }
        let mut info = self
            .probe_json(
                self.client
                    .post(format!("{}/api/show", self.metadata.endpoint))
                    .json(&json!({"model":self.metadata.model,"verbose":false})),
            )
            .await?;
        for field in ["remote_model", "remote_host"] {
            if info
                .get(field)
                .is_some_and(|value| value.as_str() != Some(""))
            {
                return Err(ProviderFailure::Rejected);
            }
        }
        if info["details"]["format"] != "gguf"
            || info["details"]["parameter_size"]
                .as_str()
                .is_none_or(str::is_empty)
            || info["model_info"]["general.architecture"]
                .as_str()
                .is_none_or(str::is_empty)
        {
            return Err(ProviderFailure::Rejected);
        }
        let (modelfile, weight) = canonical_modelfile(
            info["modelfile"]
                .as_str()
                .ok_or(ProviderFailure::Rejected)?,
        )?;
        info["modelfile"] = modelfile;
        if let Some(parameters) = info.get("parameters") {
            info["parameters"] = serde_json::to_value(parameter_groups(
                parameters.as_str().ok_or(ProviderFailure::Rejected)?,
            )?)
            .map_err(|_| ProviderFailure::Rejected)?;
        }
        let mut hash = Sha256::new();
        hash.update(b"cantos/ollama-local-model/s1\n");
        hash.update(weight.as_bytes());
        hash.update(b"\n");
        hash.update(serde_json::to_vec(&info).map_err(|_| ProviderFailure::Rejected)?);
        Ok(format!("ollama-s1:sha256:{:x}", hash.finalize()))
    }

    async fn dispatch(&self, request: &ProviderRequest) -> Result<ProviderOutput, ProviderFailure> {
        let config = &self.metadata.config;
        super::validate_request_budget(request, config).map_err(|_| ProviderFailure::Rejected)?;
        let pinned = self
            .metadata
            .local_model_digest
            .as_deref()
            .ok_or(ProviderFailure::Rejected)?;
        if self.verify_local_model().await? != pinned {
            return Err(ProviderFailure::Rejected);
        }
        let body = json!({
            "model": self.metadata.model,
            "stream": false,
            "format": serde_json::from_str::<serde_json::Value>(super::OUTPUT_SCHEMA)
                .map_err(|_| ProviderFailure::Rejected)?,
            "keep_alive": 0,
            "messages": [
                {"role": "system", "content": request.system},
                {"role": "user", "content": request.prompt}
            ],
            "options": {
                "temperature": f64::from(config.temperature_milli) / 1000.0,
                "seed": config.seed,
                "num_ctx": config.num_context,
                "num_predict": config.num_predict
            }
        });
        let request_body = serde_json::to_vec(&body).map_err(|_| ProviderFailure::Rejected)?;
        if request_body.len() > MAX_REQUEST_BYTES {
            return Err(ProviderFailure::Rejected);
        }
        let mut response = self
            .client
            .post(format!("{}/api/chat", self.metadata.endpoint))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request_body)
            .send()
            .await
            .map_err(classify_transport)?;
        // A redirect is an explicit rejection; it never forwards manuscript content.
        if !response.status().is_success() {
            return Err(ProviderFailure::Rejected);
        }
        // JSON escaping may expand a content string; the envelope has a separate finite cap.
        const MAX_RESPONSE_BYTES: usize = MAX_OUTPUT_BYTES * 6 + 4096;
        if response
            .content_length()
            .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
        {
            return Err(ProviderFailure::OutputTooLarge);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(classify_transport)? {
            if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
                return Err(ProviderFailure::OutputTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        let reply: OllamaReply =
            serde_json::from_slice(&bytes).map_err(|_| ProviderFailure::MalformedResponse)?;
        let usage = if reply.prompt_eval_count.is_some() || reply.eval_count.is_some() {
            Some(AdaptationUsage {
                input_tokens: reply.prompt_eval_count,
                output_tokens: reply.eval_count,
            })
        } else {
            None
        };
        let output = reply.message.content.into_bytes();
        let problem = if output.len() > MAX_OUTPUT_BYTES {
            Some(ProviderFailure::OutputTooLarge)
        } else if !reply.done
            || reply.message.role != "assistant"
            || reply.model != self.metadata.model
            || reply
                .done_reason
                .as_deref()
                .is_some_and(|reason| reason != "stop")
        {
            Some(ProviderFailure::MalformedResponse)
        } else {
            None
        };
        Ok(ProviderOutput {
            bytes: if problem.is_none() { output } else { vec![] },
            usage,
            // Ollama does not report monetary cost; unavailable is never invented as zero.
            cost: None,
            problem,
        })
    }
}

fn classify_transport(error: reqwest::Error) -> ProviderFailure {
    if error.is_connect() {
        ProviderFailure::Unavailable
    } else {
        // After dispatch, timeout, body truncation or socket loss may conceal completed work.
        ProviderFailure::Ambiguous
    }
}

fn add_parameter(
    groups: &mut BTreeMap<String, Vec<String>>,
    line: &str,
) -> Result<(), ProviderFailure> {
    let (key, value) = line
        .trim()
        .split_once(char::is_whitespace)
        .ok_or(ProviderFailure::Rejected)?;
    if key.is_empty()
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || value.trim().is_empty()
    {
        return Err(ProviderFailure::Rejected);
    }
    groups
        .entry(key.into())
        .or_default()
        .push(value.trim().into());
    Ok(())
}

fn parameter_groups(input: &str) -> Result<BTreeMap<String, Vec<String>>, ProviderFailure> {
    let mut groups = BTreeMap::new();
    for line in input.lines().filter(|line| !line.trim().is_empty()) {
        add_parameter(&mut groups, line)?;
    }
    Ok(groups)
}

/// Ollama renders a Go map in arbitrary key order. Preserve repeated values within each key,
/// all other directives, and literal quoted TEMPLATE/SYSTEM/LICENSE/MESSAGE content.
fn canonical_modelfile(input: &str) -> Result<(serde_json::Value, String), ProviderFailure> {
    let mut quoted: Option<&str> = None;
    let mut lines = Vec::new();
    let mut parameters = BTreeMap::new();
    let mut weights = Vec::new();
    for line in input.lines() {
        let directive = line.trim_start();
        if let Some(delimiter) = quoted {
            // A '#' within a quoted value is content, including on its closing line.
            if line.trim_end().ends_with(delimiter) {
                quoted = None;
            }
            lines.push(line);
            continue;
        }
        if !directive.starts_with('#') {
            if let Some(parameter) = directive.strip_prefix("PARAMETER ") {
                let (_, value) = parameter
                    .split_once(char::is_whitespace)
                    .ok_or(ProviderFailure::Rejected)?;
                // Newline-containing parameters have ambiguous rendered grouping: fail closed.
                if value.trim_start().starts_with('"')
                    && quote_delimiter(value.trim_start()).is_some()
                {
                    return Err(ProviderFailure::Rejected);
                }
                add_parameter(&mut parameters, parameter)?;
                continue;
            }
            if let Some(path) = directive.strip_prefix("FROM ") {
                let path = path.trim();
                let (_, digest) = path
                    .rsplit_once("sha256-")
                    .ok_or(ProviderFailure::Rejected)?;
                if !path.starts_with('/')
                    || digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err(ProviderFailure::Rejected);
                }
                weights.push(digest.to_owned());
            }
            if let Some((command, arguments)) = directive.split_once(' ') {
                let value = if command == "MESSAGE" {
                    arguments
                        .split_once(' ')
                        .map(|(_, value)| value)
                        .ok_or(ProviderFailure::Rejected)?
                } else {
                    arguments
                };
                quoted = quote_delimiter(value);
            }
        }
        lines.push(line);
    }
    if quoted.is_some() || weights.len() != 1 {
        return Err(ProviderFailure::Rejected);
    }
    let weight = weights.pop().ok_or(ProviderFailure::Rejected)?;
    Ok((json!({"lines":lines,"parameters":parameters}), weight))
}

/// Match Ollama's generated Command.String quote wrapper; preserve its body exactly.
fn quote_delimiter(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value.starts_with("\"\"\"") {
        (value.len() < 6 || !value.ends_with("\"\"\"")).then_some("\"\"\"")
    } else if value.starts_with('"') {
        (value.len() < 2 || !value.ends_with('"')).then_some("\"")
    } else {
        None
    }
}

impl AdaptationProvider for OllamaProvider {
    fn metadata(&self) -> AdaptationProviderMetadata {
        self.metadata.clone()
    }

    fn generate<'a>(&'a self, request: &'a ProviderRequest) -> ProviderFuture<'a> {
        Box::pin(self.dispatch(request))
    }
}

#[derive(Deserialize)]
struct OllamaReply {
    model: String,
    message: OllamaMessage,
    done: bool,
    done_reason: Option<String>,
    prompt_eval_count: Option<u64>,
    eval_count: Option<u64>,
}

#[derive(Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}
