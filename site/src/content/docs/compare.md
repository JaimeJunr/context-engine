---
title: vs RTK / CodeGraph / Context Mode / QMD
description: Honest, detailed comparison against the four leading competitors. Where we win, where we still lose, with explicit roadmap.
---

This page is an honest audit. We list features both ways — wins **and** gaps with explicit roadmap items.

## Quick verdict by use case

| If your main use is… | Best fit |
|---|---|
| Rust + git + cargo + docker, AI agent context | **ctx** (single binary, all-in-one) |
| Pure command output compression, 100+ commands | RTK (deeper coverage, 53k stars) |
| Heavy call graph navigation in 19+ languages | CodeGraph (more languages, framework routing) |
| Session continuity across context compactions | Context Mode (unique feature) |
| Pure semantic search over docs, fully local LLM | QMD (more advanced search pipeline) |

`ctx` wins when you want **all of the above in one binary** rather than orchestrating four separate tools.

## Full feature matrix

| | **ctx** | [RTK](https://github.com/rtk-ai/rtk) | [CodeGraph](https://github.com/colbymchenry/codegraph) | [Context Mode](https://github.com/mksglu/context-mode) | [QMD](https://github.com/tobi/qmd) |
|---|:---:|:---:|:---:|:---:|:---:|
| **Stars** | — | 53k | 22k | 15.5k | 25.5k |
| **Language** | Rust | Rust | TypeScript | TypeScript | TypeScript |
| **License** | MIT | MIT | MIT | MIT | MIT |
| | | | | | |
| **Hook PreToolUse auto-rewrite** | ✅ | ✅ | ❌ | ❌ | ❌ |
| **Native MCP server** | ✅ (10 tools) | ❌ | ✅ (9 tools) | ✅ | ✅ |
| **Command output compression** | ✅ (17 families) | ✅ (100+ commands) | ❌ | ✅ | ❌ |
| **Curated repo map** | ✅ (BM25 + PageRank) | ❌ | ❌ | ❌ | ❌ |
| **Token budget binary search** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Local RAG over docs** | ✅ | ❌ | ❌ | partial (FTS5) | ✅ |
| **Call graph** | ✅ (7 langs) | ❌ | ✅ (19+ langs) | ❌ | ❌ |
| **Query-ranked graph results** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Token budget on graph output** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Session continuity** (PreCompact) | 🚧 | ❌ | ❌ | ✅ | ❌ |
| **Multi-agent installer** | 1 (Claude Code) | 13 | 5 | 15 | partial |
| **Single binary** | ✅ | ✅ | ❌ Node bundled | ❌ Node bundled | ❌ Node bundled |
| **100% local** | ✅ | ✅ | ✅ | ✅ | ✅ |

## Where ctx wins clearly

1. **Only tool with hook + MCP server + compression + RAG + repo map + call graph in one binary.** Everyone else specializes vertically.
2. **Only tool with query-ranked graph results.** CodeGraph returns a raw list; we rank by BM25(query) + popularity + symbol kind.
3. **Only tool with token budget on graph output.** CodeGraph can overflow context with large traces; we apply binary search like `ctx map`.
4. **Only tool with Personalized PageRank for repo map.** No competitor does ranked code-aware selection.

## Where ctx still loses

### Vs RTK (command compression)

- **100+ commands vs 17 families** — RTK covers ansible, sops, helm, gcloud, dotnet, swift-build, xcodebuild, mypy variants, prisma, basedpyright, etc.
- **13 agent installers vs 1** — RTK auto-configures Cursor, Copilot, Gemini, Windsurf, Cline, Hermes, etc.
- **Telemetry / `rtk gain`** — RTK has detailed savings dashboard.

**Plan:** add filters incrementally based on user demand. The pacote completo PR closed 70% of the gap (terraform, AWS by service, all major linters, log dedup, git ok-style).

### Vs CodeGraph (call graph)

- **19+ languages vs 7** — missing C#, PHP, Swift, Kotlin, Scala, Dart, Svelte, Vue, Lua, Liquid, Pascal/Delphi, Luau.
- **Dynamic dispatch / inheritance** — `interface → impl` resolution.
- **Framework-aware routing** — Django/Rails/Express/Spring URL → handler.
- **Live file watcher** — FSEvents/inotify. We re-index on demand.

**Plan:** add languages on demand (each is ~80 LOC). Watcher is ~200 LOC. Framework routing is one strategy per framework.

### Vs Context Mode (session continuity)

- **PreCompact + SessionStart hooks** — Context Mode persists session state across context compactions. Unique feature, no other competitor matches.

**Plan:** explicit roadmap item. ~300-500 LOC + design decisions on what state to capture.

### Vs QMD (RAG quality)

- **Reciprocal Rank Fusion (RRF)** — QMD combines BM25 + vector results with RRF.
- **Query expansion via LLM** — QMD expands the query before searching.
- **LLM reranker with logprobs** — QMD uses the model's confidence to rerank top results.

**Plan:** add RRF (~150 LOC), query expansion (~80 LOC), confidence-aware reranker.

## Roadmap priority

Based on the gaps above and likely impact:

1. **More agent installers** — Cursor, Codex, opencode (trait already in place)
2. **RRF in `ctx catalog`** — cheap, broad recall improvement
3. **Live file watcher for `ctx graph`** — fixes the "stale index" annoyance
4. **More languages in `ctx graph`** — start with C#, PHP, Kotlin
5. **Session continuity** — `ctx session save/restore`
6. **More `ctx exec` filters** — helm, gcloud, ansible, prisma based on demand

See [docs/competitors/](https://github.com/JaimeJunr/context-engine/tree/main/docs/competitors) on GitHub for the raw per-competitor analysis.

## Honest disclaimer

`ctx` is **pre-1.0** (currently v0.1.0). The competitors are mature and battle-tested with thousands of stars and millions of downloads. Choose accordingly:

- **For production use today**, especially in Cursor/Copilot/Windsurf workflows, RTK or CodeGraph have more installers and broader command coverage.
- **For Claude Code workflows that want everything integrated**, `ctx` is competitive and getting closer every release.
- **For experimenting with the future of agent context tooling**, all five are worth knowing.
