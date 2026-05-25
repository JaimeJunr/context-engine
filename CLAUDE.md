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

Estrutura modular em três camadas: `pipelines/` (domínio), `integrations/` (interfaces externas), `shared/` (utilitários cross-cutting).

```
src/
├── main.rs                       # Entry point + dispatch CLI
├── lib.rs                        # Thin facade (pub mod das 3 camadas)
│
├── pipelines/                    # LÓGICA DE DOMÍNIO
│   ├── map/                      # Repository map
│   │   ├── mod.rs                # run() — orquestração
│   │   ├── scanner.rs            # Discovery .gitignore-aware
│   │   ├── extractors/           # Tree-Sitter (.ts, .py, .rb, .groovy)
│   │   ├── ranking/              # BM25 + PageRank + budget
│   │   └── output.rs             # Formatação text/json
│   │
│   ├── catalog/                  # RAG local (busca semântica)
│   │   ├── mod.rs, types.rs, store.rs, chunker.rs
│   │   ├── embedder.rs, indexer.rs, searcher.rs
│   │   └── reranker.rs, cache_ops.rs
│   │
│   └── exec/                     # Compressão de output
│       ├── mod.rs                # API pública (run_proxy)
│       ├── pipeline.rs           # 8 estágios de filtragem
│       ├── registry.rs           # Dispatch comando → filtro
│       ├── filters/              # Filtros por família (git, cargo, docker…)
│       └── metrics.rs            # Telemetria de economia
│
├── integrations/                 # INTERFACES EXTERNAS
│   └── agents/                   # Hooks Claude Code (futuros Cursor/Codex)
│       ├── mod.rs                # Trait AgentInstaller
│       ├── claude_code.rs        # Impl Claude Code
│       ├── hook_handlers.rs      # Handler de `ctx __hook`
│       └── settings_merge.rs     # Helpers JSON (idempotente)
│
└── shared/                       # UTILITÁRIOS CROSS-CUTTING
    ├── cache.rs                  # SQLite ~/.cache/context_engine/
    ├── config.rs                 # ~/.ctx/config.toml
    ├── tokenizer.rs              # 1 token ≈ 4 chars
    └── workspace.rs              # Stack detection + .ctx/config.toml
```

### Regras de dependência (não-circular)

```
shared        ← independente
pipelines/*   ← pode importar shared
integrations  ← pode importar pipelines + shared
pipelines     ← NÃO importa integrations
```

Ver [docs/architecture/modules.md](docs/architecture/modules.md) para detalhes e [docs/architecture/extending.md](docs/architecture/extending.md) para onde adicionar features novas.

## Padrões-Chave

| Padrão | Onde | Detalhes |
|--------|------|----------|
| **Paralelismo** | `pipelines/map/scanner.rs`, `pipelines/map/extractors/` | Via `rayon` — scan e parsing em multi-thread |
| **Cache** | `shared/cache.rs` | SQLite em `~/.cache/context_engine/` — invalidado por SHA256 do arquivo |
| **Token Budget** | `pipelines/map/ranking/budget.rs` | Binary search para maximizar arquivos respeitando limite (1 token ≈ 4 chars) |
| **Gramática customizada** | `build.rs`, `grammars/groovy/` | Groovy usa gramática compilada (não há Tree-Sitter oficial) |
| **LLM Endpoint** | `pipelines/catalog/embedder.rs` | Configurável — suporta OpenAI-compatible (ex: Ollama local) |
| **Hooks agente** | `integrations/agents/` | `ctx install --agent claude-code` injeta PreToolUse hook redirecionando comandos cobertos para `ctx exec` |
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
| `src/lib.rs` | Thin facade (pub mod) | Raramente — só re-exports |
| `src/pipelines/map/mod.rs` | `run()` do pipeline map | Entender Scanner → Extractor → Ranker |
| `src/pipelines/catalog/mod.rs` | API pública do catalog | Estender busca semântica |
| `src/pipelines/exec/mod.rs` | API pública do exec | Novas estratégias de compressão |
| `src/integrations/agents/mod.rs` | Trait AgentInstaller | Adicionar novo agente (Cursor, Codex, …) |

### Fluxos Principais

**Para adicionar uma linguagem:**
1. Criar `src/pipelines/map/extractors/<lang>.rs` implementando trait `Extractor`
2. Registrar dispatch em `src/pipelines/map/extractors/mod.rs`
3. Testar em `tests/integration.rs`
4. Atualizar `docs/architecture/extending.md` se necessário

**Para estender busca semântica:**
1. Editar `src/pipelines/catalog/mod.rs` (nova API pública)
2. Implementar em `src/pipelines/catalog/searcher.rs` ou `reranker.rs`
3. Testar com `cargo test pipelines::catalog::`
4. Atualizar `docs/search/`

**Para adicionar filtro de compressão:**
1. Editar `src/pipelines/exec/types.rs` (configuração)
2. Implementar em `src/pipelines/exec/filters/<família>.rs`
3. Registrar em `src/pipelines/exec/registry.rs`
4. Testar com `cargo test pipelines::exec::`
5. Atualizar `docs/exec/`

**Para integrar com um novo agente:**
1. Criar `src/integrations/agents/<agente>.rs` implementando `AgentInstaller`
2. Adicionar variante em `AgentName` enum
3. Testes em `tests/agent_install.rs`

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
cargo test pipelines::map::ranking::     # Testes de ranking
cargo test pipelines::map::extractors::  # Testes de extractors
cargo test pipelines::catalog::          # Testes de busca semântica
cargo test pipelines::exec::             # Testes de exec
cargo test integrations::agents::        # Testes de hooks de agente

# Com output
cargo test -- --nocapture
```

## Estendendo o Projeto

Ver [docs/architecture/extending.md](docs/architecture/extending.md) — fluxos completos para adicionar linguagens, agentes, filtros de exec e novas integrações.

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
