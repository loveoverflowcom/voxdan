//! Private, revision-pinned adaptation transport. Rights authorization is a recorded claim.
use serde::{Deserialize, Serialize};

use crate::{FieldIssue, RevisionResponse};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StartAdaptationRequest {
    pub operation_id: String,
    pub source_id: String,
    pub script_id: String,
    pub expected_revision: u64,
    /// The displayed, authorized destination and generation configuration must still match.
    pub expected_provider: AdaptationProviderMetadata,
    /// The creator explicitly authorizes processing this source by the configured local runtime.
    /// This assertion never establishes production or publication eligibility.
    pub rights_authorization: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AcceptAdaptationRequest {
    pub operation_id: String,
    pub expected_revision: u64,
    pub script_json: String,
    pub reviewed_findings: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CancelAdaptationRequest {
    pub operation_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationStatus {
    Queued,
    Running,
    Succeeded,
    InvalidOutput,
    Failed,
    Ambiguous,
    Cancelled,
    Accepted,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationConfig {
    pub temperature_milli: u16,
    pub seed: u32,
    pub num_context: u32,
    pub num_predict: u32,
    pub timeout_seconds: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationProviderMetadata {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    /// Verified local GGUF/model configuration fingerprint; absent only for injected test ports.
    pub local_model_digest: Option<String>,
    pub prompt_version: String,
    pub contract_version: String,
    pub config: AdaptationConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationProviderResponse {
    pub provider: Option<AdaptationProviderMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationCost {
    /// A provider-reported amount; absent records mean unavailable, never zero.
    pub currency: AdaptationCurrency,
    pub amount_minor: u64,
    pub basis: AdaptationCostBasis,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum AdaptationCurrency {
    /// ISO 4217 USD: two minor-unit decimal places.
    USD,
    /// ISO 4217 VND: zero minor-unit decimal places.
    VND,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationCostBasis {
    ProviderReported,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationProblem {
    pub code: String,
    pub issues: Vec<FieldIssue>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationFindingCode {
    SourceWarning,
    UncoveredSource,
    OmittedSource,
    TextChanged,
    UnresolvedSpeaker,
    NewSpeaker,
    PossibleNarrationConfusion,
    CueLikeMarkup,
    UnsupportedPerformanceControl,
    ProviderReviewNote,
    SourceCoverageRequiresReview,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationFinding {
    pub code: AdaptationFindingCode,
    pub block: Option<u32>,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationCoverageDisposition {
    Represented,
    Omitted,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationCoverage {
    pub block: u32,
    pub dialogue_ids: Vec<String>,
    pub disposition: AdaptationCoverageDisposition,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationProposal {
    /// Canonical complete Script IR after structural and semantic admission.
    pub script_json: String,
    pub findings: Vec<AdaptationFinding>,
    pub coverage: Vec<AdaptationCoverage>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdaptationRunResponse {
    pub id: String,
    pub created_by: String,
    pub recorded_at: String,
    pub updated_at: String,
    pub request: StartAdaptationRequest,
    pub source_sha256: String,
    pub extractor_version: String,
    pub input_content_digest: Option<String>,
    pub input_export_digest: Option<String>,
    pub generation_record_id: String,
    pub rights_record_id: String,
    pub provider: AdaptationProviderMetadata,
    pub status: AdaptationStatus,
    pub proposal: Option<AdaptationProposal>,
    pub problem: Option<AdaptationProblem>,
    pub usage: Option<AdaptationUsage>,
    pub cost: Option<AdaptationCost>,
    pub accepted_revision: Option<RevisionResponse>,
}
