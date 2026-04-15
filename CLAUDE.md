# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## O que é este projeto

`ctx` é uma CLI em Rust que gera mapas de repositório inteligentes para LLMs e agentes, além de fornecer busca semântica local em documentação. Ele descobre arquivos, extrai assinaturas de código (funções, classes, tipos), ranqueia por relevância e gera saída compacta respeitando orçamento de tokens.

## Comandos Principais

```bash
# Build
cargo build                      # Debug build
cargo build --release            # Build otimizado (LTO, opt-level=3)

# Testes
cargo test                        # Roda todos os testes (integração + unitários)
cargo test --lib                 # Apenas testes unitários (em cada módulo)
cargo test <nome_do_teste>        # Teste específico (ex: cargo test test_bm25)
cargo test -- --nocapture        # Mostrar stdout dos testes

# CLI e Subcomandos
cargo run -- map --help           # Subcomando map (gera repo map)
cargo run -- catalog --help       # Subcomando catalog (busca semântica)
cargo run -- exec --help          # Subcomando exec (compressão de output)

# Qualidade de Código
cargo clippy --all-targets --all-features -- -D warnings  # Lint (bloqueante no CI)
cargo fmt -- --check              # Verificar formatação (bloqueante no CI)
cargo fmt                          # Formatar código
```

### Git Hooks via Lefthook

Configurado em `lefthook.yml`:
- **pre-commit**: `cargo fmt --check` + `cargo clippy` — formata e detecta erros antes de commitar
- **pre-push**: `cargo test --locked --all-features` — executa toda suite de testes antes de push

Hooks são executados automaticamente; para bypassar (não recomendado): `git push --no-verify`

## Arquitetura

O projeto implementa **três pipelines principais**:

### 1. Pipeline `map` — Repository Map
Gera um mapa curado da estrutura de código com assinaturas extraídas e ranqueadas por relevância.

**Fluxo:** `Scanner` → `Extractor` → `Cache` → `Ranker (BM25 + PageRank)` → `Budget` → `Output`

### 2. Pipeline `catalog` — Semantic Search (RAG Local)
Indexa documentação, gera embeddings vetoriais e permite busca por intenção (não apenas palavras-chave).

**Fluxo:** `Chunker` → `Indexer` → `Embedder` → `Store (SQLite+vec)` → `Searcher` → `Reranker`

### 3. Pipeline `exec` — Command Output Compression
Comprime saída de comandos shell (logs, errors, etc) aplicando filtros contextuais para economizar tokens.

## Estrutura de Código

```
src/
├── main.rs                # Entry point + subcomandos clap: map | catalog | exec
├── lib.rs                 # Orquestração do pipeline map
├── config.rs              # Configuração (caminhos, limites, endpoints)
├── scanner.rs             # Descoberta de arquivos com suporte a .gitignore
├── cache.rs               # Cache SQLite em ~/.cache/context_engine/ (invalidado por SHA256)
├── tokenizer.rs           # Tokenização para BM25 (1 token ≈ 4 chars)
├── output.rs              # Formatação text/JSON da saída
│
├── extractors/            # Extração de assinaturas de código via Tree-Sitter
│   ├── mod.rs             # Trait Extractor + dispatch por extensão (.ts, .py, .rb, .groovy)
│   ├── typescript.rs      # TypeScript/TSX (interfaces, classes, tipos)
│   ├── python.rs          # Python (classes, funções, tipos)
│   ├── ruby.rs            # Ruby (classes, métodos, módulos)
│   └── groovy.rs          # Groovy (sintaxe customizada em grammars/groovy/)
│
├── ranking/               # Ranking híbrido de relevância
│   ├── mod.rs             # Orquestração BM25 + PageRank
│   ├── bm25.rs            # TF-IDF / BM25 scoring por query
│   ├── pagerank.rs        # Personalized PageRank no grafo de dependências
│   └── budget.rs          # Binary search para maximizar arquivos respeitando token budget
│
├── catalog/               # RAG Local — busca semântica em documentação
│   ├── mod.rs             # API pública (add_collection, index, search, health)
│   ├── types.rs           # Tipos (Collection, SearchResult, CollectionHealth)
│   ├── store.rs           # Persistência SQLite (colections, docs, chunks, embeddings)
│   ├── cache_ops.rs       # Operações em cache (read, write, invalidate)
│   ├── chunker.rs         # Divisão de documentos em chunks semanticamente coerentes
│   ├── embedder.rs        # Geração de embeddings via endpoint OpenAI-compatible
│   ├── indexer.rs         # Pipeline (chunking → embedding → persistência)
│   ├── searcher.rs        # Busca vetorial por similaridade
│   └── reranker.rs        # Re-ranking contextual dos resultados
│
└── exec/                  # Compressão inteligente de output
    ├── mod.rs             # API pública (compress)
    ├── pipeline.rs        # 8 estágios de filtragem (summarize, truncate, etc)
    ├── types.rs           # Configuração de filtros
    └── metrics.rs         # Tracking de economia de tokens
```

## Padrões-Chave

