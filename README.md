# Claude Engineering OS

A reusable engineering platform that installs globally and improves every Claude Code session.

## What It Does

Claude Engineering OS transforms Claude Code into an opinionated software engineering operating system with:

- **Global configuration** — Consistent settings across all projects
- **Skill-first execution** — Search, install, and use reusable skills
- **Workflow engine** — YAML-defined state machines for common tasks
- **Memory** — Persistent context across sessions
- **Verification** — Automated quality checks
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
```

## Commands

| Command | Description |
|---------|-------------|
| `claude-eng install` | Install or update configuration |
| `claude-eng skill list` | List installed skills |
| `claude-eng skill search <query>` | Search installed skills |
| `claude-eng skill add <name>` | Install a skill from registry |
| `claude-eng skill remove <name>` | Remove an installed skill |
| `claude-eng workflow list` | List available workflows |
| `claude-eng workflow status <name>` | Show workflow progress |
| `claude-eng workflow run <name>` | Start or resume a workflow |

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

## Configuration

Claude Engineering OS generates and manages:

- `~/.claude/CLAUDE.md` — Main instructions (owned by claude-eng)
- `~/.claude/CLAUDE.local.md` — Your personal overrides
- `~/.claude/settings.json` — Claude Code settings (merged additively)
- `~/.claude/skills/` — Installed skills
- `~/.claude/backups/` — Config backups before changes

## License

MIT
