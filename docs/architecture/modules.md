# Referência de Módulos

Estrutura modular do `src/`:

```
src/
├── main.rs                    Entry point CLI (dispatch para subcomandos)
├── lib.rs                     Thin facade — pub mod das 3 camadas
│
├── pipelines/                 LÓGICA DE DOMÍNIO
│   ├── map/                   Repository map (BM25 + PageRank)
│   │   ├── mod.rs             run() — orquestração do pipeline
│   │   ├── scanner.rs         Descoberta de arquivos (.gitignore aware)
│   │   ├── extractors/        Extração de assinaturas via Tree-Sitter
│   │   ├── ranking/           BM25, PageRank, budget binary search
│   │   └── output.rs          Formatação text/json
│   │
│   ├── catalog/               RAG local (busca semântica)
│   │   ├── mod.rs             API pública (add, index, search, health…)
│   │   ├── store.rs           Persistência SQLite
│   │   ├── chunker.rs         Chunking semântico
│   │   ├── embedder.rs        Embeddings via endpoint OpenAI-compatible
│   │   ├── indexer.rs         Pipeline indexação
│   │   ├── searcher.rs        Busca vetorial
│   │   └── reranker.rs        Re-ranking contextual
│   │
│   └── exec/                  Compressão de output de comandos
│       ├── mod.rs             API pública (run_proxy)
│       ├── pipeline.rs        8 estágios de filtragem
│       ├── registry.rs        Dispatch comando → filtro
│       ├── filters/           Filtros por família (git, cargo, docker…)
│       └── metrics.rs         Telemetria de economia
│
├── integrations/              INTERFACES EXTERNAS
│   ├── agents/                Hooks Claude Code (e futuros Cursor, Codex…)
│   │   ├── mod.rs             Trait AgentInstaller
│   │   ├── claude_code.rs     Impl Claude Code (hook + mcpServer)
│   │   ├── hook_handlers.rs   Handler de `ctx __hook`
│   │   └── settings_merge.rs  Helpers JSON (idempotente)
│   └── mcp/                   MCP server expondo pipelines como tools
│       ├── mod.rs             Entry point: serve() + tool_names()
│       └── server.rs          CtxServer + 4 tools (ctx_exec, ctx_search, ctx_map, ctx_list)
│
└── shared/                    UTILITÁRIOS CROSS-CUTTING
    ├── cache.rs               SQLite cache em ~/.cache/context_engine/
    ├── config.rs              ~/.ctx/config.toml
    ├── tokenizer.rs           Tokenização (1 token ≈ 4 chars)
    └── workspace.rs           Detecção de stack + `.ctx/config.toml`
```

## Regras de dependência

```
shared        ← independente
pipelines/*   ← pode importar de shared
integrations  ← pode importar de pipelines + shared
pipelines     ← NÃO importa integrations (proíbe ciclos)
```

A separação permite, no futuro, virar Cargo workspace (`ctx-shared`, `ctx-pipelines`, `ctx-integrations`, `ctx`) sem refactor adicional.

## Onde adicionar features futuras

| Feature | Localização |
|---------|-------------|
| RRF (fusão BM25 + vetorial) | `pipelines/catalog/rrf.rs` |
| Query expansion | `pipelines/catalog/query_expansion.rs` |
| Call graph (callers/callees) | `pipelines/map/graph.rs` |
| Cursor/Codex/opencode installers | `integrations/agents/<nome>.rs` |
| MCP server | `integrations/mcp/` ✅ entregue |
| Session continuity | `integrations/session/` |
| Telemetria | `shared/telemetry.rs` |

Influências competitivas estão documentadas em [docs/competitors/](../competitors/).
