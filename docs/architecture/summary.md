# Resumo Técnico

## Em Uma Linha

`ctx` = Scanner (FS) → Extractor (Tree-Sitter) → Ranker (BM25+PPR) → Budget-aware Output

---

## Os 3 Pipelines

### Pipeline 1: `map` — Repository Mapping
```
File System
    ↓ (respecting .gitignore)
Scanner (rayon parallel)
    ↓
Extractor (Tree-Sitter for each language)
    ↓
Cache (SQLite, validated by SHA256)
    ↓
Ranker (BM25 + Personalized PageRank)
    ↓
Budget Filter (binary search for max files under token limit)
    ↓
Text/JSON Output
```

**Modules:** `scanner.rs`, `extractors/`, `cache.rs`, `ranking/`, `output.rs`

### Pipeline 2: `catalog` — Semantic Search (RAG)
```
Documents (Files on disk)
    ↓
Indexer (Scanner + Parser)
    ↓
Chunker (Split into semantic chunks)
    ↓
Embedder (Call Ollama via HTTP)
    ↓
Store (SQLite with vector extension)
    ↓
Searcher (Cosine similarity)
    ↓
Reranker (LLM-based re-ranking)
    ↓
Results (ranked by relevance)
```

**Modules:** `catalog/{indexer,chunker,embedder,store,searcher,reranker}.rs`

### Pipeline 3: `exec` — Output Compression
```
Command (e.g., cargo test)
    ↓
Capture stdout/stderr
    ↓
Filter Pipeline (8 stages: summarize, truncate, colorize, etc)
    ↓
Metrics (track token savings)
    ↓
Output
```

**Modules:** `exec/{pipeline,types,metrics}.rs`

---

## Arquitetura de Código

```
src/
├── main.rs                  # CLI (clap)
├── lib.rs                   # Orquestração do pipeline `map`
├── extractors/              # Language-specific parsing
│   ├── mod.rs               # Trait Extractor + dispatch
│   ├── typescript.rs
│   ├── python.rs
│   ├── ruby.rs
│   └── groovy.rs
├── ranking/                 # BM25 + PageRank scoring
│   ├── mod.rs
│   ├── bm25.rs
│   ├── pagerank.rs
│   └── budget.rs
├── catalog/                 # RAG local
│   ├── mod.rs               # API pública
│   ├── indexer.rs
│   ├── chunker.rs
│   ├── embedder.rs
│   ├── store.rs
│   ├── searcher.rs
│   └── reranker.rs
├── exec/                    # Output compression
│   ├── mod.rs
│   ├── pipeline.rs
│   ├── types.rs
│   └── metrics.rs
├── scanner.rs              # File discovery
├── cache.rs                # SQLite persistence
├── tokenizer.rs            # BM25 tokenization
└── output.rs               # Text/JSON formatting
```

---

## Padrões-Chave

| Padrão | Onde | Razão |
|--------|------|-------|
| **Imutabilidade** | Todos | Evita side effects, facilita paralelização |
| **Paralelismo** | Scanner, Extractor | `rayon` para parsing multi-thread |
| **Cache** | Pipeline `map` | Reutiliza entre execuções |
| **Budget-aware** | Ranking | Respeita limite de tokens |
| **Trait Extractor** | extractors/ | Extensibilidade por linguagem |

---

## Decisões de Design

- **BM25 vs Embeddings:** BM25 para `map` (rápido, sem deps externas); Embeddings para `search` (semântica)
- **SQLite para tudo:** Cache `map` + Storage `search` — tudo em um banco
- **Tree-Sitter:** Parsing robusto e extensível para múltiplas linguagens
- **Ollama local:** RAG offline, privacidade, sem API keys
- **8 Stages em `exec`:** Trade-off entre compressão e retenção de informação

Ver `research/implementation-decisions.md` para análise detalhada.
