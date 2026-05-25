---
title: ctx catalog
description: Local RAG search over your documentation — semantic with Ollama or lexical without.
---

`ctx catalog` indexes documentation, generates embeddings via an OpenAI-compatible endpoint, and answers natural-language queries. 100% local.

## Usage

```bash
# Register a documentation collection
ctx add my-project --source ./docs --include "**/*.md" --exclude "**/node_modules/**"

# Index documents + generate embeddings (requires Ollama)
ctx index my-project --with-embed

# Search by intent
ctx search my-project "how does the auth pipeline work?"
```

## Subcommands

| Command | Function |
|---|---|
| `ctx add <name>` | Register a new collection (sources, include/exclude patterns, models) |
| `ctx index <name>` | Index documents (detects new/modified) |
| `ctx embed <name>` | Generate embeddings for pending chunks |
| `ctx search <name> <query>` | Semantic + lexical search |
| `ctx list` | List all collections |
| `ctx status <name>` | Show collection stats |
| `ctx compact <name>` | Optimize storage |
| `ctx bootstrap --path <dir>` | One-shot: add + index |

## Search modes

The query prefix selects the mode:

```bash
ctx search my-project "exact: middleware chain"          # lexical only
ctx search my-project "conceptual: auth flow"            # semantic emphasis
ctx search my-project "expanded: how do hooks work?"     # query expansion (planned)
ctx search my-project "natural query"                    # auto-detect (default)
```

## Configuration

Global (`~/.ctx/config.toml`):

```toml
[llm]
endpoint = "http://localhost:11434"
embedder = "nomic-embed-text"
reranker = "llama3.2"
```

Per-collection override (set during `ctx add`):

```bash
ctx add my-project \
  --source ./docs \
  --embedder-model bge-base-en \
  --reranker-model llama3.2 \
  --llm-endpoint http://localhost:11434
```

## Without Ollama

`ctx search` falls back to lexical (BM25) mode automatically. Useful for environments where you can't run a local LLM but still want semantic-ish search via term matching.
