use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "qmit",
    author = "Rishal",
    version,
    about = "AI-powered conventional commit generator and assistant",
    long_about = "qmit inspects your staged git changes and uses AI (Gemini, ChatGPT, Claude, Ollama) \
to generate standard, high-quality Conventional Commit messages. You can review, edit, or regenerate \
before committing.",
    disable_version_flag = true
)]
pub struct Cli {
    /// Print version information
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: Option<bool>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Stage all tracked modified and deleted files before generating commit (git add -u)
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Stage all changes including untracked files before generating commit (git add -A)
    #[arg(short = 'A', long = "all-files")]
    pub all_files: bool,

    /// Commit automatically without interactive confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Amend the previous commit instead of creating a new one
    #[arg(long)]
    pub amend: bool,

    /// Generate and display the commit message without actually committing
    #[arg(short = 'd', long = "dry-run")]
    pub dry_run: bool,

    /// Provide a hint or additional context to guide the AI commit message
    #[arg(short = 'm', long = "hint", value_name = "HINT")]
    pub hint: Option<String>,

    /// Override the AI provider (gemini, openai, anthropic, ollama, custom)
    #[arg(long, value_name = "PROVIDER")]
    pub provider: Option<String>,

    /// Override the model name (e.g. gemini-2.0-flash, gpt-4o, claude-3-5-sonnet)
    #[arg(long, value_name = "MODEL")]
    pub model: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage qmit configuration (AI providers, API keys, models)
    Config(ConfigArgs),
    /// Run the interactive configuration wizard
    Setup,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigSubcommands>,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    /// Run interactive configuration wizard
    Setup,
    /// Show current active configuration
    Show,
    /// Set a configuration value (e.g. `qmit config set provider gemini`)
    Set {
        key: String,
        value: String,
    },
    /// Print the path to the configuration file
    Path,
}
