# Claude Engineering OS — Architecture Specification

**Date:** 2026-06-27
**Version:** 1.0
**Status:** Draft

---

## 1. Overview

Claude Engineering OS is a **reusable, globally-installed engineering platform** built as a single Rust binary. It transforms Claude Code into an opinionated software engineering operating system by:

- Generating and managing global `~/.claude/` configuration
- Providing a skill system compatible with the existing `skills.sh` ecosystem
- Running YAML-defined workflows with state tracking and resume support
- Persisting project memory across sessions (SQLite + FTS5)
- Automating git operations with conventional commits
- Running verification pipelines (lint → test → build → review)
- Generating hook configurations for Claude Code events

The binary **does not replace** Claude Code — it enhances it by producing optimal configuration that Claude reads.

---

## 2. Core Philosophy

Every task follows this enforced workflow:

```
Understand
→ Plan
→ Search for existing skills
→ Use existing skill if found
→ Search external skill registry if not found
→ Install skill if available
→ Create new reusable skill only if absolutely necessary
→ Implement
→ Verify
→ Review
→ Report progress
→ Wait for user approval
→ Commit
→ Continue
```

Claude should never immediately start coding.

---

## 3. Target Users

**Individual developers** who use Claude Code across multiple projects and want:
- Consistent engineering practices everywhere
- Persistent memory across sessions
- Reusable workflows and skills
- Fast, zero-dependency tooling

---

## 4. Repository Structure

```
claude-engineering-os/
├── Cargo.toml
├── README.md
├── LICENSE
│
├── src/
│   ├── main.rs                 # CLI entry point (clap)
│   ├── lib.rs                  # Public API re-exports
│   │
│   ├── cli/                    # Command definitions
│   │   ├── mod.rs
│   │   ├── install.rs          # `claude-eng install`
│   │   ├── skill.rs            # `claude-eng skill {add,remove,search,list}`
│   │   ├── workflow.rs         # `claude-eng workflow {run,list,status}`
│   │   ├── config.rs           # `claude-eng config {show,set,reset}`
│   │   └── memory.rs           # `claude-eng memory {store,recall,search}`
│   │
│   ├── config/                 # Configuration management
│   │   ├── mod.rs
│   │   ├── paths.rs            # ~/.claude path resolution
│   │   ├── claude_md.rs        # CLAUDE.md generation & patching
│   │   ├── settings.rs         # settings.json management
│   │   └── templates.rs        # Config templates
│   │
│   ├── skills/                 # Skill system
│   │   ├── mod.rs
│   │   ├── manifest.rs         # SKILL.md parser
│   │   ├── registry.rs         # Remote skill registry client
│   │   ├── installer.rs        # Skill install/uninstall
│   │   ├── search.rs           # Local + registry search
│   │   └── builtin/            # Built-in skills (embedded)
│   │       ├── mod.rs
│   │       ├── brainstorming.md
│   │       ├── tdd.md
│   │       └── ...
│   │
│   ├── workflows/              # Workflow engine
│   │   ├── mod.rs
│   │   ├── engine.rs           # State machine runner
│   │   ├── definition.rs       # YAML parser + validator
│   │   ├── tracker.rs          # Progress tracking
│   │   └── builtin/            # Built-in workflows
│   │
│   ├── memory/                 # Memory subsystem
│   │   ├── mod.rs
│   │   ├── store.rs            # SQLite storage
│   │   ├── context.rs          # Context injection builder
│   │   └── search.rs           # FTS search
│   │
│   ├── hooks/                  # Hook system
│   │   ├── mod.rs
│   │   ├── runner.rs           # Hook execution engine
│   │   └── definitions.rs      # Hook registry
│   │
│   ├── verification/           # Verification pipeline
│   │   ├── mod.rs
│   │   ├── pipeline.rs         # Stage runner
│   │   └── reporters.rs        # Output formatting
│   │
│   └── git/                    # Git automation
│       ├── mod.rs
│       ├── commit.rs           # Structured commits
│       ├── branch.rs           # Branch management
│       └── review.rs           # Review helpers
│
├── skills/                     # Skill source definitions
│   ├── brainstorming/
│   │   ├── SKILL.md
│   │   └── README.md
│   └── ...
│
├── workflows/                  # Workflow definitions
│   ├── feature-dev.yaml
│   ├── bug-fix.yaml
│   └── refactor.yaml
│
├── templates/                  # Config templates
│   ├── CLAUDE.md.template
│   └── settings.json.template
│
├── docs/
│   ├── architecture.md
│   ├── user-guide.md
│   └── developer-guide.md
│
└── tests/
    ├── integration/
    └── unit/
```

