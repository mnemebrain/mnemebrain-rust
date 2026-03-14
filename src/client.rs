use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{MnemeBrainError, Result};
use crate::models::*;
use crate::subclient::{
    AttackClient, GoalClient, PolicyClient, ReconsolidationClient, RevisionClient, SandboxClient,
};

const V4_PREFIX: &str = "/api/mneme";

// ── Builder ──

/// Builder for [`MnemeBrainClient`].
pub struct MnemeBrainClientBuilder {
    base_url: String,
    timeout: Duration,
    api_key: Option<String>,
    user_agent: Option<String>,
}

impl MnemeBrainClientBuilder {
    /// Start building a client pointed at `base_url`.
    ///
    /// Defaults: 30 s timeout, no authentication, no custom user-agent.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            timeout: Duration::from_secs(30),
            api_key: None,
            user_agent: None,
        }
    }

    /// Override the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a Bearer token for every request.
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Override the User-Agent header.
    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    /// Consume the builder and return a configured [`MnemeBrainClient`].
    pub fn build(self) -> MnemeBrainClient {
        let mut builder = Client::builder().timeout(self.timeout);

        let mut headers = HeaderMap::new();
        if let Some(ref key) = self.api_key {
            let mut auth_value =
                HeaderValue::from_str(&format!("Bearer {key}")).expect("invalid API key");
            auth_value.set_sensitive(true);
            headers.insert(AUTHORIZATION, auth_value);
        }
        if let Some(ref ua) = self.user_agent {
            builder = builder.user_agent(ua.as_str());
        }
        if !headers.is_empty() {
            builder = builder.default_headers(headers);
        }

        let http = builder.build().expect("failed to build HTTP client");
        let base = self.base_url.trim_end_matches('/').to_string();

        MnemeBrainClient {
            http: Arc::new(http),
            base_url: base,
            sandbox: std::sync::OnceLock::new(),
            goals: std::sync::OnceLock::new(),
            policies: std::sync::OnceLock::new(),
            revision: std::sync::OnceLock::new(),
            attacks: std::sync::OnceLock::new(),
            reconsolidation: std::sync::OnceLock::new(),
        }
    }
}

// ── Client ──

/// HTTP client for the MnemeBrain backend API.
pub struct MnemeBrainClient {
    http: Arc<Client>,
    base_url: String,
    sandbox: std::sync::OnceLock<SandboxClient>,
    goals: std::sync::OnceLock<GoalClient>,
    policies: std::sync::OnceLock<PolicyClient>,
    revision: std::sync::OnceLock<RevisionClient>,
    attacks: std::sync::OnceLock<AttackClient>,
    reconsolidation: std::sync::OnceLock<ReconsolidationClient>,
}

impl MnemeBrainClient {
    /// Create a new unauthenticated client pointing at the given MnemeBrain server.
    pub fn new(base_url: &str, timeout: Duration) -> Self {
        MnemeBrainClientBuilder::new(base_url)
            .timeout(timeout)
            .build()
    }

    /// Create a new client with a Bearer token for authentication.
    pub fn with_auth(base_url: &str, api_key: &str, timeout: Duration) -> Self {
        MnemeBrainClientBuilder::new(base_url)
            .timeout(timeout)
            .api_key(api_key)
            .build()
    }

    /// Convenience constructor with defaults (localhost:8000, 30 s timeout).
    pub fn localhost() -> Self {
        Self::new("http://localhost:8000", Duration::from_secs(30))
    }

    // ── Internal helpers ──

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn v4(&self, path: &str) -> String {
        format!("{}{}{}", self.base_url, V4_PREFIX, path)
    }

