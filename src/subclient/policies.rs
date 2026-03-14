use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{MnemeBrainError, Result};
use crate::models::*;

const V4_PREFIX: &str = "/api/mneme";

pub struct PolicyClient {
    http: Arc<Client>,
    base_url: String,
}

impl PolicyClient {
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
        name: &str,
        steps: &[Value],
        description: &str,
        applicability: Option<&serde_json::Map<String, Value>>,
    ) -> Result<PolicyResult> {
        let mut body = json!({
            "name": name,
            "steps": steps,
            "description": description,
        });
        if let Some(app) = applicability {
            body["applicability"] = Value::Object(app.clone());
        }
        let resp = self
            .http
            .post(self.v4("/policies"))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn list(&self) -> Result<Vec<PolicyResult>> {
        let resp = self.http.get(self.v4("/policies")).send().await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get(&self, policy_id: &str) -> Result<PolicyResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/policies/{policy_id}")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get_history(&self, policy_id: &str) -> Result<Vec<PolicyResult>> {
        let resp = self
            .http
            .get(self.v4(&format!("/policies/{policy_id}/history")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn update_status(
        &self,
        policy_id: &str,
        status: PolicyStatus,
    ) -> Result<PolicyResult> {
        let body = json!({ "status": status });
        let resp = self
            .http
            .patch(self.v4(&format!("/policies/{policy_id}")))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }
}
