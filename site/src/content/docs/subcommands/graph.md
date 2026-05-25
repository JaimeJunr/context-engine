---
title: ctx graph
description: Resolved symbol graph for code navigation — callers, callees, trace, impact. Query-ranked results.
---

`ctx graph` builds and queries a symbol graph of your code. Answer questions that grep can't answer cheaply.

## Usage

```bash
# Index the project (once, then incremental)
ctx graph index --dirs src

# Who calls this function?
ctx graph callers run_proxy

# What does this function call?
ctx graph callees src/main.rs::execute

# What chain reaches this symbol?
ctx graph trace handle_request --depth 3

# What breaks if I change this?
ctx graph impact migrate_database

# Where is X defined?
ctx graph node CtxServer
```

## Unique features vs CodeGraph

| Feature | CodeGraph | **ctx** |
|---|:---:|:---:|
| Resolved call graph | ✅ | ✅ |
| **Query-ranked results** | ❌ raw list | ✅ BM25(query) + log(sites) + kind boost |
| **Token budget on output** | ❌ can overflow context | ✅ binary search like `ctx map` |
| **Site deduplication** | ❌ N entries | ✅ 1 entry + array of sites |
| Single Rust binary | ❌ (TypeScript) | ✅ |
| Co-exists with hooks + MCP + RAG + repo map | ❌ graph only | ✅ |

### Ranked example

```bash
ctx graph callers apply_pipeline --query "exec proxy" --max-tokens 800
```

Returns the top callers ranked by:
1. BM25 match between caller name/path and the query
2. Log of the number of call sites (popularity = relevance)
3. Boost for functions/methods over variables/constants

## Languages supported (7)

| Language | Extractor |
|---|---|
| TypeScript / TSX | ✅ |
| Python | ✅ |
| Ruby | ✅ |
| Go | ✅ |
| Rust | ✅ |
| Java | ✅ |
| Groovy | ✅ |

Roadmap: C#, PHP, Swift, Kotlin, Scala, Dart, Vue, Svelte, Lua.

## Storage

The graph is persisted in SQLite at `~/.cache/context_engine/graph.db`. Schema:

```sql
symbols (id, name, qualified, kind, file, line, language)
calls   (id, caller_qualified, callee_name, file, line)
imports (id, file, module, alias)
```

Re-indexing a file clears its old entries first — fully idempotent.

## Via MCP

Six MCP tools are exposed for agents:

- `ctx_graph_index` — populate the graph
- `ctx_callers` — who calls X?
- `ctx_callees` — what does X call?
- `ctx_trace` — caller chain up to depth
- `ctx_impact` — affected code on change
- `ctx_node` — definition location(s)
