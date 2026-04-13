# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## O que é este projeto

`ctx` é uma CLI em Rust que gera mapas de repositório para LLMs. Dado um diretório, ele descobre arquivos, extrai assinaturas de código (funções, classes, referências), ranqueia por relevância e gera uma saída compacta dentro de um orçamento de tokens.

## Comandos

```bash
cargo build                    # Build
cargo build --release          # Build otimizado (LTO, opt-level=3)
cargo test                     # Roda testes (tests/integration.rs + unit tests em cada módulo)
cargo test <nome_do_teste>     # Roda um teste específico
cargo run -- <dir> --help      # CLI local (subcomando map)
cargo run -- catalog --help    # Subcomando de busca semântica
cargo clippy --all-targets --all-features -- -D warnings  # Lint (bloqueante no CI)
cargo fmt -- --check           # Formatação (bloqueante no CI)
```

Hooks Git via **lefthook** (`lefthook.yml`): `cargo fmt --check` e `cargo clippy` no pre-commit; `cargo test` no pre-push.

## Arquitetura

O binário expõe dois subcomandos principais: `map` (pipeline clássico de mapa de repositório) e `catalog` (busca semântica por embeddings).

**Pipeline map:** `Scanner → Extractor → Cache → Ranker → Budget → Output`

**Pipeline catalog:** `Chunker → Embedder → Store (SQLite+vec) → Searcher → Reranker`

```
src/
├── main.rs          # Entry point, subcomandos clap: map | catalog
├── lib.rs           # Orquestração do pipeline map
├── scanner.rs       # Descoberta de arquivos com suporte a .gitignore
├── cache.rs         # Cache SQLite invalidado por SHA256 do conteúdo
├── tokenizer.rs     # Tokenização para BM25
├── output.rs        # Formatação text/JSON do mapa
├── extractors/      # Extratores por linguagem via Tree-Sitter
│   ├── mod.rs       # Trait Extractor + dispatch por extensão
│   ├── python.rs
│   ├── ruby.rs
│   ├── typescript.rs
│   └── groovy.rs    # Usa gramática customizada compilada em build.rs
├── ranking/
│   ├── mod.rs       # Coordena BM25 + PageRank
│   ├── bm25.rs      # TF-IDF / BM25 por query
│   ├── pagerank.rs  # Personalized PageRank no grafo de dependências
│   └── budget.rs    # Binary search para maximizar arquivos no budget de tokens
└── catalog/         # Recuperação semântica (embeddings)
    ├── mod.rs        # API pública: add_collection, index, embed_pending, search, health
    ├── types.rs      # Collection, SearchResult, CollectionHealth
    ├── store.rs      # Persistência SQLite (acervos, docs, chunks, embeddings)
    ├── cache_ops.rs  # Operações de cache do catálogo
    ├── chunker.rs    # Divisão de documentos em chunks
    ├── embedder.rs   # Geração de embeddings via API OpenAI-compatible
    ├── indexer.rs    # Pipeline de indexação (chunking + embedding)
    ├── searcher.rs   # Busca por similaridade vetorial
    └── reranker.rs   # Re-ranking dos resultados
```

**Padrões-chave:**
- Paralelismo via `rayon` no scan e extração
- Seleção de arquivos por orçamento de tokens via binary search em `ranking/budget.rs` (1 token ≈ 4 chars)
- Gramática Groovy customizada em `grammars/groovy/` compilada via `build.rs`
- Cache SQLite em `~/.cache/ctx/` para evitar re-parsear arquivos não alterados
- Endpoint LLM configurável (OpenAI-compatible) para embeddings do catálogo

## Documentação

- `docs/arquitetura.md` — referência técnica detalhada (PT-BR)
- `docs/roadmap.md` — próximas features planejadas
- `docs/pesquisa/` — notas de pesquisa sobre code-search e decisões de implementação
