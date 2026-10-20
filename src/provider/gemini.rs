use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::json;

pub struct GeminiProvider {
    api_key: String,
    model: String,
    base_url: Option<String>,
    client: Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            model,
            base_url,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub async fn generate_commit_message(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta");

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            base.trim_end_matches('/'),
            self.model,
            self.api_key
        );

        let body = json!({
            "system_instruction": {
                "parts": [
                    { "text": system_prompt }
                ]
            },
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        { "text": user_prompt }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 1000,
                "thinkingConfig": {
                    "thinkingBudget": 0
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Google Gemini API")?;

        let status = response.status();
        let resp_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Gemini API response as JSON")?;

        if !status.is_success() {
            if let Some(err_msg) = resp_json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
                bail!("Gemini API error ({}): {}", status, err_msg);
            }
            bail!("Gemini API returned error status {}: {}", status, resp_json);
        }

        let parts = resp_json
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("content"))
            .and_then(|cnt| cnt.get("parts"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| anyhow::anyhow!("Gemini API did not return valid candidates: {}", resp_json))?;

        let text = parts
            .iter()
            .filter(|part| part.get("thought").and_then(|t| t.as_bool()) != Some(true))
            .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        if text.trim().is_empty() {
            bail!("Gemini API returned empty text in candidate parts: {}", resp_json);
        }

        Ok(text)
    }
}