---

## 5. High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│              claude-eng (Rust binary)                │
├──────────┬──────────┬───────────┬───────────────────┤
│  Config  │  Skills  │ Workflows │     Memory        │
│  Engine  │  System  │  Engine   │     Store         │
├──────────┴──────────┴───────────┴───────────────────┤
│              Shared Core (lib.rs)                    │
│         paths / errors / logging / serialization     │
├─────────────────────────────────────────────────────┤
│           ~/.claude/ (Claude Code home)              │
│   CLAUDE.md │ settings.json │ skills/ │ memory.db   │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│              Claude Code CLI (external)              │
│   Reads config → Runs with enhanced instructions    │
└─────────────────────────────────────────────────────┘
```

---

## 6. Module Descriptions

### 6.1 Configuration Engine

**Responsibilities:**
- Resolve `~/.claude/` paths (Linux, macOS, Windows)
- Generate/update `CLAUDE.md` with workflow instructions
- Manage `settings.json` (hooks, env vars, MCP servers)
- Validate config before writing (never corrupt user's setup)
- Backup before overwriting

**Data flow:**

```
User runs: claude-eng install
                │
                ▼
    ┌───────────────────────┐
    │  Read current config  │  ← ~/.claude/settings.json
    │  Read current CLAUDE  │  ← ~/.claude/CLAUDE.md
    └───────────┬───────────┘
                │
                ▼
    ┌───────────────────────┐
    │  Apply template       │  ← templates/CLAUDE.md.template
    │  Merge settings       │  ← templates/settings.json.template
    └───────────┬───────────┘
                │
                ▼
    ┌───────────────────────┐
    │  Backup originals     │  → ~/.claude/backups/
    │  Write new config     │  → ~/.claude/CLAUDE.md
    │                       │  → ~/.claude/settings.json
    └───────────┬───────────┘
                │
                ▼
    ┌───────────────────────┐
    │  Install skills       │  → ~/.claude/skills/
    │  Initialize memory    │  → ~/.claude/memory.db
    └───────────────────────┘
```

**Design decisions:**
- Config generation is **idempotent** — running install multiple times produces the same result
- CLAUDE.md is a **generated file** — the tool owns it; users customize via `CLAUDE.local.md`
- Settings merge is **additive** — never removes user's existing hooks/MCP configs
- Atomic writes: write to temp file, then rename

---

### 6.2 Skill System

Skills are Markdown files with a frontmatter manifest.

**SKILL.md format:**

```markdown
---
name: feature-development
description: Full workflow for building new features
version: 1.0.0
triggers:
  - "implement"
  - "build"
  - "add feature"
dependencies: []
---

# Feature Development

## When to Use
...

