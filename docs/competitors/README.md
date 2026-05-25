# Concorrentes

Análise de projetos que ocupam espaço próximo ao do `ctx` (mapas de repositório, busca de código para agentes, RAG local, compressão de output, MCP servers de contexto).

Cada documento segue o mesmo template:

- **Visão geral** — o que é, tamanho, distribuição.
- **Como funciona** — arquitetura, indexação, storage.
- **Integração com agentes** — CLI, MCP, plugin.
- **Diferenças vs `ctx`** — o que ele faz que nós não fazemos.
- **O que `ctx` faz que ele não faz** — nossa vantagem atual.
- **Oportunidades** — gaps a fechar, se fizer sentido.
- **Leitura estratégica** — interpretação do movimento competitivo.

## Índice

| Projeto | Stack | Stars | Foco | Sobrepõe com |
|---------|-------|-------|------|---|
| [CodeGraph](./codegraph.md) | TypeScript | 22k | Grafo de símbolos resolvido (callers/callees/trace) via MCP | `map` |
| [RTK](./rtk.md) | Rust | 53k | Proxy CLI que comprime output de 100+ comandos | `exec` |
| [Context Mode](./context-mode.md) | TypeScript | 15.5k | MCP server: sandbox de output + session continuity | `exec`, parcial em `catalog` |
| [QMD](./qmd.md) | TypeScript | 25.5k | Search local de docs com RRF + LLM rerank local | `catalog` |

## Mapa de sobreposição por pipeline `ctx`

| Pipeline `ctx` | Concorrentes diretos | Nosso diferencial atual |
|---|---|---|
| **`map`** | CodeGraph | Token budget + PageRank; perdemos em call graph resolvido |
| **`catalog`** | QMD, parcialmente Context Mode | Endpoint OpenAI-compatible; perdemos em RRF + LLM local + query expansion |
| **`exec`** | RTK, Context Mode | Pipeline configurável; perdemos em strategies por comando e hook automático |

## Oportunidades transversais

Consolidando o que aparece em múltiplos concorrentes:

1. **Expor `ctx` como MCP server** — CodeGraph, Context Mode e QMD já são MCP nativo. É o gap mais comum.
2. **Reciprocal Rank Fusion no `catalog`** — Context Mode e QMD fazem; barato e melhora recall.
3. **Hook de interceptação automática para `exec`** — RTK e Context Mode fazem; usuário não chama manualmente.
4. **Session continuity** (PreCompact/SessionStart) — só Context Mode cobre; gap claro no mercado.
5. **Resolução de referências + call graph** no `map` — só CodeGraph cobre; muda a natureza da ferramenta.

## Leitura geral

O mercado se segmentou em **4 eixos** desde que agentes viraram mainstream:

- **Grafo de código** (CodeGraph)
- **Compressão de comando** (RTK)
- **Memória de sessão** (Context Mode)
- **Search local de docs** (QMD)

O `ctx` é o único que tenta cobrir 3 deles (`map` + `catalog` + `exec`) num só binário, mas está atrás dos especialistas em cada vertical. A decisão estratégica é **focar em 1-2 eixos e ser o melhor neles** ou continuar como swiss-army knife integrado. Os arquivos individuais detalham o gap específico de cada eixo.
