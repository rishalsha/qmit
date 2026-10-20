use anyhow::{Context, Result};
use colored::Colorize;
use inquire::{Editor, Select, Text};

pub enum UserAction {
    Commit(String),
    Edit(String),
    Regenerate(Option<String>),
    Abort,
}

pub fn print_banner(provider: &str, model: &str) {
    println!(
        "{} {} {}",
        "qmit".bright_cyan().bold(),
        "•".dimmed(),
        format!("[provider: {}, model: {}]", provider.green(), model.yellow()).dimmed()
    );
}

pub fn display_commit_box(message: &str) {
    let width = 74;
    let horizontal_line = "─".repeat(width);

    println!("\n{}", horizontal_line.cyan());
    println!("{}", " Proposed Commit Message:".cyan().bold());
    println!("{}", horizontal_line.cyan());

    let lines: Vec<&str> = message.lines().collect();
    if let Some(first_line) = lines.first() {
        println!("{}", first_line.bright_green().bold());
        if lines.len() > 1 {
            for line in &lines[1..] {
                println!("{}", line);
            }
        }
    } else {
        println!("{}", message);
    }

    println!("{}\n", horizontal_line.cyan());
}

pub fn prompt_user_action(current_message: &str) -> Result<UserAction> {
    let options = vec![
        "✔  Accept & Commit",
        "✎  Edit message in editor",
        "✏  Edit message inline",
        "🔄 Regenerate (optional feedback)",
        "✖  Cancel / Abort",
    ];

    let choice = Select::new("Action:", options)
        .with_starting_cursor(0)
        .prompt()?;

    match choice {
        c if c.starts_with("✔") => Ok(UserAction::Commit(current_message.to_string())),
        c if c.starts_with("✎") => {
            let edited = edit_in_editor(current_message)?;
            if edited.trim().is_empty() {
                println!("{}", "Edited message is empty! Keeping previous message.".yellow());
                Ok(UserAction::Commit(current_message.to_string()))
            } else {
                Ok(UserAction::Edit(edited.trim().to_string()))
            }
        }
        c if c.starts_with("✏") => {
            let edited = edit_inline(current_message)?;
            if edited.trim().is_empty() {
                println!("{}", "Edited message is empty! Keeping previous message.".yellow());
                Ok(UserAction::Commit(current_message.to_string()))
            } else {
                Ok(UserAction::Edit(edited.trim().to_string()))
            }
        }
        c if c.starts_with("🔄") => {
            let feedback = Text::new("Feedback / guidance for regeneration (optional):")
                .with_placeholder("e.g. make it shorter, mark as fix, focus on database changes")
                .prompt()?;
            let hint = if feedback.trim().is_empty() {
                None
            } else {
                Some(feedback.trim().to_string())
            };
            Ok(UserAction::Regenerate(hint))
        }
        _ => Ok(UserAction::Abort),
    }
}

pub fn edit_in_editor(initial_text: &str) -> Result<String> {
    // Try inquire's built-in editor first
    let result = Editor::new("Edit commit message:")
        .with_predefined_text(initial_text)
        .prompt();

    match result {
        Ok(text) => Ok(text),
        Err(_) => {
            // Fallback: manually invoke $EDITOR or nano/vi with temporary file
            manual_editor_fallback(initial_text)
        }
    }
}

fn manual_editor_fallback(initial_text: &str) -> Result<String> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    let temp_file = tempfile::Builder::new()
        .prefix("qmit-edit-")
        .suffix(".txt")
        .tempfile()
        .context("Failed to create temporary file for editing")?;

    std::fs::write(temp_file.path(), initial_text)
        .context("Failed to write to temporary file")?;

    let status = std::process::Command::new(&editor)
        .arg(temp_file.path())
        .status()
        .with_context(|| format!("Failed to launch editor '{}'", editor))?;

    if !status.success() {
        eprintln!("{}", format!("Editor '{}' exited with non-zero status", editor).yellow());
    }

    let updated = std::fs::read_to_string(temp_file.path())
        .context("Failed to read back edited file")?;

    Ok(updated)
}

pub fn edit_inline(initial_text: &str) -> Result<String> {
    let first_line = initial_text.lines().next().unwrap_or(initial_text);
    println!("Edit the subject line:");
    let new_subject = Text::new("Subject:")
        .with_initial_value(first_line)
        .prompt()?;

    let mut remaining = initial_text.lines().skip(1).collect::<Vec<&str>>().join("\n");
    if !remaining.trim().is_empty() {
        println!("\nExisting body:\n{}", remaining.dimmed());
        let keep_body = Select::new("Keep existing body?", vec!["Yes", "No / Clear body"])
            .with_starting_cursor(0)
            .prompt()?;
        if keep_body.starts_with("No") {
            remaining.clear();
        }
    }

    if remaining.trim().is_empty() {
        Ok(new_subject.trim().to_string())
    } else {
        Ok(format!("{}\n{}", new_subject.trim(), remaining.trim_start()))
    }
}

pub fn prompt_unresponsive_stage() -> Result<Option<bool>> {
    println!("{}", "No staged changes detected in git.".yellow().bold());
    let options = vec![
        "Stage all modified & deleted files (git add -u)",
        "Stage all files including untracked (git add -A)",
        "Abort",
    ];

    let choice = Select::new("How would you like to proceed?", options)
        .with_starting_cursor(0)
        .prompt()?;

    if choice.starts_with("Stage all modified") {
        Ok(Some(false)) // false = -u
    } else if choice.starts_with("Stage all files") {
        Ok(Some(true)) // true = -A
    } else {
        Ok(None)
    }
}
