use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{MnemeBrainError, Result};
use crate::models::*;

const V4_PREFIX: &str = "/api/mneme";

pub struct AttackClient {
    http: Arc<Client>,
    base_url: String,
}

impl AttackClient {
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

    pub async fn create(
        &self,
        belief_id: &str,
        target_belief_id: &str,
        attack_type: AttackType,
        weight: f64,
    ) -> Result<AttackEdgeResult> {
        let body = json!({
            "target_belief_id": target_belief_id,
            "attack_type": attack_type,
            "weight": weight,
        });
        let resp = self
            .http
            .post(self.v4(&format!("/attacks/{belief_id}")))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn list(&self, belief_id: &str) -> Result<Vec<AttackEdgeResult>> {
        let resp = self
            .http
            .get(self.v4(&format!("/attacks/{belief_id}")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_chain(
        &self,
        belief_id: &str,
        max_depth: u32,
    ) -> Result<Vec<Vec<AttackEdgeResult>>> {
        let resp = self
            .http
            .get(self.v4(&format!("/attacks/{belief_id}/chain")))
            .query(&[("max_depth", max_depth.to_string())])
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn deactivate(&self, edge_id: &str) -> Result<()> {
        let resp = self
            .http
            .post(self.v4(&format!("/attacks/{edge_id}/deactivate")))
            .send()
            .await?;
        self.check(resp).await?;
        Ok(())
    }
}
