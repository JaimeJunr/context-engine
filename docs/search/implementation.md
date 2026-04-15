# Implementação — Sistema de Recuperação Semântica

Implementação do sistema de recuperação semântica de conhecimento (RAG local) conforme especificação em [`especificacao-rag.md`](especificacao-rag.md).

## Status: CONCLUÍDO

## Arquivos criados

| Arquivo | Descrição |
|---|---|
| `src/catalog/types.rs` | Entidades: Collection, Document, Chunk, SearchResult, SearchMode |
| `src/catalog/store.rs` | Camada SQLite: schema + CRUD para todas as tabelas |
| `src/catalog/chunker.rs` | Segmentação markdown-aware com sobreposição 15% (RD-03, RD-04) |
| `src/catalog/indexer.rs` | Pipeline de catalogação com SHA256 e recatalogação seletiva (RD-01, RD-02, RD-05) |
| `src/catalog/embedder.rs` | Client Ollama com lazy load e timer 5min (RD-20, RD-21) |
| `src/catalog/cache_ops.rs` | Cache de operações custosas via SHA256 (RD-22) |
| `src/catalog/searcher.rs` | BM25 + vetorial + RRF fusion + normalização (RD-09 a RD-16) |
| `src/catalog/reranker.rs` | Julgamento qualitativo LLM top-30 (RD-13, RD-14) |
| `src/catalog/mod.rs` | API pública do módulo |
| `src/bin/ctx_search.rs` | CLI com subcomandos: add, index, embed, search, list, status, compact |

## Arquivos modificados

- `Cargo.toml` — novas deps (reqwest, tokio, serde, glob, chrono) + binário `ctx-search`
- `src/lib.rs` — adicionado `pub mod catalog`

## Testes

**20/20 passando** — cobrindo chunker, searcher (RRF, position bonus, normalização), embedder e types.

## Uso básico

```bash
# Requer Ollama local rodando: ollama serve
# Modelos necessários: ollama pull nomic-embed-text && ollama pull llama3.2

# Registrar acervo
cargo run --bin ctx-search -- add meus-docs --source ./docs --include "**/*.md"

# Indexar
cargo run --bin ctx-search -- index meus-docs --with-embed

# Buscar
cargo run --bin ctx-search -- search meus-docs "como funciona o chunking?"

# Status
cargo run --bin ctx-search -- status meus-docs
```
