use anyhow::{bail, Context, Result};
use colored::Colorize;
use inquire::{Password, Select, Text};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_provider")]
    pub provider: String,

    #[serde(default)]
    pub gemini: ProviderConfig,

    #[serde(default)]
    pub openai: ProviderConfig,

    #[serde(default)]
    pub anthropic: ProviderConfig,

    #[serde(default)]
    pub ollama: ProviderConfig,

    #[serde(default)]
    pub custom: ProviderConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

fn default_provider() -> String {
    "gemini".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "gemini".to_string(),
            gemini: ProviderConfig {
                api_key: None,
                model: Some("gemini-2.5-flash".to_string()),
                base_url: None,
            },
            openai: ProviderConfig {
                api_key: None,
                model: Some("gpt-4o-mini".to_string()),
                base_url: Some("https://api.openai.com/v1".to_string()),
            },
            anthropic: ProviderConfig {
                api_key: None,
                model: Some("claude-3-5-haiku-20241022".to_string()),
                base_url: None,
            },
            ollama: ProviderConfig {
                api_key: None,
                model: Some("llama3.2".to_string()),
                base_url: Some("http://localhost:11434/v1".to_string()),
            },
            custom: ProviderConfig {
                api_key: None,
                model: None,
                base_url: None,
            },
        }
    }
}

