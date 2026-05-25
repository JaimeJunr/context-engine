# RTK — Rust Token Killer

- **Repo:** [rtk-ai/rtk](https://github.com/rtk-ai/rtk)
- **Stack:** Rust (single binary, <10ms overhead)
- **Stars / Forks:** 53k / 3.3k
- **Licença:** Apache-2.0
- **Distribuição:** `brew install rtk`, `cargo install`, curl installer, binários pré-compilados (macOS/Linux/Windows)

## Visão geral

Proxy CLI que **intercepta saída de 100+ comandos** (git, testes, build, lint, AWS CLI, kubectl, docker, npm, pip…) e comprime antes de chegar ao contexto do agente. Promete 60-90% menos tokens. É o concorrente direto do nosso `ctx exec`.

## Como funciona

Quatro estratégias por categoria de comando:

1. **Filtragem inteligente** — remove ruído, comentários, boilerplate
2. **Agrupamento** — agrega itens similares (arquivos por diretório, erros por tipo)
3. **Truncação contextual** — preserva o relevante, elimina redundância
4. **Deduplicação** — colapsa linhas repetidas em contadores

### Exemplo de redução (sessão real 30 min)

| Comando | Antes | Depois | Redução |
|---|---|---|---|
| `git status` × 10 | 3.000 | 600 | 80% |
| `cargo test` × 5 | 25.000 | 2.500 | 90% |
| `cat/read` × 20 | 40.000 | 12.000 | 70% |
| **Total** | 118k | 23.9k | 80% |

## Integração com agentes

Suporta 13 ferramentas (Claude Code, Copilot, Cursor, Gemini, Windsurf, Cline, Hermes…) via:

- **Hooks** que reescrevem transparente: `git status` → `rtk git status`
- **Plugins** nativos quando a ferramenta expõe API de interceptação
- **CLAUDE.md / rules** como fallback (Windows nativo)

Setup: `rtk init -g` ou variante por agente.

## Diferenças vs `ctx exec`

| Aspecto | RTK | `ctx exec` (atual) |
|---|---|---|
| Cobertura de comandos | 100+ comandos com strategy específica | **17 famílias** (git, cargo, npm/yarn/pnpm, jest/vitest, tsc, eslint, prettier, biome, ruff, mypy, pytest, golangci-lint, go test, rubocop, rspec, rake, gradle/mvn/grails, docker, kubectl, aws (logs/sts/s3/ec2/lambda/iam/dynamodb/cfn), terraform/tofu, gh, ls/find/grep, curl/jq) |
| Interceptação | hooks reescrevem comando automaticamente | hook auto-rewrite via `ctx install --agent claude-code` ✅ |
| Strategies por domínio | git/test/build/aws/k8s com regras dedicadas | **mesmo padrão**: cada comando tem filtro específico (`filters/<família>.rs`) |
| Distribuição multi-agente | installer por ferramenta (13 suportadas) | 1 (Claude Code) — trait pronta para mais |
| Telemetria | opt-in, métricas de economia | `ctx exec report` (sem opt-in/server) |
| Log dedup inteligente | `log_cmd.rs` com normalização de timestamps/UUIDs | ✅ `pipelines::exec::dedup` aplicado em kubectl/docker/aws logs |
| `git push/pull/commit` → "ok" | sim, redução -92% | ✅ filtros dedicados retornam `ok <sha> <branch>: <subject>` |
| AWS por serviço (dynamodb unwrap, iam strip) | sim | ✅ `aws::dynamodb` (unwrap S/N/BOOL), `aws::iam` (strip PolicyDocument) |
| Terraform/Tofu plan | sim | ✅ `filters/terraform.rs` (plan/apply/init/validate) |
| Linters TS/JS/Py/Go/Ruby | sim | ✅ tsc, eslint, prettier, biome, ruff, mypy, golangci, go test, rubocop, rspec |

## O que `ctx` faz que ele não faz

RTK é **só compressão de output**. Não cobre:

- `map` (assinaturas + ranking BM25/PageRank de código)
- `catalog` (RAG semântico em documentação)
- Grafo de símbolos (CodeGraph faz, RTK não)

## Oportunidades

Olhando para o RTK, o que faltaria no nosso `exec` para competir no mesmo eixo:

1. **Strategies específicas por comando** (hoje somos genéricos). `git status`, `cargo test`, `kubectl logs` têm padrões muito diferentes e merecem pipeline dedicado.
2. **Hook de reescrita automática** — agente roda `git status` e o hook redireciona para `ctx exec git status` sem intervenção.
3. **Installer multi-agente** com `ctx init --agent <name>` configurando hooks/CLAUDE.md.

## Leitura estratégica

RTK é especialista vertical em **um** dos nossos 3 pipelines, e está muito à frente naquela vertical (53k stars, 13 integrações). Se quisermos disputar esse espaço, o caminho é **adotar a abordagem por strategy** em vez de pipeline genérico. Alternativa: tratar `exec` como feature complementar e focar diferenciação em `map` + `catalog`, deixando que o usuário componha com RTK.
