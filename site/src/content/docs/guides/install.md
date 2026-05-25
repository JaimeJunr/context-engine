---
title: Install
description: Install ctx-engine on Linux, macOS or Windows in one command.
---

## Quick install (no Rust required)

The installer detects your OS and architecture, downloads the right pre-built binary, and places `ctx` in your PATH.

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/JaimeJunr/context-engine/releases/latest/download/ctx-engine-installer.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/JaimeJunr/context-engine/releases/latest/download/ctx-engine-installer.ps1 | iex
```

## With Rust

If you already have Rust 1.70+ installed:

```bash
cargo install ctx-engine
```

The binary still installs as `ctx` (the crate is published as `ctx-engine` because `context-engine` was taken on crates.io).

## Configure your agent

```bash
ctx install --agent claude-code
```

This writes a `PreToolUse` hook and registers the MCP server in `~/.claude/settings.json`. Reopen your Claude Code session for the hook to take effect.

[Details on agent integration →](/context-engine/guides/agent-integration/)

## Optional: Ollama for semantic search

`ctx catalog` uses embeddings via an OpenAI-compatible endpoint. The easiest setup is local Ollama:

```bash
ollama serve
ollama pull nomic-embed-text   # default embedder
ollama pull llama3.2            # default reranker
```

Without Ollama, `ctx search` still works in lexical (BM25) mode.

## Supported platforms

| OS | Architecture |
|---|---|
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), aarch64 (Apple Silicon) |
| Windows | x86_64 |

## Uninstall

```bash
ctx uninstall --agent claude-code   # removes only what ctx installed
```

The `_installer: "ctx"` marker on each setting ensures we only remove our own entries — your other hooks and MCP servers stay intact.
