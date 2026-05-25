---
title: Quick Start
description: First repo map, first search, first call graph in 5 minutes.
---

## 1. Install (30 seconds)

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/JaimeJunr/context-engine/releases/latest/download/ctx-engine-installer.sh | sh
ctx install --agent claude-code
```

[Full install guide →](/context-engine/guides/install/)

## 2. Try `ctx map` (no setup needed)

Generate a curated repo map ranked by relevance to a task:

```bash
cd your-rust-or-typescript-project
ctx map --title "Add OAuth login flow" --dirs src --max-tokens 4000
```

The output is text-format extracted signatures (no full file bodies) ranked by BM25 against the title and weighted by Personalized PageRank. Drop it into your LLM prompt.

JSON output for tool composition:

```bash
ctx map --title "..." --dirs src --format json
```

## 3. Try `ctx exec` (instant compression)

Run any command through the proxy and see the filtered output:

```bash
ctx exec git status      # only modified files, no boilerplate
ctx exec cargo test      # only failures preserved
ctx exec kubectl logs my-pod   # logs deduplicated (×N grouping)
```

Token-saving report:

```bash
ctx exec report
```

## 4. Try `ctx graph` (call graph)

Build the symbol graph for your project, then query it:

```bash
ctx graph index --dirs src
ctx graph callers some_function
ctx graph trace handle_request --depth 3
ctx graph impact migrate_database
```

Results are ranked by relevance to the query and respect a token budget — unique features vs CodeGraph.

## 5. Try `ctx catalog` (semantic search)

Index your documentation and search it:

```bash
ctx add my-project --source ./docs --include "**/*.md"
ctx index my-project --with-embed   # requires Ollama, see install guide
ctx search my-project "how does authentication work?"
```

## 6. Use everything via MCP

After `ctx install --agent claude-code`, ten MCP tools are available to Claude:

| Tool | Function |
|---|---|
| `ctx_exec` | Run command with compression |
| `ctx_search` | Semantic search in catalog |
| `ctx_map` | Generate repo map |
| `ctx_list` | List catalogs |
| `ctx_graph_index` | Index project for graph |
| `ctx_callers` | Who calls this symbol? |
| `ctx_callees` | What does this call? |
| `ctx_trace` | Caller chain up to depth |
| `ctx_impact` | What breaks if I change X? |
| `ctx_node` | Where is X defined? |

You can call them manually too:

```bash
ctx mcp tools    # list available tools
ctx mcp serve    # start stdio server (used internally by the agent)
```
