# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- MAINTENANCE: novas entradas vão em [Unreleased]; ao bumpar versão, mova-as
     para a nova seção `## [x.y.z] - YYYY-MM-DD` acima de [Unreleased]. -->

## [Unreleased]

> Próximas fases v0.3.0+: ver [docs/v0.2-roadmap.md](docs/v0.2-roadmap.md) — file watcher, FTS5, dispatch dinâmico, mais frameworks (Spring/Django/Express), session continuity.

## [0.2.0] - 2026-05-25

Foco: **paridade total nas 5 linguagens prioritárias (Java, Groovy, Ruby, Rust, JS/TS) + framework-aware routing** para Grails, Rails e NestJS. Primeiro passo formal do roadmap para fechar gap competitivo vs CodeGraph.

### Added

#### Cobertura completa das linguagens prioritárias

`ctx map` agora extrai assinaturas em **7 linguagens** (3 novas: **Rust, Java, JavaScript**); `ctx graph` agora extrai calls/refs em **8 linguagens** (2 novas: **Groovy, JavaScript**). JavaScript ganhou extractor próprio em ambos os pipelines, separado do TypeScript.

| Linguagem | Map (sigs) | Graph (calls) | Framework router |
|---|:---:|:---:|:---:|
| TypeScript / TSX | ✅ | ✅ | **NestJS** ✅ |
| JavaScript / JSX / MJS / CJS | ✅ | ✅ | — |
| Python | ✅ | ✅ | — |
| Ruby | ✅ | ✅ | **Rails** ✅ |
| Groovy | ✅ | ✅ | **Grails** ✅ |
| Rust | ✅ | ✅ | — |
| Java | ✅ | ✅ | — |
| Go | — | ✅ | — |

#### Framework-aware routing (URL → handler)

Novo módulo `src/pipelines/graph/frameworks/` com trait `FrameworkRouter`. Durante `ctx graph index`, detecta arquivos especiais e injeta símbolos sintéticos `route::METHOD /path` no grafo com call sites ligando à action:

- **Rails** (`config/routes.rb`): `resources` (7+ actions RESTful), `resource`, verb DSL (`get`/`post`/`put`/`patch`/`delete`/`match`) com `to: 'ctrl#act'` ou rocket (`=> 'ctrl#act'`), `namespace` prefixando o path.
- **Grails** (`UrlMappings.groovy`): explicit mappings `"/url"(controller: ..., action: ..., method: ...)` + `resources: "name"` gerando 5 actions. Normaliza `$id` → `:id`.
- **NestJS** (`*.ts` com `@Controller`): combina `@Controller('/prefix')` + `@Get(':id')`/`@Post()`/`@Patch(...)` etc, capturando `ClassName::method` como handler.

Resultado prático: `ctx graph callers show` em projeto Rails/Grails retorna as rotas que mapeiam para a action; `ctx graph node "route::ANY /users/:id"` retorna o source do mapeamento.

#### Claude Desktop installer

`ClaudeDesktopInstaller` (`src/integrations/agents/claude_desktop.rs`) instala MCP server `ctx` no app Claude Desktop. **Diferencial único:** zero concorrentes (RTK, CodeGraph, Context Mode, QMD) tem installer automático para Desktop. QMD chega mais perto com snippet manual macOS-only.

- CLI: `ctx install --agent claude-desktop` + `ctx uninstall --agent claude-desktop`
- Path resolution cross-platform via `dirs::config_dir()`:
  - Linux: `~/.config/Claude/claude_desktop_config.json`
  - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
  - Windows: `%APPDATA%/Claude/claude_desktop_config.json`
- Preserva o arquivo quando ele ainda tem `preferences` do usuário (diferente do Claude Code installer que apaga arquivos que ficam vazios).

#### Roadmap formal

- `docs/v0.2-roadmap.md` documenta as 5 fases do plano completo para zerar gap vs CodeGraph (~12k LOC total). Esta release entrega: Fase B parcial (linguagens prioritárias) + Fase D parcial (3 frameworks centrais) + Fase E parcial (Claude Desktop).

### Changed

- `SUPPORTED_EXTS` do scanner expandido para `.rs/.java/.js/.jsx`
- `GRAPH_EXTS` expandido para `.groovy/.gradle/.js/.jsx/.mjs/.cjs`
- `ext_to_lang` retorna `rust`/`java`/`javascript` para extensões correspondentes
- `AgentName` enum tem variante `ClaudeDesktop` (CLI: `--agent claude-desktop`)
- `installer_for()` despacha para `ClaudeDesktopInstaller`

### Performance

- Indexação em paralelo via `rayon` continua dominando o tempo
- Framework routing adiciona < 5% de overhead por arquivo (regex + split em arquivos de rota são pequenos)

### Notes

- Claude Desktop não suporta hooks `PreToolUse` (só MCP servers), por isso o installer só escreve o bloco `mcpServers`
- 282 tests passing (+22 vs v0.1.0): 16 novos testes de map extractors + 3 novos de graph extractors + 11 novos de framework routers + 5 novos do Claude Desktop installer
- clippy `-D warnings` + `cargo fmt --check` limpos

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

[Unreleased]: https://github.com/JaimeJunr/ctx-engine/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/JaimeJunr/ctx-engine/releases/tag/v0.1.0
