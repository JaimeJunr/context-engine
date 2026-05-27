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
│   ├── exec/                  Compressão de output de comandos
│   │   ├── mod.rs             API pública (run_proxy)
│   │   ├── pipeline.rs        8 estágios de filtragem
│   │   ├── registry.rs        Dispatch comando → filtro
│   │   ├── filters/           Filtros por família (git, cargo, docker…)
│   │   └── metrics.rs         Telemetria de economia
│   │
│   └── graph/                 Grafo de símbolos e chamadas semânticas (novo)
│       ├── mod.rs             Indexação paralela & API pública
│       ├── extractor.rs       Extração via Tree-Sitter (8 linguagens)
│       ├── store.rs           Persistência SQLite
│       ├── query.rs           Navegação por BFS (callers, trace, impact)
│       ├── types.rs           Símbolos e call sites
│       └── frameworks/        Framework-aware routing (Rails, Grails, NestJS)
│
├── integrations/              INTERFACES EXTERNAS
│   ├── agents/                Instaladores para agentes (Claude Code/Desktop...)
│   │   ├── mod.rs             Trait AgentInstaller
│   │   ├── claude_code.rs     Impl Claude Code (hook + mcpServer)
│   │   ├── claude_desktop.rs  Impl Claude Desktop (mcpServer)
│   │   ├── hook_handlers.rs   Handler de `ctx __hook`
│   │   └── settings_merge.rs  Helpers JSON (idempotente)
│   └── mcp/                   MCP server expondo pipelines como tools
│       ├── mod.rs             Entry point: serve()
│       └── server.rs          CtxServer + 10 tools (exec, search, map, list, graph_index e navigation)
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
| Grafo de chamadas (novas linguagens/frameworks) | `pipelines/graph/` |
| Cursor/Codex/opencode installers | `integrations/agents/<nome>.rs` |
| MCP server | `integrations/mcp/` ✅ entregue (10 tools) |
| Session continuity | `integrations/session/` |
| Telemetria | `shared/telemetry.rs` |

Influências competitivas estão documentadas em [docs/competitors/](../competitors/).
