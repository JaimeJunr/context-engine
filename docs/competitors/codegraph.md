# CodeGraph

- **Repo:** [colbymchenry/codegraph](https://github.com/colbymchenry/codegraph)
- **Site:** https://colbymchenry.github.io/codegraph/
- **Stack:** TypeScript, SQLite (FTS5, WAL), Tree-Sitter
- **Stars / Forks:** 22k / 1.2k (snapshot 2026-05-24)
- **Licença:** MIT
- **Distribuição:** `npm i -g @colbymchenry/codegraph`, installer `curl | sh` (bundla runtime Node)

## Visão geral

CodeGraph indexa um repositório como **grafo de conhecimento de código** (símbolos + arestas: calls, imports, herança, interface→impl) e expõe esse grafo via **MCP server** para agentes (Claude Code, Cursor, Codex CLI, opencode, Hermes). O argumento de venda é trocar a exploração ad-hoc do agente (grep/find/Read em subagents) por consultas diretas a um índice pré-construído.

**Benchmark divulgado:** média de 35% mais barato, 57% menos tokens, 46% mais rápido, 71% menos tool calls em 7 codebases reais (VS Code, Excalidraw, Django, Tokio, OkHttp, Gin, Alamofire).

## Como funciona

| Aspecto | Detalhe |
|---|---|
| **Parsing** | Tree-Sitter por linguagem, AST → nodes + edges |
| **Resolução** | Pós-extração: call→def, import→source, herança, dispatch dinâmico |
| **Storage** | SQLite com FTS5 em `.codegraph/codegraph.db`, modo WAL |
| **Atualização** | File watcher nativo (FSEvents/inotify/ReadDirectoryChangesW), debounce 2s |
| **Ignore** | Respeita `.gitignore` automaticamente |
| **Framework-aware** | Reconhece Django, Flask, FastAPI, Express, NestJS, Laravel, Rails, Spring, Gin, Axum, ASP.NET, Vapor, React Router, SvelteKit — liga URL patterns a handlers |

### Linguagens (19+)

TypeScript, JavaScript, Python, Go, Rust, Java, C#, PHP, Ruby, C, C++, Swift, Kotlin, Scala, Dart, Svelte, Vue, Liquid, Pascal/Delphi, Lua, Luau.

## Integração com agentes

Roda como **MCP server (stdio)**. Installer interativo configura cada agente automaticamente (`codegraph init -i`) e remove via `codegraph uninstall`.

### Tools MCP expostas

| Tool | Função |
|---|---|
| `codegraph_search` | lookup de símbolo por nome |
| `codegraph_context` | composição de código relevante para uma task em uma única call |
| `codegraph_trace` | tracing de call-path com corpos inline (segue dispatch dinâmico) |
| `codegraph_callers` | quem chama este símbolo |
| `codegraph_callees` | quem este símbolo chama |
| `codegraph_impact` | análise de código afetado por mudança |
| `codegraph_node` | detalhes do símbolo + source opcional |
| `codegraph_explore` | source de múltiplos símbolos + mapa de relacionamentos |
| `codegraph_files` | estrutura de arquivos indexados |

## Diferenças vs `ctx` (atualizado)

| Capacidade | CodeGraph | `ctx` |
|---|---|---|
| Grafo de chamadas resolvido | callers/callees/trace/impact com dispatch dinâmico | ✅ callers/callees/trace/impact/node (sem dispatch dinâmico ainda) |
| Resolução de referências | call→def, import→source, herança, interface→impl | ✅ call→def por nome simples; imports cross-file; sem herança ainda |
| **Ranking de relevância em resultados** | ❌ lista crua | ✅ BM25(query) + número de sites |
| **Budget de tokens em outputs de grafo** | ❌ pode estourar contexto | ✅ binary search igual ao `map` |
| **Dedup de sites de chamada similares** | ❌ | ✅ caller com N sites → 1 entrada + array de sites |
| MCP server nativo | stdio, 9 tools | stdio, **10 tools** (4 + 6 de grafo) |
| Framework-aware routing | URL pattern → handler em 14+ frameworks | 🚧 ausente |
| Live file watcher | FSEvents/inotify, debounce 2s | 🚧 re-indexação manual via `ctx graph index` |
| Dispatch dinâmico / herança | sim | 🚧 ausente |
| Linguagens | 19+ | **7** (TS, Py, Ruby, Go, Rust, Java, Groovy) |
| Busca textual | SQLite FTS5 | BM25 in-memory + grafo SQLite |
| Installer multi-agente | auto-configura 5 agentes | 1 (Claude Code) |

## O que `ctx` faz que ele não faz

- **`catalog`** — RAG semântico com embeddings sobre documentação. CodeGraph é 100% estrutural sobre código; não cobre wiki/docs/markdown.
- **`exec`** — compressão contextual de output de comandos (logs, stack traces, erros).
- **Token budget com binary search** — maximiza arquivos cabendo num limite de tokens.
- **PageRank personalizado** no ranking de relevância de arquivos.

## Oportunidades (se quisermos fechar gap)

Ordem de maior alavancagem para menor:

1. **Expor `ctx` como MCP server** com tools de grafo (`ctx_callers`, `ctx_callees`, `ctx_trace`). Pré-requisito: resolver referências, hoje só temos assinaturas.
2. **File watcher** para manter cache vivo entre invocações em vez de re-scan.
3. **Mais linguagens** via Tree-Sitter (Go, Rust, Java cobrem boa parte do mercado).
4. **Framework routing** (URL → handler) — diferencial perceptível em codebases web.

O `catalog` e o `exec` continuam sendo nossos diferenciais; ele não pretende cobrir esse espaço.

## Leitura estratégica

O movimento central do CodeGraph foi **resolver referências e virar MCP server**, não "gerar mapa textual". Para o agente, a diferença é qualitativa: em vez de receber um blob de assinaturas e ainda precisar grep para achar callers, faz `codegraph_callers(sym)` e recebe a resposta. Isso explica o "85% fewer tool calls" em VS Code.

O `ctx` está num eixo adjacente: mapa curado + RAG de docs + compressão de output. Sobreposição real é só no `map`. Se quisermos competir no eixo dele, o passo crítico é resolver refs e expor via MCP — o resto (linguagens, frameworks) escala depois.
