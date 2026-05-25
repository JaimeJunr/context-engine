---
title: ctx map
description: Generate a curated repo map ranked by BM25 + Personalized PageRank, capped by token budget.
---

`ctx map` extracts signatures (functions, classes, types) from your code via Tree-Sitter, ranks them by relevance to the task title, and returns a compact text or JSON output that fits a token budget.

## Usage

```bash
ctx map --title "CAP-123: Add OAuth login" --dirs "src/auth,src/models" --max-tokens 4000
```

## Options

| Flag | Default | Description |
|---|---|---|
| `--title <str>` | required | Task description for BM25 ranking |
| `--dirs <csv>` | required | Comma-separated directories to scan |
| `--top <n>` | `0` | Fixed file count (overrides `--max-tokens`) |
| `--max-tokens <n>` | `4096` | Token budget (1 token ≈ 4 chars) |
| `--format <text\|json>` | `text` | Output format |
| `--seeds <csv>` | — | Seed dirs to boost in Personalized PageRank |
| `--max-depth <n>` | `15` | Max scan depth |
| `--no-cache` | — | Force re-parse (ignores SQLite cache) |

## How it works

```
Scanner → Extractor (Tree-Sitter) → Cache → BM25 + PageRank → Budget → Output
```

1. **Scanner** discovers files respecting `.gitignore`
2. **Extractor** parses each file and extracts signatures (4 languages: TS, Py, Rb, Groovy)
3. **Cache** stores extractions in `~/.cache/context_engine/` keyed by SHA256 — invalidated automatically when file changes
4. **BM25** scores each file against the task title
5. **PageRank** (optional, via `--seeds`) boosts files in seed dirs and their dependents
6. **Budget** uses binary search to maximize files while respecting `--max-tokens`
7. **Output** formats text or JSON

## Example output

```text
src/auth/oauth.rs:
  pub fn login(provider: &str, code: &str) -> Result<Session, AuthError>
  pub fn refresh(token: &str) -> Result<Session, AuthError>
  pub struct Session { token: String, user_id: u64, expires_at: i64 }

src/models/user.rs:
  pub struct User { id: u64, name: String, email: String }
  impl User { fn find_by_email(email: &str) -> Option<User> }
```

## When to use

- **Drop into LLM prompt** when you need to give the model awareness of code structure without sending whole files
- **Compose with `ctx search`** to combine code signatures + relevant docs in one prompt
- **Available as MCP tool** `ctx_map` — agents call it directly
