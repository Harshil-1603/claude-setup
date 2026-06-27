# Claude Engineering OS

A reusable engineering platform that installs globally and improves every Claude Code session.

## What It Does

Claude Engineering OS transforms Claude Code into an opinionated software engineering operating system with:

- **Global configuration** — Consistent settings across all projects
- **Skill-first execution** — Search, install, and use reusable skills
- **Workflow engine** — YAML-defined state machines for common tasks
- **Memory** — Persistent context across sessions (SQLite FTS5)
- **Verification** — Automated quality checks (lint, test, build)
- **Git automation** — Structured commits and branch management

## Installation

```bash
# From source
cargo install --path .

# Or download a release binary
curl -sSL https://github.com/yourname/claude-engineering-os/releases/latest/download/claude-eng-linux -o claude-eng
chmod +x claude-eng
sudo mv claude-eng /usr/local/bin/
```

## Quick Start

```bash
# Install into ~/.claude/
claude-eng install

# List installed skills
claude-eng skill list

# Search for a skill
claude-eng skill search testing

# Install a skill from GitHub
claude-eng skill add owner/repo

# Remove a skill
claude-eng skill remove skill-name

# List available workflows
claude-eng workflow list

# Start or resume a workflow
claude-eng workflow run feature-dev

# Store a memory
claude-eng memory store --kind decision "Use JWT for auth"

# Recall memories
claude-eng memory recall "auth"

# Run verification
claude-eng verify

# Create a conventional commit
claude-eng commit "feat: add login page"
```

## Commands

### Config & Skills

| Command | Description |
|---------|-------------|
| `claude-eng install` | Install or update configuration |
| `claude-eng skill list` | List installed skills |
| `claude-eng skill search <query>` | Search installed skills |
| `claude-eng skill add <name>` | Install a skill from registry |
| `claude-eng skill remove <name>` | Remove an installed skill |

### Workflows

| Command | Description |
|---------|-------------|
| `claude-eng workflow list` | List available workflows |
| `claude-eng workflow status <name>` | Show workflow progress |
| `claude-eng workflow run <name>` | Start or resume a workflow |

### Memory

| Command | Description |
|---------|-------------|
| `claude-eng memory store --kind <kind> <content>` | Store a memory entry |
| `claude-eng memory recall <query>` | Search memories via FTS5 |
| `claude-eng memory list` | List all memories |
| `claude-eng memory delete <id>` | Delete a memory |
| `claude-eng memory context` | Generate context markdown |

### Verification

| Command | Description |
|---------|-------------|
| `claude-eng verify` | Run lint, test, build pipeline |
| `claude-eng verify --stages lint,test` | Run specific stages only |
| `claude-eng verify --json` | Machine-readable output |

### Git

| Command | Description |
|---------|-------------|
| `claude-eng commit "feat: ..."` | Stage all + conventional commit |
| `claude-eng branch <name>` | Create a feature branch |
| `claude-eng review` | Generate PR description |
| `claude-eng log --count 10` | Pretty git log with Co-Authored-By |

## Built-in Skills

| Skill | Description |
|-------|-------------|
| `brainstorming` | Explore intent before implementing |
| `tdd` | Test-driven development workflow |
| `systematic-debugging` | Structured approach to fixing bugs |
| `verification` | Quality check pipeline |
| `code-review` | Structured code review |

## Built-in Workflows

| Workflow | Description |
|----------|-------------|
| `feature-development` | End-to-end feature development with 6 states |
| `bug-fix` | Structured bug fixing with 5 states |
| `refactor` | Safe refactoring with verification at each step |

## Verification Auto-Detection

The `verify` command auto-detects project type and runs appropriate commands:

| Project | Lint | Test | Build |
|---------|------|------|-------|
| Rust | `cargo fmt --check` | `cargo test` | `cargo build` |
| Node.js | `npx eslint .` | `npm test` | `npm run build` |
| Python | `ruff check .` | `pytest` | `python -m build` |

Custom commands can be set in `claude-eng.yaml` at project root.

## Configuration

Claude Engineering OS generates and manages:

- `~/.claude/CLAUDE.md` — Main instructions (owned by claude-eng)
- `~/.claude/CLAUDE.local.md` — Your personal overrides
- `~/.claude/settings.json` — Claude Code settings (merged additively)
- `~/.claude/skills/` — Installed skills
- `~/.claude/backups/` — Config backups before changes
- `~/.claude/memory.db` — SQLite database for persistent memory

## License

MIT
