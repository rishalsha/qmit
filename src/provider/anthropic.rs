use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::json;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
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
        let endpoint = "https://api.anthropic.com/v1/messages";

        let body = json!({
            "model": self.model,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "temperature": 0.2,
            "max_tokens": 1000
        });

        let response = self
            .client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .header("x-api-key", self.api_key.trim())
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let status = response.status();
        let resp_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse response JSON from Anthropic API")?;

        if !status.is_success() {
            if let Some(err_msg) = resp_json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                bail!("Anthropic API error ({}): {}", status, err_msg);
            }
            bail!("Anthropic API returned error status {}: {}", status, resp_json);
        }

        let text = resp_json
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("No text content in Anthropic response: {}", resp_json))?;

        Ok(text.to_string())
    }
}
