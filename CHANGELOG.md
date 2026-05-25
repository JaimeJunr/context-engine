# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- MAINTENANCE: novas entradas vão em [Unreleased]; ao bumpar versão, mova-as
     para a nova seção `## [x.y.z] - YYYY-MM-DD` acima de [Unreleased]. -->

## [Unreleased]

## [0.1.0] - 2026-05-25

Primeiro release oficial. Consolida 4 pipelines de domínio (`map`, `catalog`, `exec`, `graph`), camada de integrações (hook PreToolUse + MCP server), e cobertura inicial de 7 linguagens. Trabalho construído iterativamente a partir das análises competitivas de RTK, CodeGraph, Context Mode e QMD em [`docs/competitors/`](docs/competitors/).

### Added

#### Pipeline `graph` — grafo de símbolos resolvido (novo)
- `src/pipelines/graph/store.rs` — SQLite store (symbols, calls, imports) em `~/.cache/context_engine/graph.db`
- `src/pipelines/graph/extractor.rs` — Tree-Sitter queries para **7 linguagens** (TypeScript, Python, Ruby, Go, Rust, Java, Groovy)
- `src/pipelines/graph/query.rs` — API pública: `callers`, `callees`, `trace`, `impact`, `node` com **diferenciais únicos**:
  - Ranking por BM25(query) + log(sites) + boost por kind
  - Token budget binary search nos outputs
  - Dedup de call sites (mesmo caller, N sites → 1 entrada + array)
- CLI: `ctx graph index|callers|callees|trace|impact|node`

#### Pipeline `exec` — compressão de output (estendido)
- 17 famílias de comando cobertas: git, cargo, npm/yarn/pnpm, jest/vitest, tsc, eslint, prettier, biome, ruff, mypy, pytest, golangci-lint, go test, rubocop, rspec, rake, gradle/mvn/grails, docker, kubectl, aws (sts/s3/ec2/lambda/iam/dynamodb/cfn), terraform/tofu, gh, ls/find/grep, curl/jq, sqlite3
- Util `dedup` (`src/pipelines/exec/dedup.rs`): normaliza timestamps/UUIDs/hex/IPs/paths + agrupa repetidas em janela
- Logs com dedup automático: `kubectl logs`, `docker logs`, `aws logs`
- Git ok-style: `git push/pull/commit` → `ok <sha> <branch>: <subject>` (-92% como RTK)
- AWS por serviço: DynamoDB unwrap `{"S":"x"}`→`"x"`, IAM strip `PolicyDocument` inline
- Terraform/Tofu: `plan/apply/init/validate` — strip refresh/lock, preserva diff + summary
- Linters formatados preservam `file:line:col error/warning` removendo boilerplate
- `ctx exec report` — métricas agregadas de economia

#### Integrations — interfaces para agentes (novo)
- `src/integrations/agents/` — trait `AgentInstaller` + impl Claude Code
- `ctx install --agent claude-code` — escreve hook PreToolUse + entry `mcpServers` em `~/.claude/settings.json` (idempotente, marcador `_installer=ctx`)
- `ctx uninstall --agent claude-code` — reverte preservando hooks alheios
- `ctx __hook claude-code-pre-tool-use` — reescreve Bash calls cobertos para `ctx exec <cmd>` (degradação suave em erros, exit 0 garantido)

#### MCP server (novo)
- `src/integrations/mcp/` — server stdio via `rmcp 1.7` + `schemars 1.0`
- `ctx mcp serve` (long-running) + `ctx mcp tools`
- **10 tools** expostas: `ctx_exec`, `ctx_search`, `ctx_map`, `ctx_list`, `ctx_graph_index`, `ctx_callers`, `ctx_callees`, `ctx_trace`, `ctx_impact`, `ctx_node`
- Schemas JSON gerados automaticamente

#### Pipeline `map` (refinamento)
- `ctx init` cria `.ctx/config.toml` detectando stack do projeto (Rust, Node, React, Rails, JVM, Python, monorepos turbo/nx)

#### Pipeline `catalog` (RAG local)
- API completa: `add`, `index`, `embed`, `search`, `list`, `status`, `compact`, `bootstrap`
- Suporte a endpoint OpenAI-compatible (Ollama local recomendado)
- `SearchResult` agora derive `Serialize` para uso via MCP

### Changed

- **Refactor modular completo**: reorganizada `src/` em três camadas (`pipelines/`, `integrations/`, `shared/`) com regras de dependência não-circulares
- `src/lib.rs` reduzido a ~30 linhas (apenas `pub mod` + entry-points)
- `run_proxy_capture` extraído como variante de `run_proxy` que retorna `String` (necessário para MCP)
- `pipelines::map::scanner::scan_files_with_exts` — variante com extensões customizadas (graph usa superset do map)
- `Cargo.toml` populado com metadados completos para crates.io (description, license, keywords, categories)

### Fixed

- Quoting/escaping de `tool_input.command` em hooks PreToolUse — usa `shell-words` para parse robusto
- Loop infinito potencial em `__hook`: comandos já iniciados com `ctx exec` não são reescritos

### Removed

- Re-exports de compatibilidade legados em `lib.rs` (migração para paths canônicos `pipelines/*`, `shared/*`, `integrations/*`)

### Performance

- Indexação do grafo em paralelo via `rayon` (CPU-bound) com inserção serial no SQLite
- Cache SQLite compartilhado entre `map` e `exec metrics` em `~/.cache/context_engine/`

### Security

- Hook PreToolUse sempre retorna exit 0 — qualquer falha vira passthrough silencioso, não quebra sessão do agente
- Uninstall remove apenas entradas marcadas com `_installer: "ctx"` — preserva configuração alheia

### Documentation

- `README.md` reescrito com tabela comparativa honesta vs RTK, CodeGraph, Context Mode, QMD
- `docs/competitors/{rtk,codegraph,context-mode,qmd}.md` — análise por concorrente com gaps fechados / abertos
- `docs/architecture/modules.md` + `extending.md` — refletindo estrutura modular nova
- `docs/guides/agent-integration.md` — uso de `ctx install` (hook + MCP juntos)
- `CLAUDE.md` reescrito para nova estrutura

### Tests

- **271 testes** (228 unit + 43 integração), 0 falhas
- `tests/agent_install.rs` — ciclo install→hook→uninstall com `HOME` temporário
- Cobertura nova em `dedup`, `git filters`, `aws filters`, `terraform`, `ts/python/go/ruby linters`, `graph::{store,query,extractor}`

[Unreleased]: https://github.com/JaimeJunr/context-engine/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/JaimeJunr/context-engine/releases/tag/v0.1.0
