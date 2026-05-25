# Context Mode

- **Repo:** [mksglu/context-mode](https://github.com/mksglu/context-mode)
- **Stack:** TypeScript, SQLite (FTS5, BM25, Porter stemming)
- **Stars / Forks:** 15.5k / 1.1k
- **Distribuição:** MCP server nativo, plugins por ferramenta

## Visão geral

MCP server que **intercepta tool calls no protocolo** e processa saídas em sandboxes isolados, retornando apenas resultado estruturado. Argumento de venda: 98-99% de redução em outputs grandes (snapshots Playwright, listagens GitHub, logs).

Cobre dois eixos:

1. **Sandboxing de output** (sobreposto com `ctx exec` e RTK)
2. **Session continuity** — preserva estado entre compactações (`PreCompact` + `SessionStart` hooks)

## Como funciona

| Componente | Detalhe |
|---|---|
| **Sandbox execution** | 12 runtimes (JS, Python, Go, Rust…) rodam código em subprocess; só stdout entra no contexto |
| **Knowledge base** | SQLite FTS5, chunking, BM25 + Porter stemming |
| **Session tracking** | edits, git ops, tasks, errors persistidos via 5 hooks (PreToolUse, PostToolUse, UserPromptSubmit, PreCompact, SessionStart) |
| **Hook routing** | PreToolUse bloqueia comandos perigosos antes de executar |

### Features de busca

- **Reciprocal Rank Fusion** combinando Porter stemming + trigram substring
- **Proximity reranking** para queries multi-termo
- **Fuzzy correction** ("kuberntes" → "kubernetes")
- **TTL caching** — URLs indexadas em 24h skipam re-fetch
- **Intent-driven filtering** — output >5KB com intent recebe summarization

## Integração com agentes

15+ plataformas:

- **Hook-capable** (Claude Code, Gemini CLI, VS Code Copilot, JetBrains, Cursor, OpenCode, Codex CLI) — hooks injetam routing rules em runtime, zero arquivos no projeto
- **Não-hook** (Zed, Antigravity) — copy manual de `AGENTS.md`/`GEMINI.md` (~60% compliance)
- **Plugin** (OpenCode, KiloCode, OMP, OpenClaw) — registra hooks in-process, hard-block enforcement

## Benchmarks

| Caso | Antes | Depois | Economia |
|---|---|---|---|
| Playwright snapshot | 56 KB | 299 B | 99% |
| GitHub issues (20) | 59 KB | 1.1 KB | 98% |
| Sessão completa | 315 KB | 5.4 KB | 94% |

Duração de sessão: ~30 min → ~3 h no mesmo context window.

## Diferenças vs `ctx`

| Capacidade | Context Mode | `ctx` |
|---|---|---|
| Interceptação no protocolo MCP | hooks bloqueiam/reescrevem antes do output entrar | usuário/agente chama `ctx exec` manualmente |
| Session continuity (PreCompact/SessionStart) | snapshot 2 KB reconstrói estado pós-compactação | ausente |
| Sandbox multi-runtime | 12 linguagens, subprocess isolation | sem execução isolada |
| Multi-strategy search com RRF | RRF + proximity + fuzzy | BM25 + PageRank (sem RRF/fuzzy) |
| TTL cache de URLs | sim | parcial (cache de embeddings em `catalog`) |
| Permission enforcement | bloqueia `sudo`, `rm -rf`, leitura de `.env` via hooks | ausente |

## O que `ctx` faz que ele não faz

- **`map` com extração de assinaturas + PageRank** — Context Mode busca em chunks textuais, não tem grafo de código
- **Token budget com binary search** para maximizar arquivos
- **RAG semântico com embeddings** (Context Mode é BM25 + stemming, sem embeddings vetoriais)

## Oportunidades

Pontos do Context Mode que poderiam entrar no `ctx`:

1. **Session continuity** — `PreCompact` hook que serializa estado relevante (arquivos ativos, tasks, decisões) e `SessionStart` que restaura. Diferencial real, pouco coberto pelos concorrentes.
2. **Reciprocal Rank Fusion** no `catalog` — fundir BM25 (texto) + vetorial (embedding) costuma melhorar recall sem custo.
3. **Hook MCP de interceptação** em vez de chamada manual ao `ctx exec`.
4. **Fuzzy correction** em queries de `catalog search`.

## Leitura estratégica

Context Mode disputa o mesmo espaço que RTK (compressão de output), mas via **protocolo MCP** em vez de proxy CLI. A inovação real dele é **session continuity** — nenhum dos outros concorrentes (RTK, CodeGraph, QMD) cobre isso.

Se quisermos um diferencial competitivo barato, "session memory" é o gap mais óbvio no mercado e o mais alinhado com nossa filosofia de economizar contexto.
