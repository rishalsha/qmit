use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::json;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            base_url,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub async fn generate_commit_message(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let endpoint = if self.base_url.ends_with("/chat/completions") {
            self.base_url.clone()
        } else {
            format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
        };

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "temperature": 0.2,
            "max_tokens": 1000
        });

        let mut req = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json");

        if !self.api_key.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key.trim()));
        }

        let response = req
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to OpenAI endpoint ({})", endpoint))?;

        let status = response.status();
        let resp_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response JSON from OpenAI endpoint")?;

        if !status.is_success() {
            if let Some(err_msg) = resp_json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                bail!("OpenAI API error ({}): {}", status, err_msg);
            }
            bail!("API returned error status {}: {}", status, resp_json);
        }

        let content = resp_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|cnt| cnt.as_str())
            .ok_or_else(|| anyhow::anyhow!("No content found in OpenAI response: {}", resp_json))?;

        Ok(content.to_string())
    }
}