## Steps
...
```

**Data flow:**

```
User: "Build a login page"
            │
            ▼
    ┌───────────────────┐
    │ Keyword matching  │  ← skills/*/SKILL.md (frontmatter triggers)
    │ + fuzzy search    │  ← skills by name/description
    └────────┬──────────┘
             │
      ┌──────┴──────┐
      │ Found?      │
      ├─── YES ────►│ Load skill content → Inject into Claude context
      │             │
      ├─── NO ─────►│ Search registry (skills.sh API)
      │             │
      │    Found?   │
      │    ├── YES ─► Download → Install locally → Load
      │    ├── NO ──► Create new skill (user approval required)
      └─────────────┘
```

**Design decisions:**
- Skills are **Markdown only** — no code execution, no security risk
- Registry search uses `skills.sh` API (existing ecosystem)
- Local skills take priority over registry results
- Skill install is a `git clone` + symlink into `~/.claude/skills/`
- Skill metadata is cached locally (refresh on `claude-eng skill update`)

---

### 6.3 Workflow Engine

Workflows are YAML-defined state machines.

**Workflow definition format:**

```yaml
name: feature-development
version: 1.0.0
description: End-to-end feature development workflow

states:
  - id: understand
    name: "Understand Requirements"
    actions:
      - ask_clarifying_questions
      - search_existing_code
    next: plan

  - id: plan
    name: "Create Implementation Plan"
    actions:
      - identify_files_to_change
      - consider_edge_cases
      - present_plan_for_approval
    next: implement

  - id: implement
    name: "Implement Changes"
    actions:
      - write_code
      - add_tests
      - update_docs
    next: verify

  - id: verify
    name: "Verify Implementation"
    actions:
      - run_tests
      - check_linting
      - manual_review
    next: report

  - id: report
    name: "Report Progress"
    actions:
      - summarize_changes
      - present_to_user
    next: done

  - id: done
    name: "Complete"
    actions:
      - create_commit
      - update_memory
```

**Data flow:**

```
User: "Implement the auth system"
            │
            ▼
    ┌───────────────────┐
    │ Match workflow     │  ← workflows/*.yaml
    │ to request         │
    └────────┬──────────┘
             │
             ▼
    ┌───────────────────┐
    │ Load workflow def  │
    │ Initialize state   │
    └────────┬──────────┘
             │
             ▼
    ┌───────────────────┐
    │ Execute actions    │◄──┐
    │ for current state  │   │
    └────────┬──────────┘   │
             │              │
             ▼              │
    ┌───────────────────┐   │
    │ Check completion   │   │
    │ criteria           │   │
    └────────┬──────────┘   │
             │              │
      ┌──────┴──────┐      │
      │ Complete?   │      │
      ├─── YES ────►│      │
      │  Advance to │      │
      │  next state ├──────┘
      ├─── NO ─────►│ Continue current state
      └─────────────┘
```

**Design decisions:**
- Workflows are **declarative YAML** — easy to write, diff, share
- State machine is explicit — no hidden control flow
- Every state transition requires user approval (per core philosophy)
- Workflows can reference skills (e.g., `use_skill: tdd` in a state)
- Progress is persisted to `~/.claude/workflows/<id>.json` (resume support)
- Built-in workflows ship with the binary; users can create custom ones

---

### 6.4 Memory Subsystem

Memory persists context across Claude Code sessions.

**Data model:**

```sql
CREATE TABLE memories (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL,          -- 'decision', 'progress', 'context', 'error'
    project     TEXT,                   -- project path or NULL for global
    content     TEXT NOT NULL,
    tags        TEXT,                   -- comma-separated
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    accessed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at  DATETIME               -- NULL = never
);

CREATE VIRTUAL TABLE memories_fts USING fts5(content, tags);
```

**Data flow:**

```
After task completion:
            │
            ▼
    ┌───────────────────┐
    │ Extract key info  │
    └────────┬──────────┘
             │
             ▼
    ┌───────────────────┐
    │ Classify memory   │
    │ Set expiry        │
    └────────┬──────────┘
             │
             ▼
    ┌───────────────────┐
    │ Store in SQLite   │
    │ Update FTS index  │
    └───────────────────┘

Next session:
            │
            ▼
    ┌───────────────────┐
    │ Query relevant    │
    │ memories          │
    └────────┬──────────┘
             │
             ▼
    ┌───────────────────┐
    │ Build context     │
    │ injection         │
    └───────────────────┘
```

**Design decisions:**
- SQLite with FTS5 for fast full-text search
- Memories are **project-scoped** by default, global optional
- Automatic expiry for temporary context
- Memory search is explicit — Claude calls `claude-eng memory recall <query>`
- All data stays local, no telemetry

---

### 6.5 Hook System

Hooks are pre/post handlers that run around Claude Code events.

**Supported hooks:**

```yaml
hooks:
  on_session_start:
    - inject_memory_context
    - load_project_state

  on_task_complete:
    - extract_memories
    - update_progress

  on_error:
    - log_error
    - suggest_fixes

  on_before_commit:
    - run_tests
    - lint_check
```

**Design decisions:**
- Hooks are defined in `settings.json` (Claude Code's native format)
- The binary **generates** hook configs, doesn't replace Claude's hook system
- Hooks can call back into the binary (e.g., `claude-eng memory store ...`)
- Hook execution is sequential, with error handling per-hook

---

### 6.6 Verification Pipeline

Verification runs after implementation to ensure quality.

**Pipeline stages:**

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  Lint   │───►│  Test   │───►│  Build  │───►│  Review │
│  Check  │    │  Suite  │    │  Check  │    │  Report │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
     │              │              │              │
     ▼              ▼              ▼              ▼
  [PASS/FAIL]  [PASS/FAIL]  [PASS/FAIL]   [SUMMARY]
```

**Design decisions:**
- Verification is **configurable per project** (not all projects have tests)
- Pipeline stages are defined in `claude-eng.yaml` at project root
- Failures stop the pipeline (fail-fast)
- Results are stored in memory for future reference
- Verification can be run standalone: `claude-eng verify`

---

### 6.7 Git Automation

Automates structured commits, branching, and review workflows.

**Key features:**
- **Conventional commits:** `claude-eng commit "feat: add auth system"`
- **Auto-branching:** Create feature branches before implementation
- **PR helpers:** Generate PR descriptions from changes
- **Review integration:** Structured code review with findings

**Data flow:**

```
claude-eng commit "message"
            │
            ▼
    ┌───────────────────┐
    │ Stage all changes  │
    │ Format commit msg  │
    │ Add Co-Authored-By │
    └────────┬──────────┘
             │
             ▼
    ┌───────────────────┐
    │ Run pre-commit    │
    │ hooks             │
    └────────┬──────────┘
             │
      ┌──────┴──────┐
      │ Hooks pass?  │
      ├─── YES ────►│ Create commit
      ├─── NO ─────►│ Show errors, abort
      └─────────────┘
```

---

## 7. Full System Data Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                         USER INPUT                               │
│                    "Build a login page"                          │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│                    CLAUDE-ENG CLI                                │
│                                                                  │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐│
│  │   Config   │  │   Skills   │  │  Workflows │  │   Memory   ││
│  │   Engine   │  │   System   │  │   Engine   │  │   Store    ││
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘│
│        │               │               │               │        │
│        └───────────────┴───────┬───────┴───────────────┘        │
│                                │                                │
│                                ▼                                │
│                    ┌───────────────────────┐                    │
│                    │    CLAUDE.MD          │                    │
│                    │  (Generated config)   │                    │
│                    │                       │                    │
│                    │  - Workflow state     │                    │
│                    │  - Active skill       │                    │
│                    │  - Memory context     │                    │
│                    │  - Verification rules │                    │
│                    └───────────┬───────────┘                    │
└────────────────────────────────┼────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                    CLAUDE CODE                                    │
│                                                                  │
│  Reads CLAUDE.md → Follows workflow → Uses skills → Reports     │
│                                                                  │
│  Actions:                                                        │
│  - Ask questions     - Write code        - Run tests            │
│  - Search skills     - Create commits    - Update memory        │
└──────────────────────────────────────────────────────────────────┘
```

---

## 8. Interfaces Between Modules

| Interface | From | To | Protocol |
|-----------|------|----|----------|
| `Config::generate()` | CLI install | Config Engine | Rust function call |
| `SkillRegistry::search()` | Skill System | skills.sh API | HTTPS REST |
| `SkillInstaller::install()` | Skill System | Config Engine | Rust function call |
| `WorkflowEngine::run()` | CLI workflow | Workflow Engine | Rust function call |
| `WorkflowEngine::save_state()` | Workflow Engine | Memory Store | SQLite write |
| `MemoryStore::store()` | Hook System | Memory Store | SQLite write |
| `MemoryStore::recall()` | Config Engine | Memory Store | SQLite read |
| `VerificationPipeline::run()` | Workflow Engine | Verification | Rust function call |
| `GitAutomation::commit()` | Workflow Engine | Git Automation | `git2` crate |
| `ClaudeMd::patch()` | All modules | Config Engine | Rust function call |

---

## 9. Technical Stack

| Component | Library | Purpose |
|-----------|---------|---------|
| CLI | `clap` | Command-line argument parsing |
| Config | `serde` + `serde_yaml` | Serialization/deserialization |
| Storage | `rusqlite` | SQLite database |
| Search | `rusqlite` FTS5 | Full-text search |
| HTTP | `reqwest` | Registry API calls |
| Git | `git2` (libgit2) | Git operations |
| Paths | `dirs` | Cross-platform path resolution |
| Logging | `tracing` | Structured logging |
| Errors | `anyhow` + `thiserror` | Error handling |
| Testing | `assert_cmd` + `tempfile` | CLI integration tests |

---

## 10. Phase Breakdown

### Phase 1: Core + Skills (MVP)
**Duration:** ~2-3 weeks
**Goal:** Installable binary that manages config and skills

| Module | Tasks | Deliverable |
|--------|-------|-------------|
| CLI | Clap setup, command routing | `claude-eng` binary |
| Config | Path resolution, templates, install | `claude-eng install` |
| Skills | Manifest parser, local registry, install/uninstall | `claude-eng skill {add,remove,list}` |
| Built-in skills | 3-5 core skills | Skills available immediately |

### Phase 2: Workflow Engine
**Duration:** ~2 weeks
**Goal:** YAML workflows with state tracking

| Module | Tasks | Deliverable |
|--------|-------|-------------|
| Workflow engine | YAML parser, state machine, persistence | `claude-eng workflow run <name>` |
| Built-in workflows | feature-dev, bug-fix, refactor | 3 starter workflows |
| Progress tracking | JSON persistence, resume | `claude-eng workflow status` |

### Phase 3: Memory + Verification
**Duration:** ~2 weeks
**Goal:** SQLite memory store and verification pipelines

| Module | Tasks | Deliverable |
|--------|-------|-------------|
| Memory | SQLite setup, FTS, CRUD operations | `claude-eng memory {store,recall,search}` |
| Verification | Pipeline runner, stage definitions | `claude-eng verify` |
| Hook integration | Generate hook configs | Hooks work in Claude Code |

### Phase 4: Git Automation + Polish
**Duration:** ~2 weeks
**Goal:** Structured commits, PR helpers, error handling

| Module | Tasks | Deliverable |
|--------|-------|-------------|
| Git | Conventional commits, branching | `claude-eng commit` |
| Review | PR description generation | `claude-eng review` |
| Error handling | Graceful errors, recovery | Production-ready UX |
| Documentation | User guide, examples | Complete docs |

---

## 11. Design Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Fast startup, single binary, zero runtime deps |
| Config format | YAML + Markdown | Human-readable, diffable, compatible with ecosystem |
| Memory store | SQLite + FTS5 | Fast, single-file, no server, built-in search |
| Skills format | Markdown (SKILL.md) | Compatible with skills.sh ecosystem |
| Installation | Single binary | No Rust toolchain needed on user machines |
| Config ownership | Binary generates, Claude reads | Clean separation, idempotent installs |
| Local overrides | `CLAUDE.local.md` | Users can customize without breaking generated config |
| Registry | skills.sh API | Existing ecosystem, no new infrastructure |

---

## 12. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Claude Code API changes | Config format breaks | Pin to known version, version checks |
| Skill ecosystem fragmentation | Incompatible skills | Strict SKILL.md validation |
| SQLite compilation | Build complexity | Use `rusqlite` with bundled feature |
| Windows compatibility | Path handling | Use `dirs` crate, test on all platforms |
| Config corruption | User's setup breaks | Atomic writes, automatic backups |
| Over-engineering | Slow delivery | Phase 1 is minimal, iterate |

---

## 13. Milestones

| Milestone | Target | Criteria |
|-----------|--------|----------|
| M1: Bootable CLI | Week 1 | Binary runs, `--help` works, `install` generates config |
| M2: Skill system | Week 2 | Can add/remove/list/search skills |
| M3: First workflow | Week 3 | Can run a YAML workflow end-to-end |
| M4: Memory store | Week 4 | Can store/recall memories across sessions |
| M5: Verification | Week 5 | Can run verification pipeline |
| M6: Git automation | Week 6 | Can create structured commits |
| M7: Beta release | Week 7 | All features working, docs complete |
| M8: v1.0 | Week 8 | Production-ready, tested, documented |

---

## 14. Acceptance Criteria

### Phase 1 (MVP)
- [ ] `claude-eng install` generates valid `~/.claude/CLAUDE.md`
- [ ] `claude-eng install` merges settings without corrupting existing config
- [ ] `claude-eng skill list` shows all installed skills
- [ ] `claude-eng skill add <name>` installs a skill from registry
- [ ] `claude-eng skill search <query>` finds skills locally and in registry
- [ ] Built-in skills are available after install
- [ ] Config generation is idempotent (running twice produces same result)
- [ ] Atomic writes prevent config corruption
- [ ] Backups are created before any config changes

### Phase 2
- [ ] `claude-eng workflow run feature-dev` starts a workflow
- [ ] State is persisted and can be resumed after interruption
- [ ] Workflow progress is displayed to user
- [ ] State transitions require user approval
- [ ] Built-in workflows work out of the box

### Phase 3
- [ ] `claude-eng memory store` saves to SQLite
- [ ] `claude-eng memory recall <query>` returns relevant memories
- [ ] `claude-eng verify` runs the verification pipeline
- [ ] Verification results are stored in memory
- [ ] Hook configs are generated correctly

### Phase 4
- [ ] `claude-eng commit` creates conventional commits
- [ ] PR descriptions are generated from changes
- [ ] Error messages are helpful and actionable
- [ ] Documentation is complete and accurate

---

*This document is the authoritative architecture specification for Claude Engineering OS. All implementation decisions should reference this document.*
