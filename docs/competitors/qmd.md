# QMD — Query Markup Documents

- **Repo:** [tobi/qmd](https://github.com/tobi/qmd)
- **Stack:** TypeScript / Node (node-llama-cpp + modelos GGUF), SQLite FTS5
- **Stars / Forks:** 25.5k / 1.6k
- **Licença:** MIT

## Visão geral

Search engine local para markdown, transcrições, notas e knowledge bases. **Tudo roda local** (embeddings, reranking, query expansion) via `node-llama-cpp` com GGUF. É o concorrente direto do nosso `ctx catalog`.

Curiosidade: o autor é o Tobi Lütke (CEO da Shopify), o que explica parte da tração.

## Como funciona

Pipeline em três camadas:

1. **Indexação** — documentos chunked em ~900 tokens com quebras em limites semânticos (headings, blocos de código). Para código (TS, JS, Python, Go, Rust): AST-aware via tree-sitter, quebra em funções/classes.
2. **Busca híbrida paralela** — BM25 (FTS5) + busca vetorial, fusão via Reciprocal Rank Fusion (RRF).
3. **Re-ranking** — LLM avalia top-30 com "yes/no + logprobs confidence", blend posicional com scores de recuperação.

### Features distintivas

- **Query expansion** — gera variantes automáticas da query para melhor cobertura semântica
- **Context layering** — metadados descritivos retornados junto com resultados
- **AST-aware chunking** em código

## Consumo

- **CLI:** `search`, `vsearch`, `query` com `--json`, `--files`
- **SDK:** Node/Bun lib com `store.search()` aceitando queries simples ou estruturadas
- **MCP:** tools `query`, `get`, `multi_get`, `status` para Claude Desktop e afins (stdio ou HTTP)

## Diferenças vs `ctx catalog`

| Aspecto | QMD | `ctx catalog` |
|---|---|---|
| LLM local (embedding + rerank) | node-llama-cpp + GGUF, tudo offline | endpoint OpenAI-compatible (Ollama é externo) |
| Reciprocal Rank Fusion (BM25 + vetorial) | sim | só vetorial + reranker contextual |
| Query expansion automática | sim | ausente |
| LLM reranker com logprobs | sim, blend posicional | reranker contextual sem logprobs |
| AST-aware chunking em código | tree-sitter para 5 linguagens | chunking semântico em markdown |
| MCP server nativo | sim | ausente (só CLI) |
| Distribuição | npm/SDK/MCP | Cargo |

## O que `ctx` faz que ele não faz

- **`map`** — extração de assinaturas, ranking BM25 + PageRank, token budget. QMD é search puro, não gera mapa de repo.
- **`exec`** — QMD não comprime output de comandos.
- **Tree-sitter para Ruby/Groovy** (QMD cobre TS/JS/Py/Go/Rust em AST chunking; nós temos Ruby/Groovy).

## Oportunidades

Itens do QMD que melhorariam o `catalog`:

1. **Reciprocal Rank Fusion** (BM25 + vetorial) — barato de implementar, ganho consistente em recall.
2. **Query expansion** — gerar 2-3 variantes via LLM antes da busca.
3. **LLM reranker com logprobs** em vez do reranker contextual atual.
4. **AST-aware chunking** quando documento é código (hoje chunking é genérico).
5. **MCP server** expondo `ctx_search` para Claude Desktop sem precisar shell-out CLI.

## Leitura estratégica

QMD está no estado-da-arte de **search local de docs**. Empata com Context Mode em RRF/fuzzy, ganha em LLM local (não depende de Ollama externo) e em AST-chunking.

O nosso `catalog` está dois passos atrás do QMD nesse eixo. Se quisermos manter `catalog` como diferencial vs CodeGraph (que não cobre docs), precisamos pelo menos **RRF + query expansion + MCP server** para não ficar obsoleto.
