use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{MnemeBrainError, Result};
use crate::models::*;

const V4_PREFIX: &str = "/api/mneme";

pub struct GoalClient {
    http: Arc<Client>,
    base_url: String,
}

impl GoalClient {
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
        goal: &str,
        owner: &str,
        priority: f64,
        success_criteria: Option<&serde_json::Map<String, Value>>,
        deadline: Option<&str>,
    ) -> Result<GoalResult> {
        let mut body = json!({
            "goal": goal,
            "owner": owner,
            "priority": priority,
        });
        if let Some(sc) = success_criteria {
            body["success_criteria"] = Value::Object(sc.clone());
        }
        if let Some(dl) = deadline {
            body["deadline"] = Value::String(dl.into());
        }
        let resp = self.http.post(self.v4("/goals")).json(&body).send().await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn list(&self) -> Result<Vec<GoalResult>> {
        let resp = self.http.get(self.v4("/goals")).send().await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn get(&self, goal_id: &str) -> Result<GoalResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/goals/{goal_id}")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn evaluate(&self, goal_id: &str) -> Result<GoalEvaluationResult> {
        let resp = self
            .http
            .get(self.v4(&format!("/goals/{goal_id}/evaluate")))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn update_status(&self, goal_id: &str, status: GoalStatus) -> Result<GoalResult> {
        let body = json!({ "status": status });
        let resp = self
            .http
            .patch(self.v4(&format!("/goals/{goal_id}")))
            .json(&body)
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn abandon(&self, goal_id: &str) -> Result<()> {
        let resp = self
            .http
            .post(self.v4(&format!("/goals/{goal_id}/abandon")))
            .send()
            .await?;
        self.check(resp).await?;
        Ok(())
    }
}
