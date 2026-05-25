# ctx — Context Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rustup.rs/)
[![Tests](https://img.shields.io/badge/tests-214%20passing-green.svg)](#desenvolvimento)

> **Maintainers e AI agents:** leia [`CLAUDE.md`](CLAUDE.md) antes de fazer qualquer mudança.

CLI Rust que dá ao seu agente de codificação **mapas curados de repositório**, **busca semântica em docs**, **compressão de output** e **integração via MCP/hooks** — tudo em um único binário 100% local.

🦀 **Comece aqui:**
- [Quick Start](docs/guides/quick-start.md) — em 5 minutos, primeiro mapa de repo.
- [Integração com Agentes](docs/guides/agent-integration.md) — `ctx install --agent claude-code` configura hook + MCP server automaticamente.
- [Análise de Concorrentes](docs/competitors/) — comparação técnica com RTK, CodeGraph, Context Mode, QMD.

**Documentação completa:** [`docs/INDEX.md`](docs/INDEX.md).

```bash
cargo build --release
cp target/release/ctx ~/.local/bin/
ctx install --agent claude-code   # configura hook + MCP em ~/.claude/settings.json
```

## Por que ctx?

O único CLI que cobre **4 eixos competitivos** em um binário Rust único, 100% local: hook de reescrita automática, MCP server nativo, compressão de output e busca semântica de docs.

| | **ctx** | [RTK](https://github.com/rtk-ai/rtk) | [CodeGraph](https://github.com/colbymchenry/codegraph) | [Context Mode](https://github.com/mksglu/context-mode) | [QMD](https://github.com/tobi/qmd) |
|---|:---:|:---:|:---:|:---:|:---:|
| Hook PreToolUse auto-rewrite | ✅ | ✅ | ❌ | ❌ | ❌ |
| MCP server nativo | ✅ | ❌ | ✅ | ✅ | ✅ |
| Compressão de output de comando | ✅ | ✅ | ❌ | ✅ | ❌ |
| Repo map curado (BM25 + PageRank) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Busca semântica em docs (RAG local) | ✅ | ❌ | ❌ | parcial (FTS5) | ✅ |
| Token budget com binary search | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Grafo de chamadas** (callers/callees/trace/impact) | ✅ | ❌ | ✅ | ❌ | ❌ |
| **Ranking de relevância em resultados de grafo** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Budget de tokens em outputs de grafo** | ✅ | ❌ | ❌ | ❌ | ❌ |
| Session continuity (PreCompact) | 🚧 | ❌ | ❌ | ✅ | ❌ |
| 100% local (sem API externa) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Single binary Rust nativo | ✅ | ✅ | ❌ (TS) | ❌ (TS) | ❌ (TS) |
| Cobertura de comandos exec | **17 famílias** | 100+ comandos | n/a | parcial | n/a |
| Linguagens (grafo) | **7** (TS, Py, Rb, Go, Rust, Java, Groovy) | n/a | 19+ | n/a | 5 (AST) |
| Multi-agente installer | 1 (Claude Code) | 13 | 5 | 15 | parcial |
| Stars (referência) | — | 53k | 22k | 15.5k | 25.5k |

**Onde ganhamos clarissimamente:**
- **Único** com hook + MCP server + compressão + RAG + repo map + grafo de chamadas no mesmo binário.
- **Único** com Token Budget via binary search (maximiza arquivos respeitando limite de tokens).
- **Único** com BM25 + Personalized PageRank híbrido para ranqueamento de repo map.
- **Único** com **ranking de relevância à query** em resultados de grafo (callers ranqueados por BM25 + número de sites). CodeGraph devolve lista crua sem ranking.
- **Único** com **budget de tokens em outputs de grafo** — trace/impact respeitam limite. CodeGraph pode estourar contexto.

**Onde estamos atrás (assumido, com plano):**
- Cobertura de comandos do `exec` (17 famílias vs 100+ do RTK) — ver [análise](docs/competitors/rtk.md) e roadmap.
- Multi-agente installer (só Claude Code hoje; trait `AgentInstaller` pronto para Cursor/Codex/opencode).
- Linguagens no grafo (7 vs 19+ do CodeGraph — falta C#, PHP, Swift, Kotlin, Scala, Dart, Svelte, Vue, Lua).
- Dispatch dinâmico / herança / framework routing (URL → handler) — CodeGraph faz, nós não.
- Live file watcher — re-indexamos sob demanda, não automaticamente.

### Cobertura `ctx exec` (17 famílias)

| Categoria | Comandos |
|---|---|
| **VCS** | `git status/log/diff/show/branch/tag/stash/blame/push/pull/add/commit/fetch` (push/commit reduzem a `ok <sha>`) |
| **Build/Test Rust** | `cargo test/build/check/clippy/fmt/run/install` |
| **Build/Test JS/TS** | `npm/yarn/pnpm` (install/test/build/run), `jest`, `vitest` |
| **Linters JS/TS** | `tsc`, `eslint`, `prettier`, `biome` |
| **Linters Python** | `ruff`, `mypy`, `pytest` |
| **Linters Go** | `go test`, `go build`, `go vet`, `golangci-lint` |
| **Linters Ruby** | `rubocop`, `rspec`, `rake` |
| **JVM** | `gradle/gradlew/mvn/mvnw` (test/build/package/install/verify), `grails` |
| **Containers** | `docker ps/images/logs/compose`, `kubectl get/logs/describe` (logs com **dedup inteligente**) |
| **Cloud AWS** | `aws logs/sts/s3/ec2/lambda/iam/dynamodb/cloudformation` (DynamoDB unwrap, IAM policy strip) |
| **IaC** | `terraform plan/apply/init/validate`, `tofu plan/apply/init/validate` |
| **GitHub** | `gh pr/issue/run` |
| **Filesystem** | `ls`, `find`, `tree`, `grep/rg/ag` |
| **Network/Data** | `curl/wget`, `jq`, `sqlite3` |

## Features

- **`ctx map`** — repo map curado: extrai assinaturas via Tree-Sitter, ranqueia com BM25 + Personalized PageRank, respeita `.gitignore`, output em texto/JSON dentro de orçamento de tokens.
- **`ctx catalog`** — RAG local: indexa documentação, gera embeddings via endpoint OpenAI-compatible (Ollama, etc), busca por intenção.
- **`ctx exec`** — proxy universal: aplica filtros de compressão por família (git, cargo, npm, docker, kubectl, aws, gh, gradle, maven, pytest, etc), persiste métricas.
- **`ctx graph`** — grafo de símbolos resolvido (callers/callees/trace/impact/node) em 7 linguagens. **Resultados ranqueados** por relevância à query e respeitam token budget.
- **`ctx install --agent claude-code`** — configura hook PreToolUse (auto-rewrite de `git status` → `ctx exec git status`) **e** MCP server (`ctx_exec`, `ctx_search`, `ctx_map`, `ctx_list`) num único comando idempotente.
- **`ctx mcp serve`** — MCP server stdio expondo 4 tools com schema JSON gerado via `rmcp` + `schemars`.
- **Arquitetura modular** com 3 camadas (`pipelines/`, `integrations/`, `shared/`) preparada para crescer sem virar mono-arquivo.

## Estrutura do Código

```
src/
├── pipelines/               LÓGICA DE DOMÍNIO
│   ├── map/                 Scanner → Extractor → Ranker → Budget → Output
│   ├── catalog/             Chunker → Indexer → Embedder → Searcher → Reranker
│   └── exec/                Filtros por família + métricas
│
├── integrations/            INTERFACES EXTERNAS
│   ├── agents/              Hooks + MCP entries (Claude Code, futuros Cursor/Codex)
│   └── mcp/                 MCP server stdio com 4 tools
│
└── shared/                  UTILITÁRIOS CROSS-CUTTING
    ├── cache.rs             SQLite ~/.cache/context_engine/
    ├── config.rs            ~/.ctx/config.toml
    ├── tokenizer.rs
    └── workspace.rs
```

Detalhes em [`docs/architecture/modules.md`](docs/architecture/modules.md).

## Linguagens Suportadas (extração de assinaturas)

| Linguagem | Suporte |
|-----------|---------|
| TypeScript / TSX | ✅ Completo |
| Python | ✅ Completo |
| Ruby | ✅ Completo |
| Groovy | ✅ Completo (gramática customizada) |
| Outras (Go, Rust, Java, C#, …) | 🚧 Roadmap |

## Requisitos

- **Rust 1.70+** ([instalar](https://rustup.rs/))
- **Opcional para `ctx catalog`:** endpoint LLM OpenAI-compatible (Ollama local recomendado):
  ```bash
  ollama serve
  ollama pull nomic-embed-text   # embedder padrão
  ollama pull llama3.2            # reranker padrão
  ```
  Sem isso, `ctx search` ainda funciona em modo léxico (sem embeddings).

## Início Rápido

```bash
# Build
cargo build --release
cp target/release/ctx ~/.local/bin/

# Configurar integração com agente (escreve hook + MCP em ~/.claude/settings.json)
ctx install --agent claude-code

# Reabra a sessão do Claude Code e o agente passa a usar ctx automaticamente.
```

### Exemplo 1: `ctx map`

```bash
ctx map \
  --title "CAP-123: Adicionar validação de CPF" \
  --dirs "src/models,src/validators" \
  --max-tokens 4000
```

**Opções:** `--max-tokens N`, `--format json`, `--seeds dir1,dir2` (Personalized PageRank), `--top N`.

### Exemplo 2: `ctx catalog`

```bash
ctx add meu-projeto --source ./docs --include "**/*.md"
ctx index meu-projeto --with-embed
ctx search meu-projeto "como funciona o pipeline de dados?"
```

### Exemplo 3: `ctx exec`

```bash
ctx exec cargo test          # comprime output de testes
ctx exec git status          # idem
ctx exec report              # relatório de tokens economizados
```

### Exemplo 4: `ctx mcp`

```bash
ctx mcp tools                # lista tools expostas pelo server
ctx mcp serve                # sobe MCP server em stdio (long-running)
```

## Configuração

Global (`~/.ctx/config.toml`):

```toml
[llm]
endpoint = "http://localhost:11434"
embedder = "nomic-embed-text"
reranker = "llama3.2"
```

Per-projeto (`.ctx/config.toml`, gerado por `ctx init`):

```toml
[map]
dirs = ["src", "lib"]
max_depth = 15
ignore_extra = ["**/fixtures/**"]
```

## Documentação Completa

- [`docs/INDEX.md`](docs/INDEX.md) — hub central
- [`docs/architecture/`](docs/architecture/) — design interno, padrões, extensão
- [`docs/guides/`](docs/guides/) — how-to, quick-start, integração com agentes
- [`docs/map/`](docs/map/), [`docs/search/`](docs/search/), [`docs/exec/`](docs/exec/) — docs por pipeline
- [`docs/competitors/`](docs/competitors/) — análise de RTK, CodeGraph, Context Mode, QMD

## Desenvolvimento

```bash
cargo test --locked --all-features                       # 214 testes
cargo clippy --all-targets --all-features -- -D warnings # lint estrito
cargo fmt -- --check                                     # formatação
cargo run -- map --help                                  # ajuda dos subcomandos
```

**Git Hooks (Lefthook):**
- `pre-commit`: `cargo fmt --check` + `cargo clippy`
- `pre-push`: `cargo test --locked --all-features`

## Licença

MIT (veja [LICENSE](LICENSE))
