use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{MnemeBrainError, Result};
use crate::models::*;

const V4_PREFIX: &str = "/api/mneme";

pub struct SandboxClient {
    http: Arc<Client>,
    base_url: String,
}

impl SandboxClient {
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

    pub async fn fork(
        &self,
        frame_id: Option<&str>,
        scenario_label: &str,
        ttl_seconds: u32,
    ) -> Result<SandboxResult> {
        let mut body = json!({
            "scenario_label": scenario_label,
            "ttl_seconds": ttl_seconds,
        });
        if let Some(fid) = frame_id {
            body["frame_id"] = Value::String(fid.into());
        }
        let resp = self
            .http
            .post(self.v4("/sandbox/fork"))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn quick(&self, frame_id: Option<&str>) -> Result<SandboxResult> {
        let mut body = json!({});
        if let Some(fid) = frame_id {
            body["frame_id"] = Value::String(fid.into());
        }
        let resp = self
            .http
            .post(self.v4("/sandbox/quick"))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_context(&self, sandbox_id: &str) -> Result<SandboxContextResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/sandbox/{sandbox_id}/context")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn assume(
        &self,
        sandbox_id: &str,
        belief_id: &str,
        truth_state: TruthState,
    ) -> Result<()> {
        let body = json!({
            "belief_id": belief_id,
            "truth_state": truth_state,
        });
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/assume")))
            .json(&body)
            .send()
            .await?;
        self.check(resp).await?;
        Ok(())
    }

    pub async fn retract(&self, sandbox_id: &str, evidence_id: &str) -> Result<()> {
        let body = json!({ "evidence_id": evidence_id });
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/retract")))
            .json(&body)
            .send()
            .await?;
        self.check(resp).await?;
        Ok(())
    }

    pub async fn believe(
        &self,
        sandbox_id: &str,
        claim: &str,
        belief_type: BeliefType,
    ) -> Result<Value> {
        let body = json!({
            "claim": claim,
            "belief_type": belief_type,
        });
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/believe")))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(v)
    }

    pub async fn revise(
        &self,
        sandbox_id: &str,
        belief_id: &str,
        evidence: &EvidenceInput,
    ) -> Result<()> {
        let body = json!({
            "belief_id": belief_id,
            "source_ref": evidence.source_ref,
            "content": evidence.content,
            "polarity": evidence.polarity,
            "weight": evidence.weight,
            "reliability": evidence.reliability,
        });
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/revise")))
            .json(&body)
            .send()
            .await?;
        self.check(resp).await?;
        Ok(())
    }

    pub async fn attack(
        &self,
        sandbox_id: &str,
        attacker_belief_id: &str,
        target_belief_id: &str,
        attack_type: AttackType,
        weight: f64,
    ) -> Result<Value> {
        let body = json!({
            "attacker_belief_id": attacker_belief_id,
            "target_belief_id": target_belief_id,
            "attack_type": attack_type,
            "weight": weight,
        });
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/attack")))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(v)
    }

    pub async fn diff(&self, sandbox_id: &str) -> Result<SandboxDiffResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/sandbox/{sandbox_id}/diff")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn commit(
        &self,
        sandbox_id: &str,
        commit_mode: CommitMode,
        selected_ids: Option<&[String]>,
    ) -> Result<SandboxCommitResult> {
        let mut body = json!({ "commit_mode": commit_mode });
        if let Some(ids) = selected_ids {
            body["selected_ids"] = json!(ids);
        }
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/commit")))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn discard(&self, sandbox_id: &str) -> Result<()> {
        let resp = self
            .http
            .post(self.v4(&format!("/sandbox/{sandbox_id}/discard")))
            .send()
            .await?;
        self.check(resp).await?;
        Ok(())
    }

    pub async fn explain(&self, sandbox_id: &str, belief_id: &str) -> Result<SandboxExplainResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/sandbox/{sandbox_id}/explain/{belief_id}")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn evaluate_goal(
        &self,
        sandbox_id: &str,
        goal_id: &str,
    ) -> Result<GoalEvaluationResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/sandbox/{sandbox_id}/goals/{goal_id}/evaluate")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }
}
