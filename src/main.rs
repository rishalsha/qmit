mod cli;
mod config;
mod git;
mod prompt;
mod provider;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands, ConfigSubcommands};
use colored::Colorize;
use config::Config;
use indicatif::{ProgressBar, ProgressStyle};
use provider::ProviderKind;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(Commands::Config(config_args)) = cli.command {
        return handle_config_command(config_args);
    }

    // Ensure we are inside a git repository
    if !git::is_git_repository() {
        eprintln!(
            "{} Not a git repository (or any of the parent directories).",
            "Error:".red().bold()
        );
        eprintln!("Please run {} inside a git repository.", "qmit".cyan());
        std::process::exit(1);
    }

    // Handle staging flags
    if cli.all_files {
        git::stage_all().context("Failed to stage all files")?;
        println!("{}", "Staged all changes (git add -A)".dimmed());
    } else if cli.all {
        git::stage_modified().context("Failed to stage modified files")?;
        println!("{}", "Staged tracked modified files (git add -u)".dimmed());
    } else if !git::has_staged_changes()? {
        if git::has_unstaged_changes()? {
            match ui::prompt_unresponsive_stage()? {
                Some(true) => {
                    git::stage_all().context("Failed to stage all files")?;
                    println!("{}", "Staged all changes.".dimmed());
                }
                Some(false) => {
                    git::stage_modified().context("Failed to stage modified files")?;
                    println!("{}", "Staged tracked modified files.".dimmed());
                }
                None => {
                    println!("{}", "Commit aborted (no changes staged).".yellow());
                    return Ok(());
                }
            }
        } else {
            println!("{}", "Working tree clean, nothing to commit.".green());
            return Ok(());
        }
    }

    // Collect git diff and context
    let git_ctx = git::collect_git_context()?;
    if git_ctx.staged_files.is_empty() && git_ctx.diff.trim().is_empty() {
        println!("{}", "No staged changes found to commit.".yellow());
        return Ok(());
    }

    // Load or setup configuration
    let mut config = Config::load()?;
    let mut resolved = config.resolve(cli.provider.as_deref(), cli.model.as_deref())?;

    // If API key is missing and provider needs one, trigger interactive setup
    if resolved.api_key.is_none() && resolved.provider_name != "ollama" {
        println!(
            "{}",
            "No API key configured for provider. Let's set it up quickly:".yellow().bold()
        );
        config = Config::interactive_setup()?;
        resolved = config.resolve(cli.provider.as_deref(), cli.model.as_deref())?;
    }

    ui::print_banner(&resolved.provider_name, &resolved.model);

    let provider = provider::create_provider(&resolved)?;
    let mut current_hint = cli.hint.clone();

    // Initial message generation
    let mut message = generate_message_with_spinner(
        &provider,
        &git_ctx,
        current_hint.as_deref(),
    )
    .await?;

    // Dry-run mode: display and exit
    if cli.dry_run {
        println!("{}", "\n[Dry-run mode: Commit message will not be committed]".dimmed());
        ui::display_commit_box(&message);
        return Ok(());
    }

    // Yes mode: commit immediately without prompting
    if cli.yes {
        ui::display_commit_box(&message);
        git::execute_commit(&message, cli.amend)?;
        println!("{} Successfully committed!", "✔".green().bold());
        return Ok(());
    }

    // Interactive review loop
    loop {
        ui::display_commit_box(&message);

        match ui::prompt_user_action(&message)? {
            ui::UserAction::Commit(final_msg) => {
                git::execute_commit(&final_msg, cli.amend)?;
                println!("\n{} Successfully committed!", "✔".green().bold());
                break;
            }
            ui::UserAction::Edit(edited_msg) => {
                message = edited_msg;
                println!("{}", "Updated commit message:".dimmed());
            }
            ui::UserAction::Regenerate(feedback) => {
                if let Some(fb) = feedback {
                    let combined = match current_hint {
                        Some(ref prev) => format!("{}\nAdditional guidance: {}", prev, fb),
                        None => fb,
                    };
                    current_hint = Some(combined);
                }

                message = generate_message_with_spinner(
                    &provider,
                    &git_ctx,
                    current_hint.as_deref(),
                )
                .await?;
            }
            ui::UserAction::Abort => {
                println!("{}", "Commit aborted.".yellow());
                break;
            }
        }
    }

    Ok(())
}

async fn generate_message_with_spinner(
    provider: &ProviderKind,
    git_ctx: &git::GitContext,
    hint: Option<&str>,
) -> Result<String> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.green} {msg}")?,
    );
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message("Generating conventional commit message with AI...");

    let user_prompt = prompt::build_user_prompt(git_ctx, hint);
    let result = provider
        .generate_commit_message(prompt::SYSTEM_PROMPT, &user_prompt)
        .await;

    spinner.finish_and_clear();

    let raw = result?;
    let cleaned = prompt::clean_commit_message(&raw);
    Ok(cleaned)
}

fn handle_config_command(config_args: cli::ConfigArgs) -> Result<()> {
    let mut config = Config::load()?;

    match config_args.command {
        Some(ConfigSubcommands::Setup) | None => {
            Config::interactive_setup()?;
        }
        Some(ConfigSubcommands::Show) => {
            config.display_current()?;
        }
        Some(ConfigSubcommands::Set { key, value }) => {
            config.set_key_value(&key, &value)?;
            println!(
                "{} Successfully set '{}' = '{}'",
                "✔".green().bold(),
                key.cyan(),
                value.yellow()
            );
        }
        Some(ConfigSubcommands::Path) => {
            println!("{}", Config::config_path()?.display());
        }
    }

    Ok(())
}
