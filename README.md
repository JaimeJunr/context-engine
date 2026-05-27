# ctx — Context Engine

[![Crates.io](https://img.shields.io/crates/v/ctx-engine.svg)](https://crates.io/crates/ctx-engine)
[![Docs](https://img.shields.io/badge/docs-jaimejunr.github.io%2Fcontext--engine-blue)](https://jaimejunr.github.io/context-engine/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rustup.rs/)
[![Tests](https://img.shields.io/badge/tests-282%20passing-green.svg)](#desenvolvimento)
[![Release](https://img.shields.io/github/v/release/JaimeJunr/context-engine)](https://github.com/JaimeJunr/context-engine/releases)

> **Maintainers e AI agents:** leia [`CLAUDE.md`](CLAUDE.md) antes de fazer qualquer mudança.

CLI Rust que dá ao seu agente de codificação **mapas curados de repositório**, **busca semântica em docs**, **compressão de output**, **grafo de chamadas em 8 linguagens** e **integração via MCP/hooks** — tudo em um único binário 100% local.

🌐 **Site oficial:** **<https://jaimejunr.github.io/context-engine/>**

## Install

**Sem Rust instalado** — um comando pega o build certo para o seu OS:

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/JaimeJunr/context-engine/releases/latest/download/ctx-engine-installer.sh | sh

# Windows (PowerShell)
irm https://github.com/JaimeJunr/context-engine/releases/latest/download/ctx-engine-installer.ps1 | iex
```

**Já tem Rust?** Use cargo:

```bash
cargo install ctx-engine
```

**Configura seu agente** em um comando — escreve hook PreToolUse + MCP server no `~/.claude/settings.json`:

```bash
ctx install --agent claude-code
```

Reabra a sessão do Claude Code e o agente passa a usar `ctx` automaticamente para compressão de comandos e consulta de grafo/repo map.

### Desinstalar

```bash
ctx uninstall --agent claude-code   # remove só o que o ctx instalou (preserva o resto)
```

### Plataformas suportadas

| OS | Arquitetura |
|---|---|
| Linux | x86_64, aarch64 |
| macOS | x86_64 (Intel), aarch64 (Apple Silicon) |
| Windows | x86_64 |

📖 **Próximos passos:**
- [Quick Start](docs/guides/quick-start.md) — em 5 minutos, primeiro mapa de repo
- [Integração com Agentes](docs/guides/agent-integration.md) — detalhes do hook + MCP
- [Análise de Concorrentes](docs/competitors/) — comparação técnica vs RTK, CodeGraph, Context Mode, QMD

**Documentação completa:** [`docs/INDEX.md`](docs/INDEX.md).

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
| Linguagens (grafo) | **8** (TS, JS, Py, Rb, Go, Rust, Java, Groovy) | n/a | 19+ | n/a | 5 (AST) |
| Multi-agente installer | 2 (Claude Code, Claude Desktop) | 13 | 5 | 15 | parcial |
| Stars (referência) | — | 53k | 22k | 15.5k | 25.5k |

**Onde ganhamos clarissimamente:**
- **Único** com hook + MCP server + compressão + RAG + repo map + grafo de chamadas no mesmo binário.
- **Único** com Token Budget via binary search (maximiza arquivos respeitando limite de tokens).
- **Único** com BM25 + Personalized PageRank híbrido para ranqueamento de repo map.
- **Único** com **ranking de relevância à query** em resultados de grafo (callers ranqueados por BM25 + número de sites). CodeGraph devolve lista crua sem ranking.
- **Único** com **budget de tokens em outputs de grafo** — trace/impact respeitam limite. CodeGraph pode estourar contexto.

**Onde estamos atrás (assumido, com plano):**
- Cobertura de comandos do `exec` (17 famílias vs 100+ do RTK) — ver [análise](docs/competitors/rtk.md) e roadmap.
- Multi-agente installer (Claude Code e Claude Desktop; trait `AgentInstaller` pronto para Cursor/Codex/opencode).
- Linguagens no grafo (8 vs 19+ do CodeGraph — falta C#, PHP, Swift, Kotlin, Scala, Dart, Svelte, Vue, Lua).
- Dispatch dinâmico / herança — CodeGraph faz, nós não. (Framework-aware routing inicial entregue para NestJS, Rails, Grails).
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
- **`ctx graph`** — grafo de símbolos resolvido (callers/callees/trace/impact/node) em 8 linguagens. **Resultados ranqueados** por relevância à query e respeitam token budget.
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

## Linguagens Suportadas

| Linguagem | `ctx map` (assinaturas) | `ctx graph` (callers/callees) | Framework router |
|---|:---:|:---:|:---:|
| TypeScript / TSX | ✅ | ✅ | **NestJS** ✅ |
| JavaScript / JSX | ✅ | ✅ | — |
| Python | ✅ | ✅ | 🚧 |
| Ruby | ✅ | ✅ | **Rails** ✅ |
| Groovy | ✅ | ✅ | **Grails** ✅ |
| Rust | ✅ | ✅ | 🚧 |
| Java | ✅ | ✅ | 🚧 (Spring planejado) |
| Go | — | ✅ | 🚧 (Gin/chi/mux planejados) |
| C#, PHP, Swift, Kotlin, Scala, Dart, Vue, Svelte, Lua | 🚧 | 🚧 | 🚧 |

## Dependência opcional

`ctx catalog` (busca semântica em docs) usa embeddings via endpoint OpenAI-compatible. **Ollama local** é o caminho recomendado:

```bash
ollama serve
ollama pull nomic-embed-text   # embedder padrão
ollama pull llama3.2            # reranker padrão
```

Sem Ollama, `ctx search` ainda funciona em modo léxico (sem embeddings).

## Uso

### `ctx map` — repo map curado

```bash
ctx map \
  --title "CAP-123: Adicionar validação de CPF" \
  --dirs "src/models,src/validators" \
  --max-tokens 4000
```

**Opções:** `--max-tokens N`, `--format json`, `--seeds dir1,dir2` (Personalized PageRank), `--top N`.

### `ctx catalog` — busca semântica em docs

```bash
ctx add meu-projeto --source ./docs --include "**/*.md"
ctx index meu-projeto --with-embed
ctx search meu-projeto "como funciona o pipeline de dados?"
```

### `ctx exec` — compressão de output

```bash
ctx exec cargo test          # comprime output de testes
ctx exec git status          # idem
ctx exec report              # relatório de tokens economizados
```

### `ctx graph` — grafo de chamadas

```bash
ctx graph index --dirs src                    # indexa o projeto
ctx graph callers apply_pipeline              # quem chama esta função?
ctx graph trace handle_request --depth 3      # cadeia até este símbolo
ctx graph impact migrate_db                   # o que quebra se eu mudar?
ctx graph callers run --query "exec proxy" --max-tokens 800   # ranqueado por relevância
```

### `ctx mcp` — MCP server

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
cargo test --locked --all-features                       # 282 testes
cargo clippy --all-targets --all-features -- -D warnings # lint estrito
cargo fmt -- --check                                     # formatação
cargo run -- map --help                                  # ajuda dos subcomandos
```

**Git Hooks (Lefthook):**
- `pre-commit`: `cargo fmt --check` + `cargo clippy`
- `pre-push`: `cargo test --locked --all-features`

## Licença

MIT (veja [LICENSE](LICENSE))
