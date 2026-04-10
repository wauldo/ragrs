//! Wauldo Verification API client

use crate::error::{RagrsError, Result};
use serde::{Deserialize, Serialize};

const DEFAULT_API_URL: &str = "https://api.wauldo.com";

/// Client for the Wauldo Verification API
pub struct VerifyClient {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

/// Result of a verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub verdict: String,
    pub confidence: f32,
    pub supported: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]
struct FactCheckRequest<'a> {
    claim: &'a str,
    source: &'a str,
}

#[derive(Deserialize)]
struct FactCheckResponse {
    verdict: String,
    confidence: f32,
    supported: bool,
    reason: Option<String>,
}

impl VerifyClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: DEFAULT_API_URL.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Verify a claim against source text
    pub async fn verify(&self, claim: &str, source: &str) -> Result<VerifyResult> {
        let url = format!("{}/v1/fact-check", self.base_url);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&FactCheckRequest { claim, source })
            .send()
            .await
            .map_err(|e| RagrsError::Verification(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(RagrsError::Verification(format!(
                "API returned {status}: {body}"
            )));
        }

        let data: FactCheckResponse = resp
            .json()
            .await
            .map_err(|e| RagrsError::Verification(format!("invalid response: {e}")))?;

        Ok(VerifyResult {
            verdict: data.verdict,
            confidence: data.confidence,
            supported: data.supported,
            reason: data.reason,
        })
    }
}
