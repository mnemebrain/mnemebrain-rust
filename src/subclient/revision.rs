use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{MnemeBrainError, Result};
use crate::models::*;

const V4_PREFIX: &str = "/api/mneme";

pub struct RevisionClient {
    http: Arc<Client>,
    base_url: String,
}

impl RevisionClient {
    pub(crate) fn new(http: Arc<Client>, base_url: String) -> Self {
        Self { http, base_url }
    }

    fn v4(&self, path: &str) -> String {
        format!("{}{}{}", self.base_url, V4_PREFIX, path)
    }

    async fn check(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(MnemeBrainError::Http {
                status,
                message: body,
            });
        }
        Ok(resp.json::<Value>().await?)
    }

    pub async fn set_policy(
        &self,
        policy_name: &str,
        max_retraction_depth: Option<i64>,
        max_retractions: Option<i64>,
    ) -> Result<RevisionPolicyResult> {
        let mut body = json!({ "policy_name": policy_name });
        if let Some(d) = max_retraction_depth {
            body["max_retraction_depth"] = json!(d);
        }
        if let Some(r) = max_retractions {
            body["max_retractions"] = json!(r);
        }
        let resp = self
            .http
            .post(self.v4("/revision/policy"))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_policy(&self) -> Result<RevisionPolicyResult> {
        let resp = self.http.get(self.v4("/revision/policy")).send().await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn list_audit(&self) -> Result<Vec<RevisionAuditEntry>> {
        let resp = self.http.get(self.v4("/revision/audit")).send().await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn revise(
        &self,
        incoming_belief_id: &str,
        conflicting_evidence: &[RevisionEvidenceItem],
        incoming_evidence: &[RevisionEvidenceItem],
        agent_id: &str,
    ) -> Result<RevisionResult> {
        let body = json!({
            "incoming_belief_id": incoming_belief_id,
            "conflicting_evidence": conflicting_evidence,
            "incoming_evidence": incoming_evidence,
            "agent_id": agent_id,
        });
        let resp = self
            .http
            .post(self.v4("/revision/revise"))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }
}