| Padrão | Onde | Detalhes |
|--------|------|----------|
| **Paralelismo** | `scanner.rs`, `extractors/` | Via `rayon` — scan e parsing em multi-thread |
| **Cache** | `cache.rs` | SQLite em `~/.cache/context_engine/` — invalidado por SHA256 do arquivo |
| **Token Budget** | `ranking/budget.rs` | Binary search para maximizar arquivos respeitando limite (1 token ≈ 4 chars) |
| **Gramática customizada** | `build.rs`, `grammars/groovy/` | Groovy usa gramática compilada (não há Tree-Sitter oficial) |
| **LLM Endpoint** | `catalog/embedder.rs` | Configurável — suporta OpenAI-compatible (ex: Ollama local) |
| **Immutability** | Todos os módulos | Preferir criar novos objetos a mutar; patterns como builder, copy-on-write |

## Documentação do Projeto

A documentação principal está em **`docs/INDEX.md`**. Consulte quando:
- Entender decisões arquiteturais: `docs/arquitetura.md`
- Implementar nova feature: `docs/patterns.md` (filosofia, como estender)
- Usar `ctx exec`: `docs/exec/overview.md`
- Compreender trade-offs: `docs/pesquisa/`

## Como Navegar o Código

### Entrypoints

| Arquivo | Propósito | Quando Usar |
|---------|----------|-----------|
| `src/main.rs` | Subcomandos CLI (clap) | Adicionar novo comando ou mudar interface |
| `src/lib.rs` | Orquestração do pipeline `map` | Entender fluxo: Scanner → Extractor → Ranker |
| `src/catalog/mod.rs` | API pública do `catalog` | Novos métodos ou mudanças na busca semântica |
| `src/exec/mod.rs` | API pública do `exec` | Novas estratégias de compressão |

### Fluxos Principais

**Para adicionar uma linguagem:**
1. Criar `src/extractors/<lang>.rs` implementando trait `Extractor`
2. Registrar dispatch em `src/extractors/mod.rs`
3. Testar em `tests/integration.rs`
4. Atualizar `docs/architecture/extending.md` se necessário

**Para estender busca semântica:**
1. Editar `src/catalog/mod.rs` (nova API pública)
2. Implementar em `src/catalog/searcher.rs` ou `reranker.rs`
3. Testar com `cargo test catalog::`
4. Atualizar `docs/search/` (specification.md ou implementation.md)

**Para adicionar filtro de compressão:**
1. Editar `src/exec/types.rs` (configuração)
2. Implementar em `src/exec/pipeline.rs` (estágio)
3. Adicionar métricas em `src/exec/metrics.rs`
4. Testar com `cargo test exec::`
5. Atualizar `docs/exec/` se necessário

## Debugging

```bash
# Ativar logs detalhados
RUST_LOG=ctx=debug cargo run -- map --dirs src

# Logs específicos de módulo
RUST_LOG=ctx::catalog=trace cargo run -- catalog search meu-acervo "query"

# Benchmark (release mode)
RUST_LOG=ctx=info cargo run --release -- map --dirs . --no-cache

# Limpar cache e forçar re-parse
rm -rf ~/.cache/context_engine/
cargo run -- map --dirs . --no-cache
```

## Testes

```bash
# Todos
cargo test

# Por módulo
cargo test ranking::           # Testes de ranking
cargo test extractors::        # Testes de extractors
cargo test catalog::           # Testes de busca semântica

# Com output
cargo test -- --nocapture
```

## Estendendo o Projeto

### Adicionar Suporte a Linguagem
1. Criar novo arquivo em `src/extractors/<lang>.rs`
2. Implementar trait `Extractor` (ver `src/extractors/mod.rs`)
3. Testar em `tests/integration.rs`
4. Registrar em dispatch em `src/extractors/mod.rs`

### Estender `catalog`
1. Novos comandos: editar `src/main.rs` (clap) + `src/catalog/mod.rs` (API)
2. Novos filtros: editar `src/exec/pipeline.rs`
3. Novas estratégias de re-ranking: criar em `src/catalog/reranker.rs`

Referência: `docs/architecture/extending.md`

## Documentação

A documentação foi reorganizada em 7 pastas temáticas:

- **`docs/INDEX.md`** — Hub central, navegação por cenário
- **`docs/guides/`** — How-to, quick-start, troubleshooting, visão do projeto
- **`docs/map/`** — Documentação do subcomando `ctx map`
- **`docs/search/`** — Documentação do subcomando `ctx search` (catalog)
- **`docs/exec/`** — Documentação do subcomando `ctx exec`
- **`docs/architecture/`** — Design interno, padrões, extensão
- **`docs/research/`** — Pesquisa, decisões técnicas
- **`docs/api/`** — Referência da biblioteca Rust

Ver `DOCS_STRUCTURE.md` para mapa completo.

**Referências por cenário:**
- Adicionar linguagem → `docs/architecture/extending.md`
- Estender catalog → `docs/architecture/extending.md` + `docs/search/`
- Entender ranking → `docs/map/ranking-algorithm.md`
- Troubleshoot → `docs/guides/troubleshooting.md`
