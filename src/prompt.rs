use crate::git::GitContext;

pub const SYSTEM_PROMPT: &str = r#"You are an expert Git commit generator adhering strictly to the Conventional Commits v1.0.0 specification.
Your goal is to inspect the provided staged Git diff, status, and context, and output a precise, clear, and professional conventional commit message.

### Format Rules:
1. Format must strictly follow:
   <type>(<optional scope>): <description>

   [optional body explaining 'what' and 'why', using bullet points '- ']

   [optional footer(s), e.g. BREAKING CHANGE: <explanation>]

2. Allowed types:
   - feat: A new feature for the user or consumer
   - fix: A bug fix for the user or consumer
   - docs: Documentation only changes
   - style: Changes that do not affect code logic (formatting, missing semicolons, etc.)
   - refactor: A code change that neither fixes a bug nor adds a feature
   - perf: A code change that improves performance
   - test: Adding or updating tests
   - build: Changes affecting build system, packaging, or external dependencies (cargo, npm, etc.)
   - ci: Changes to CI configuration files and workflows
   - chore: Maintenance tasks, tooling updates, housekeeping
   - revert: Reverts a previous commit

3. Subject line rules:
   - Imperative, present tense: "add" not "added" or "adds", "fix" not "fixed".
   - Start the description after the colon with a lowercase letter.
   - Do NOT end the subject line with a period.
   - Keep the subject line within 72 characters (ideally <= 50 characters).
   - Include a concise scope in parentheses if clear from the files or modules changed (e.g., `feat(auth):`, `fix(cli):`).

4. Body rules:
   - Leave one blank line between the subject and the body.
   - Explain WHAT changed and WHY, not mechanical line-by-line repetitions.
   - For multi-file or multi-part changes, use concise bullet points (`- ...`).
   - If trivial or small (e.g. single typo fix), a single subject line without a body is preferred.

5. OUTPUT REQUIREMENT:
   - Output ONLY the raw commit message.
   - Do NOT wrap the message in markdown code fences (no ```).
   - Do NOT add any preamble, explanation, or greeting (e.g., do NOT write "Here is the commit message:").
"#;

pub fn build_user_prompt(git_ctx: &GitContext, hint: Option<&str>) -> String {
    let mut prompt = String::new();

    if let Some(h) = hint {
        if !h.trim().is_empty() {
            prompt.push_str(&format!("### User Hint / Guidance:\n{}\n\n", h.trim()));
        }
    }

    if !git_ctx.branch.is_empty() {
        prompt.push_str(&format!("Branch: {}\n\n", git_ctx.branch));
    }

    if !git_ctx.status_short.is_empty() {
        prompt.push_str("### Staged Status:\n");
        prompt.push_str(&git_ctx.status_short);
        prompt.push_str("\n\n");
    }

    if !git_ctx.staged_files.is_empty() {
        prompt.push_str("### Staged Files:\n");
        for file in &git_ctx.staged_files {
            prompt.push_str(&format!("- {}\n", file));
        }
        prompt.push('\n');
    }

    if !git_ctx.recent_commits.is_empty() {
        prompt.push_str("### Recent Repository Commits (for style context):\n");
        for commit in &git_ctx.recent_commits {
            prompt.push_str(&format!("- {}\n", commit));
        }
        prompt.push('\n');
    }

    if !git_ctx.stat.is_empty() {
        prompt.push_str("### Staged Diff Summary:\n");
        prompt.push_str(&git_ctx.stat);
        prompt.push_str("\n\n");
    }

    prompt.push_str("### Staged Git Diff:\n");
    prompt.push_str(&git_ctx.diff);
    prompt.push_str("\n\nGenerate the conventional commit message now:");

    prompt
}

pub fn clean_commit_message(raw: &str) -> String {
    let mut text = raw.trim();

    // Remove markdown code fences if model enclosed it in ``` or ```git
    if text.starts_with("```") {
        if let Some(pos) = text.find('\n') {
            text = &text[pos + 1..];
        }
        if text.ends_with("```") {
            text = text.trim_end_matches("```");
        }
    }

    // Strip leading phrases like "Here is the commit message:" or "Commit message:"
    let lines: Vec<&str> = text.lines().collect();
    let mut start_idx = 0;
    while start_idx < lines.len() {
        let line = lines[start_idx].trim();
        let lower = line.to_lowercase();
        if lower.starts_with("here is")
            || lower.starts_with("commit message:")
            || lower.starts_with("proposed commit message:")
            || lower.is_empty()
        {
            start_idx += 1;
        } else {
            break;
        }
    }

    let cleaned = lines[start_idx..].join("\n");
    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_message_with_fences() {
        let input = "```git\nfeat(cli): add prompt support\n\n- Add interactive loop\n```";
        let cleaned = clean_commit_message(input);
        assert_eq!(
            cleaned,
            "feat(cli): add prompt support\n\n- Add interactive loop"
        );
    }

    #[test]
    fn test_clean_message_with_preamble() {
        let input = "Here is the commit message:\n\nfix(parser): handle empty strings";
        let cleaned = clean_commit_message(input);
        assert_eq!(cleaned, "fix(parser): handle empty strings");
    }
}
