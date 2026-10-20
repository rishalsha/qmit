pub mod anthropic;
pub mod gemini;
pub mod openai;

use crate::config::ResolvedConfig;
use anyhow::{bail, Result};

pub enum ProviderKind {
    Gemini(gemini::GeminiProvider),
    OpenAi(openai::OpenAiProvider),
    Anthropic(anthropic::AnthropicProvider),
}

impl ProviderKind {
    pub async fn generate_commit_message(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        match self {
            Self::Gemini(p) => p.generate_commit_message(system_prompt, user_prompt).await,
            Self::OpenAi(p) => p.generate_commit_message(system_prompt, user_prompt).await,
            Self::Anthropic(p) => p.generate_commit_message(system_prompt, user_prompt).await,
        }
    }
}

pub fn create_provider(resolved: &ResolvedConfig) -> Result<ProviderKind> {
    match resolved.provider_name.as_str() {
        "gemini" | "google" => {
            let api_key = resolved.api_key.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Gemini API key is not configured. Run `qmit config setup` or set GEMINI_API_KEY environment variable."
                )
            })?;
            Ok(ProviderKind::Gemini(gemini::GeminiProvider::new(
                api_key.clone(),
                resolved.model.clone(),
                resolved.base_url.clone(),
            )))
        }
        "openai" | "chatgpt" => {
            let api_key = resolved.api_key.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "OpenAI API key is not configured. Run `qmit config setup` or set OPENAI_API_KEY environment variable."
                )
            })?;
            Ok(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                api_key.clone(),
                resolved.model.clone(),
                resolved.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            )))
        }
        "anthropic" | "claude" => {
            let api_key = resolved.api_key.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Anthropic API key is not configured. Run `qmit config setup` or set ANTHROPIC_API_KEY environment variable."
                )
            })?;
            Ok(ProviderKind::Anthropic(anthropic::AnthropicProvider::new(
                api_key.clone(),
                resolved.model.clone(),
            )))
        }
        "ollama" => {
            let base_url = resolved
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string());
            let api_key = resolved.api_key.clone().unwrap_or_else(|| "ollama".to_string());
            Ok(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                api_key,
                resolved.model.clone(),
                base_url,
            )))
        }
        "custom" => {
            let base_url = resolved.base_url.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Custom endpoint base URL is not configured. Run `qmit config setup` to set base_url."
                )
            })?;
            let api_key = resolved.api_key.clone().unwrap_or_default();
            Ok(ProviderKind::OpenAi(openai::OpenAiProvider::new(
                api_key,
                resolved.model.clone(),
                base_url.clone(),
            )))
        }
        other => bail!("Unsupported provider: {}", other),
    }
}
