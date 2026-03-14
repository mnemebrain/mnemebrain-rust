use std::sync::Arc;

use reqwest::Client;
use serde_json::Value;

use crate::error::{MnemeBrainError, Result};
use crate::models::*;

const V4_PREFIX: &str = "/api/mneme";

pub struct ReconsolidationClient {
    http: Arc<Client>,
    base_url: String,
}

impl ReconsolidationClient {
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

    pub async fn queue(&self) -> Result<ReconsolidationQueueResult> {
        let resp = self
            .http
            .get(self.v4("/reconsolidation/queue"))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn run(&self) -> Result<ReconsolidationRunResult> {
        let resp = self
            .http
            .post(self.v4("/reconsolidation/run"))
            .send()
            .await?;
        let v = self.check(resp).await?;
        Ok(serde_json::from_value(v)?)
    }
}
