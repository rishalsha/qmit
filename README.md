# qmit 🚀

[![Crates.io](https://img.shields.io/crates/v/qmit.svg)](https://crates.io/crates/qmit)
[![Release](https://github.com/rishalsha/qmit/actions/workflows/release.yml/badge.svg)](https://github.com/rishalsha/qmit/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**qmit** (pronounced *"commit"*) is a blazing-fast, AI-powered Rust CLI tool that checks your Git diff and crafts standard, high-quality [Conventional Commit](https://www.conventionalcommits.org/en/v1.0.0/) messages.

Never get stuck staring at an empty commit prompt again.

---

## ✨ Features

- **Standard Conventional Commits**: Generates precise commit headers (`feat(scope): ...`, `fix: ...`, `refactor: ...`) and well-structured commit bodies adhering to the Conventional Commits v1.0.0 standard.
- **Multiple AI Providers Supported**:
  - **Google Gemini** (Gemini 2.5 Flash, 2.0 Flash, 1.5 Flash, 1.5 Pro)
  - **OpenAI / ChatGPT** (GPT-4o, GPT-4o-mini)
  - **Anthropic Claude** (Claude 3.5 Haiku, Claude 3.5 Sonnet)
  - **Ollama** (Local models like Llama 3.2, Qwen 2.5 Coder, Mistral)
  - **Custom Endpoints** (OpenAI-compatible APIs like DeepSeek, Groq, OpenRouter, etc.)
- **Interactive Review Loop**:
  - `✔ Accept & Commit`: Immediately commits with the proposed message.
  - `✎ Edit in editor`: Opens your `$EDITOR` (`nvim`, `vim`, `nano`, `code`, etc.) to edit the message.
  - `✏ Edit inline`: Quickly adjust the subject line and body directly in the terminal.
  - `🔄 Regenerate`: Provide hints or feedback to refine the message.
  - `✖ Cancel / Abort`: Abort safely without altering repository state.
- **Intelligent Staging**:
  - Detects if you forgot to stage changes and interactively offers to stage modified files (`git add -u`) or all files (`git add -A`).
  - Supports `-a` / `--all` and `-A` / `--all-files` flags.
- **Context-Aware Diff Analysis**:
  - Examines branch name, staged diff, status indicators, and diff statistics.
  - Inspects recent repository commit history to preserve project conventions.
  - Automatically filters lockfiles (`Cargo.lock`, `package-lock.json`, etc.) from raw diff to preserve context window tokens.
  - Safe commits using temporary message files (immune to shell quote escaping and respects Git hooks).

---

## 📦 Installation

### Option 1: Via Cargo (Crates.io)

```bash
cargo install qmit
```

### Option 2: Pre-compiled Binaries (Linux, macOS, Windows)

Download pre-built standalone binaries for your OS and architecture from the [GitHub Releases](https://github.com/rishalsha/qmit/releases) page.

Unpack the archive and place `qmit` in your system's `PATH` (e.g. `/usr/local/bin` or `~/.local/bin`).

### Option 3: From Source

```bash
git clone https://github.com/rishalsha/qmit.git
cd qmit
cargo install --path .
```

The `qmit` binary will be installed to `~/.cargo/bin/qmit`. Ensure `~/.cargo/bin` is in your `PATH`.

---

## ⚡ Quick Start

### 1. Configure your AI Provider

Run the interactive setup wizard:

```bash
qmit config setup
```

You can choose your provider (Gemini, OpenAI, Anthropic, Ollama, or Custom), enter your API key, and select a model.

Alternatively, `qmit` respects standard environment variables:

```bash
export GEMINI_API_KEY="your-gemini-key"
# or
export OPENAI_API_KEY="your-openai-key"
# or
export ANTHROPIC_API_KEY="your-claude-key"
```

### 2. Make some changes and run `qmit`

```bash
git add src/
qmit
```

`qmit` will analyze your staged diff, query your chosen AI model, and present a formatted Conventional Commit message:

```
qmit • [provider: gemini, model: gemini-2.0-flash]

──────────────────────────────────────────────────────────────────────────
 Proposed Commit Message:
──────────────────────────────────────────────────────────────────────────
feat(cli): add interactive commit review loop

- Add options to edit in $EDITOR or inline
- Add regeneration with custom user feedback
- Support auto-staging modified and untracked files
──────────────────────────────────────────────────────────────────────────

? Action: ›
❯ ✔  Accept & Commit
  ✎  Edit message in editor
  ✏  Edit message inline
  🔄 Regenerate (optional feedback)
  ✖  Cancel / Abort
```

---

## 🛠️ CLI Usage & Flags

```
Usage: qmit [OPTIONS] [COMMAND]

Commands:
  config  Manage qmit configuration (AI providers, API keys, models)
  help    Print this message or the help of the given subcommand(s)

Options:
  -a, --all              Stage tracked modified/deleted files before generating commit (git add -u)
  -A, --all-files        Stage all files including untracked (git add -A)
  -y, --yes              Commit automatically without interactive confirmation
      --amend            Amend the previous commit instead of creating a new one
  -d, --dry-run          Display the generated commit message without committing
  -m, --hint <HINT>      Provide guidance or intent to the AI
      --provider <PROV>  Override active provider (gemini, openai, anthropic, ollama, custom)
      --model <MODEL>    Override model name
  -h, --help             Print help
  -V, --version          Print version
```

### Common Workflows

#### Auto-stage and Commit:
```bash
qmit -a
```

#### Commit with guidance / hint:
```bash
qmit -m "breaking change: rename user_id to account_id"
```

#### Non-interactive (One-liner):
```bash
qmit -a -y
```

#### Preview commit message (Dry Run):
```bash
qmit -d
```

#### Using Local Ollama:
```bash
qmit --provider ollama --model llama3.2
```

---

## ⚙️ Configuration Management

Configuration is stored in `~/.config/qmit/config.toml`.

- **Interactive Setup**:
  ```bash
  qmit config setup
  ```
- **Show Active Config**:
  ```bash
  qmit config show
  ```
- **Set specific value**:
  ```bash
  qmit config set provider openai
  qmit config set openai.model gpt-4o
  qmit config set gemini.model gemini-2.0-flash
  ```
- **Print config path**:
  ```bash
  qmit config path
  ```

---

## 📄 License

MIT
