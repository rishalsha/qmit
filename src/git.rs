use anyhow::{bail, Context, Result};
use std::process::Command;

pub struct GitContext {
    pub branch: String,
    pub staged_files: Vec<String>,
    pub status_short: String,
    pub recent_commits: Vec<String>,
    pub diff: String,
    pub stat: String,
}

pub fn is_git_repository() -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();

    match output {
        Ok(out) => out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true",
        Err(_) => false,
    }
}

pub fn has_staged_changes() -> Result<bool> {
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to run 'git diff --cached --quiet'")?;

    // exit code 1 means there ARE differences staged
    // exit code 0 means there are NO differences staged
    Ok(status.code() == Some(1))
}

pub fn has_unstaged_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(!text.trim().is_empty())
}

pub fn stage_modified() -> Result<()> {
    let status = Command::new("git")
        .args(["add", "-u"])
        .status()
        .context("Failed to run 'git add -u'")?;

    if !status.success() {
        bail!("'git add -u' exited with non-zero status");
    }
    Ok(())
}

pub fn stage_all() -> Result<()> {
    let status = Command::new("git")
        .args(["add", "-A"])
        .status()
        .context("Failed to run 'git add -A'")?;

    if !status.success() {
        bail!("'git add -A' exited with non-zero status");
    }
    Ok(())
}

pub fn collect_git_context() -> Result<GitContext> {
    // Current branch
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

    // Short status
    let status_output = Command::new("git")
        .args(["status", "--short"])
        .output()
        .context("Failed to get git status")?;
    let status_short = String::from_utf8_lossy(&status_output.stdout).trim().to_string();

    // Staged files list
    let files_output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("Failed to get staged file names")?;
    let files_str = String::from_utf8_lossy(&files_output.stdout);
    let staged_files: Vec<String> = files_str
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Recent 5 commits for style context
    let log_output = Command::new("git")
        .args(["log", "-n", "5", "--oneline"])
        .output();
    let recent_commits = match log_output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    };

    // Staged stat
    let stat_output = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .output()
        .context("Failed to get diff stat")?;
    let stat = String::from_utf8_lossy(&stat_output.stdout).trim().to_string();

    // Staged diff
    // Ignore lockfiles in raw diff if possible or truncate large diffs
    let diff_output = Command::new("git")
        .args([
            "diff",
            "--cached",
            "--no-color",
            "--",
            ".",
            ":(exclude)*Cargo.lock",
            ":(exclude)*package-lock.json",
            ":(exclude)*yarn.lock",
            ":(exclude)*pnpm-lock.yaml",
            ":(exclude)*poetry.lock",
            ":(exclude)*go.sum",
        ])
        .output()
        .context("Failed to get git diff")?;
    
    let mut diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
    if diff.trim().is_empty() {
        // If diff was only lockfiles, get regular diff
        let raw_diff_output = Command::new("git")
            .args(["diff", "--cached", "--no-color"])
            .output()
            .context("Failed to get git diff")?;
        diff = String::from_utf8_lossy(&raw_diff_output.stdout).to_string();
    }

    // Limit diff size to ~30,000 chars to avoid exhausting LLM context limits while keeping fidelity
    const MAX_DIFF_LEN: usize = 30_000;
    if diff.len() > MAX_DIFF_LEN {
        let mut truncated = diff[..MAX_DIFF_LEN].to_string();
        truncated.push_str(&format!(
            "\n\n[... Diff truncated: original size was {} characters, {} files changed. See stat above for full list ...]\n",
            diff.len(),
            staged_files.len()
        ));
        diff = truncated;
    }

    Ok(GitContext {
        branch,
        staged_files,
        status_short,
        recent_commits,
        diff,
        stat,
    })
}

pub fn execute_commit(message: &str, amend: bool) -> Result<()> {
    let temp_file = tempfile::Builder::new()
        .prefix("qmit-msg-")
        .suffix(".txt")
        .tempfile()
        .context("Failed to create temporary file for commit message")?;

    std::fs::write(temp_file.path(), message)
        .context("Failed to write commit message to temporary file")?;

    let mut args = vec!["commit", "-F", temp_file.path().to_str().unwrap()];
    if amend {
        args.push("--amend");
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to run 'git commit'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "Git commit failed:\n{}{}",
            stdout,
            stderr
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}