    async fn check_response(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(MnemeBrainError::Http {
                status,
                message: body,
            });
        }
        let body = resp.json::<Value>().await?;
        Ok(body)
    }

    // ── Health ──

    pub async fn health(&self) -> Result<HealthResponse> {
        let resp = self.http.get(self.url("/health")).send().await?;
        let body = self.check_response(resp).await?;
        Ok(serde_json::from_value(body)?)
    }

    // ── Core operations ──

    /// Assert a new belief with evidence. Uses [`BelieveRequest`] for all options.
    pub async fn believe(&self, request: &BelieveRequest) -> Result<BeliefResult> {
        let resp = self
            .http
            .post(self.url("/believe"))
            .json(request)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn explain(&self, claim: &str) -> Result<Option<ExplanationResult>> {
        let resp = self
            .http
            .get(self.url("/explain"))
            .query(&[("claim", claim)])
            .send()
            .await?;
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        let v = self.check_response(resp).await?;
        Ok(Some(serde_json::from_value(v)?))
    }

    /// Search beliefs. Uses [`SearchRequest`] for all options.
    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse> {
        // ConflictPolicy serializes to a lowercase string via serde.
        let cp_val = serde_json::to_value(&request.conflict_policy)?;
        let cp_str = cp_val
            .as_str()
            .expect("ConflictPolicy must serialize to a string")
            .to_string();

        let resp = self
            .http
            .get(self.url("/search"))
            .query(&[
                ("query", request.query.as_str()),
                ("limit", &request.limit.to_string()),
                ("alpha", &request.alpha.to_string()),
                ("conflict_policy", cp_str.as_str()),
            ])
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn retract(&self, evidence_id: &str) -> Result<Vec<BeliefResult>> {
        let body = json!({ "evidence_id": evidence_id });
        let resp = self
            .http
            .post(self.url("/retract"))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        // Handle both formats: bare array (lite) or {affected_beliefs: [...]} (full)
        if v.is_array() {
            Ok(serde_json::from_value(v)?)
        } else {
            let results: Vec<BeliefResult> = serde_json::from_value(
                v.get("affected_beliefs")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
            )?;
            Ok(results)
        }
    }

    pub async fn revise(&self, belief_id: &str, evidence: &EvidenceInput) -> Result<BeliefResult> {
        let body = json!({
            "belief_id": belief_id,
            "evidence": evidence,
        });
        let resp = self
            .http
            .post(self.url("/revise"))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn list_beliefs(&self, filters: &BeliefFilters) -> Result<BeliefListResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("min_confidence", filters.min_confidence.to_string()),
            ("max_confidence", filters.max_confidence.to_string()),
            ("limit", filters.limit.to_string()),
            ("offset", filters.offset.to_string()),
        ];
        if let Some(ref ts) = filters.truth_state {
            let v = serde_json::to_value(ts)?;
            if let Some(s) = v.as_str() {
                params.push(("truth_state", s.to_string()));
            }
        }
        if let Some(ref bt) = filters.belief_type {
            let v = serde_json::to_value(bt)?;
            if let Some(s) = v.as_str() {
                params.push(("belief_type", s.to_string()));
            }
        }
        if let Some(ref t) = filters.tag {
            params.push(("tag", t.clone()));
        }
        let resp = self
            .http
            .get(self.url("/beliefs"))
            .query(&params)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    // ── Working Memory Frame ──

    /// Open a frame using the full backend API shape. Uses [`FrameOpenRequest`] for all options.
    pub async fn frame_open(&self, request: &FrameOpenRequest) -> Result<FrameOpenResult> {
        let resp = self
            .http
            .post(self.url("/frame/open"))
            .json(request)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Open a frame using the lite API shape (query_id + goal_id + top_k).
    pub async fn frame_open_lite(&self, request: &LiteFrameOpenRequest) -> Result<FrameOpenResult> {
        let resp = self
            .http
            .post(self.url("/frame/open"))
            .json(request)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn frame_add(&self, frame_id: &str, claim: &str) -> Result<BeliefSnapshot> {
        let body = json!({ "claim": claim });
        let resp = self
            .http
            .post(self.url(&format!("/frame/{frame_id}/add")))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn frame_scratchpad(&self, frame_id: &str, key: &str, value: Value) -> Result<()> {
        let body = json!({ "key": key, "value": value });
        let resp = self
            .http
            .post(self.url(&format!("/frame/{frame_id}/scratchpad")))
            .json(&body)
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn frame_context(&self, frame_id: &str) -> Result<FrameContextResult> {
        let resp = self
            .http
            .get(self.url(&format!("/frame/{frame_id}/context")))
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn frame_commit(
        &self,
        frame_id: &str,
        new_beliefs: &[Value],
        revisions: &[Value],
    ) -> Result<FrameCommitResult> {
        let body = json!({
            "new_beliefs": new_beliefs,
            "revisions": revisions,
        });
        let resp = self
            .http
            .post(self.url(&format!("/frame/{frame_id}/commit")))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Close a frame without committing (DELETE /frame/{id}).
    pub async fn frame_close(&self, frame_id: &str) -> Result<()> {
        let resp = self
            .http
            .delete(self.url(&format!("/frame/{frame_id}")))
            .send()
            .await?;
        // 204 No Content has no body to parse
        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(MnemeBrainError::Http {
                status,
                message: body,
            });
        }
        Ok(())
    }

    // ── Utility endpoints ──

    pub async fn reset(&self) -> Result<()> {
        let resp = self.http.post(self.url("/reset")).send().await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn set_time_offset(&self, days: i64) -> Result<()> {
        let body = json!({ "days": days });
        let resp = self
            .http
            .post(self.url("/time-offset"))
            .json(&body)
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn consolidate(&self) -> Result<ConsolidateResult> {
        let resp = self.http.post(self.url("/consolidate")).send().await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_memory_tier(&self, belief_id: &str) -> Result<MemoryTierResult> {
        let resp = self
            .http
            .get(self.url(&format!("/memory-tier/{belief_id}")))
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn query_multihop(&self, query: &str) -> Result<MultihopResponse> {
        let resp = self
            .http
            .get(self.url("/multihop"))
            .query(&[("query", query)])
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    // ── Benchmark endpoints ──

    pub async fn benchmark_sandbox_fork(
        &self,
        scenario_label: &str,
    ) -> Result<BenchmarkSandboxResult> {
        let body = json!({ "scenario_label": scenario_label });
        let resp = self
            .http
            .post(self.v4("/benchmark/sandbox/fork"))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn benchmark_sandbox_assume(
        &self,
        sandbox_id: &str,
        belief_id: &str,
        truth_state: &str,
    ) -> Result<BenchmarkSandboxResult> {
        let body = json!({
            "belief_id": belief_id,
            "truth_state": truth_state,
        });
        let resp = self
            .http
            .post(self.v4(&format!("/benchmark/sandbox/{sandbox_id}/assume")))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn benchmark_sandbox_resolve(
        &self,
        sandbox_id: &str,
        belief_id: &str,
    ) -> Result<BenchmarkSandboxResult> {
        let resp = self
            .http
            .get(self.v4(&format!(
                "/benchmark/sandbox/{sandbox_id}/resolve/{belief_id}"
            )))
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn benchmark_sandbox_discard(&self, sandbox_id: &str) -> Result<()> {
        let resp = self
            .http
            .post(self.v4(&format!("/benchmark/sandbox/{sandbox_id}/discard")))
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn benchmark_attack(
        &self,
        attacker_id: &str,
        target_id: &str,
        attack_type: &str,
        weight: f64,
    ) -> Result<BenchmarkAttackResult> {
        let body = json!({
            "attacker_id": attacker_id,
            "target_id": target_id,
            "attack_type": attack_type,
            "weight": weight,
        });
        let resp = self
            .http
            .post(self.v4("/benchmark/attack"))
            .json(&body)
            .send()
            .await?;
        let v = self.check_response(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    // ── Sub-clients ──

    pub fn sandbox(&self) -> &SandboxClient {
        self.sandbox
            .get_or_init(|| SandboxClient::new(self.http.clone(), self.base_url.clone()))
    }

    pub fn goals(&self) -> &GoalClient {
        self.goals
            .get_or_init(|| GoalClient::new(self.http.clone(), self.base_url.clone()))
    }

    pub fn policies(&self) -> &PolicyClient {
        self.policies
            .get_or_init(|| PolicyClient::new(self.http.clone(), self.base_url.clone()))
    }

    pub fn revision(&self) -> &RevisionClient {
        self.revision
            .get_or_init(|| RevisionClient::new(self.http.clone(), self.base_url.clone()))
    }

    pub fn attacks(&self) -> &AttackClient {
        self.attacks
            .get_or_init(|| AttackClient::new(self.http.clone(), self.base_url.clone()))
    }

    pub fn reconsolidation(&self) -> &ReconsolidationClient {
        self.reconsolidation
            .get_or_init(|| ReconsolidationClient::new(self.http.clone(), self.base_url.clone()))
    }
}

impl Default for MnemeBrainClient {
    fn default() -> Self {
        Self::localhost()
    }
}