pub struct ResolvedConfig {
    pub provider_name: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine user configuration directory")?
            .join("qmit");
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file at {:?}", path))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }
        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file to {:?}", path))?;
        Ok(())
    }

    /// Resolve effective settings with CLI overrides and environment variables
    pub fn resolve(
        &self,
        override_provider: Option<&str>,
        override_model: Option<&str>,
    ) -> Result<ResolvedConfig> {
        let provider_name = override_provider
            .unwrap_or(&self.provider)
            .to_lowercase();

        let (mut model, mut api_key, base_url) = match provider_name.as_str() {
            "gemini" | "google" => {
                let env_key = std::env::var("GEMINI_API_KEY")
                    .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                    .ok();
                (
                    self.gemini.model.clone().unwrap_or_else(|| "gemini-2.5-flash".to_string()),
                    env_key.or_else(|| self.gemini.api_key.clone()),
                    self.gemini.base_url.clone(),
                )
            }
            "openai" | "chatgpt" => {
                let env_key = std::env::var("OPENAI_API_KEY").ok();
                (
                    self.openai.model.clone().unwrap_or_else(|| "gpt-4o-mini".to_string()),
                    env_key.or_else(|| self.openai.api_key.clone()),
                    self.openai.base_url.clone().or_else(|| Some("https://api.openai.com/v1".to_string())),
                )
            }
            "anthropic" | "claude" => {
                let env_key = std::env::var("ANTHROPIC_API_KEY").ok();
                (
                    self.anthropic.model.clone().unwrap_or_else(|| "claude-3-5-haiku-20241022".to_string()),
                    env_key.or_else(|| self.anthropic.api_key.clone()),
                    self.anthropic.base_url.clone(),
                )
            }
            "ollama" => (
                self.ollama.model.clone().unwrap_or_else(|| "llama3.2".to_string()),
                self.ollama.api_key.clone().or_else(|| Some("ollama".to_string())),
                self.ollama.base_url.clone().or_else(|| Some("http://localhost:11434/v1".to_string())),
            ),
            "custom" => {
                let env_key = std::env::var("QMIT_API_KEY").ok();
                (
                    self.custom.model.clone().unwrap_or_else(|| "default".to_string()),
                    env_key.or_else(|| self.custom.api_key.clone()),
                    self.custom.base_url.clone(),
                )
            }
            other => bail!("Unknown provider '{}'. Supported: gemini, openai, anthropic, ollama, custom", other),
        };

        if let Some(m) = override_model {
            model = m.to_string();
        }

        // Check generic QMIT_API_KEY as fallback
        if api_key.is_none() {
            if let Ok(k) = std::env::var("QMIT_API_KEY") {
                api_key = Some(k);
            }
        }

        Ok(ResolvedConfig {
            provider_name,
            model,
            api_key,
            base_url,
        })
    }

    /// Interactive configuration wizard
    pub fn interactive_setup() -> Result<Self> {
        println!("{}", "─── qmit Configuration Wizard ───".cyan().bold());
        println!("Configure which AI provider you'd like to use for generating commit messages.\n");

        let providers = vec![
            "Gemini (Google AI - Recommended)",
            "OpenAI (ChatGPT - gpt-4o / gpt-4o-mini)",
            "Anthropic (Claude 3.5)",
            "Ollama (Local LLM)",
            "Custom (OpenAI-compatible endpoint)",
        ];

        let selection = Select::new("Choose AI Provider:", providers)
            .with_starting_cursor(0)
            .prompt()?;

        let mut config = Self::load().unwrap_or_default();

        match selection {
            s if s.starts_with("Gemini") => {
                config.provider = "gemini".to_string();
                let current_key = config.gemini.api_key.clone().unwrap_or_default();
                let key_hint = if !current_key.is_empty() {
                    format!(" [currently {}]", mask_key(&current_key))
                } else {
                    "".to_string()
                };

                let api_key = Password::new(&format!("Enter Gemini API key{}:", key_hint))
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .prompt()?;

                if !api_key.trim().is_empty() {
                    config.gemini.api_key = Some(api_key.trim().to_string());
                }

                let model_options = vec![
                    "gemini-2.5-flash (recommended, fast & responsive)",
                    "gemini-2.5-pro (high reasoning capability)",
                    "gemini-3.8-flash (latest preview)",
                    "Other (type custom model name)",
                ];
                let chosen_model = Select::new("Select Gemini Model:", model_options)
                    .with_starting_cursor(0)
                    .prompt()?;
                if chosen_model.starts_with("Other") {
                    let custom_model = Text::new("Enter model name:").prompt()?;
                    config.gemini.model = Some(custom_model.trim().to_string());
                } else {
                    let m = chosen_model.split_whitespace().next().unwrap();
                    config.gemini.model = Some(m.to_string());
                }
            }
            s if s.starts_with("OpenAI") => {
                config.provider = "openai".to_string();
                let current_key = config.openai.api_key.clone().unwrap_or_default();
                let key_hint = if !current_key.is_empty() {
                    format!(" [currently {}]", mask_key(&current_key))
                } else {
                    "".to_string()
                };

                let api_key = Password::new(&format!("Enter OpenAI API key{}:", key_hint))
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .prompt()?;

                if !api_key.trim().is_empty() {
                    config.openai.api_key = Some(api_key.trim().to_string());
                }

                let model_options = vec![
                    "gpt-4o-mini (fast, cheap, recommended)",
                    "gpt-4o (high capability)",
                    "Other (type custom model name)",
                ];
                let chosen_model = Select::new("Select OpenAI Model:", model_options)
                    .with_starting_cursor(0)
                    .prompt()?;
                if chosen_model.starts_with("Other") {
                    let custom_model = Text::new("Enter model name:").prompt()?;
                    config.openai.model = Some(custom_model.trim().to_string());
                } else {
                    let m = chosen_model.split_whitespace().next().unwrap();
                    config.openai.model = Some(m.to_string());
                }
            }
            s if s.starts_with("Anthropic") => {
                config.provider = "anthropic".to_string();
                let current_key = config.anthropic.api_key.clone().unwrap_or_default();
                let key_hint = if !current_key.is_empty() {
                    format!(" [currently {}]", mask_key(&current_key))
                } else {
                    "".to_string()
                };

                let api_key = Password::new(&format!("Enter Anthropic API key{}:", key_hint))
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .prompt()?;

                if !api_key.trim().is_empty() {
                    config.anthropic.api_key = Some(api_key.trim().to_string());
                }

                let model_options = vec![
                    "claude-3-5-haiku-20241022 (fast, great for commits)",
                    "claude-3-5-sonnet-20241022 (powerful)",
                    "Other (type custom model name)",
                ];
                let chosen_model = Select::new("Select Anthropic Model:", model_options)
                    .with_starting_cursor(0)
                    .prompt()?;
                if chosen_model.starts_with("Other") {
                    let custom_model = Text::new("Enter model name:").prompt()?;
                    config.anthropic.model = Some(custom_model.trim().to_string());
                } else {
                    let m = chosen_model.split_whitespace().next().unwrap();
                    config.anthropic.model = Some(m.to_string());
                }
            }
            s if s.starts_with("Ollama") => {
                config.provider = "ollama".to_string();
                let url = Text::new("Ollama URL:")
                    .with_default("http://localhost:11434/v1")
                    .prompt()?;
                config.ollama.base_url = Some(url.trim().to_string());

                let model = Text::new("Ollama Model Name:")
                    .with_default("llama3.2")
                    .prompt()?;
                config.ollama.model = Some(model.trim().to_string());
            }
            s if s.starts_with("Custom") => {
                config.provider = "custom".to_string();
                let url = Text::new("Endpoint Base URL (OpenAI-compatible):")
                    .with_placeholder("https://api.deepseek.com/v1 or https://api.groq.com/openai/v1")
                    .prompt()?;
                config.custom.base_url = Some(url.trim().to_string());

                let model = Text::new("Model Name:")
                    .prompt()?;
                config.custom.model = Some(model.trim().to_string());

                let api_key = Password::new("API key (optional for local):")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .prompt()?;
                if !api_key.trim().is_empty() {
                    config.custom.api_key = Some(api_key.trim().to_string());
                }
            }
            _ => {}
        }

        config.save()?;
        println!("\n{} Configuration saved to {:?}", "✔".green().bold(), Self::config_path()?);
        Ok(config)
    }

    pub fn set_key_value(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        match parts.as_slice() {
            ["provider"] | ["default_provider"] => {
                self.provider = value.to_lowercase();
            }
            ["gemini", "api_key"] => self.gemini.api_key = Some(value.to_string()),
            ["gemini", "model"] => self.gemini.model = Some(value.to_string()),
            ["gemini", "base_url"] => self.gemini.base_url = Some(value.to_string()),

            ["openai", "api_key"] => self.openai.api_key = Some(value.to_string()),
            ["openai", "model"] => self.openai.model = Some(value.to_string()),
            ["openai", "base_url"] => self.openai.base_url = Some(value.to_string()),

            ["anthropic", "api_key"] => self.anthropic.api_key = Some(value.to_string()),
            ["anthropic", "model"] => self.anthropic.model = Some(value.to_string()),
            ["anthropic", "base_url"] => self.anthropic.base_url = Some(value.to_string()),

            ["ollama", "api_key"] => self.ollama.api_key = Some(value.to_string()),
            ["ollama", "model"] => self.ollama.model = Some(value.to_string()),
            ["ollama", "base_url"] => self.ollama.base_url = Some(value.to_string()),

            ["custom", "api_key"] => self.custom.api_key = Some(value.to_string()),
            ["custom", "model"] => self.custom.model = Some(value.to_string()),
            ["custom", "base_url"] => self.custom.base_url = Some(value.to_string()),

            _ => bail!(
                "Unknown configuration key '{}'. Examples: provider, gemini.model, gemini.api_key, openai.model",
                key
            ),
        }
        self.save()?;
        Ok(())
    }

    pub fn display_current(&self) -> Result<()> {
        let path = Self::config_path()?;
        println!("{}", "─── Current qmit Configuration ───".cyan().bold());
        println!("File: {}", path.display().to_string().dimmed());
        println!("Active Provider: {}", self.provider.green().bold());

        let print_provider = |name: &str, cfg: &ProviderConfig| {
            let active = if self.provider == name { " (active)".green().bold() } else { "".normal() };
            println!("\n[{}{}]", name.yellow(), active);
            println!("  model    = {}", cfg.model.as_deref().unwrap_or("none").cyan());
            println!("  base_url = {}", cfg.base_url.as_deref().unwrap_or("default").dimmed());
            println!(
                "  api_key  = {}",
                cfg.api_key.as_deref().map(mask_key).unwrap_or_else(|| "not set".dimmed().to_string())
            );
        };

        print_provider("gemini", &self.gemini);
        print_provider("openai", &self.openai);
        print_provider("anthropic", &self.anthropic);
        print_provider("ollama", &self.ollama);
        print_provider("custom", &self.custom);
        println!();
        Ok(())
    }
}

pub fn mask_key(k: &str) -> String {
    if k.len() <= 8 {
        "********".to_string()
    } else {
        format!("{}...{}", &k[..4], &k[k.len() - 4..])
    }
}
