# Documentação — Context Engine

> **Guia Central** para `ctx` — CLI para repository maps e busca semântica local.

---

## 🚀 Quick Navigation

### Para Usuários
- **[README.md](../README.md)** — Instalação, exemplos rápidos
- **[Quick Start](guides/quick-start.md)** — Começar em 5 minutos
- **[CLI Reference](guides/cli-reference.md)** — Todos os subcomandos e opções

### Para Desenvolvedores
- **[Arquitetura](architecture/README.md)** — Design interno, pipelines, módulos
- **[Padrões de Design](architecture/design-patterns.md)** — Invariantes, como estender
- **[API Rust](api/README.md)** — Usar `context_engine` como biblioteca

### Para Pesquisa
- **[Code Search SoTA](research/code-search-state-of-art.md)** — Survey de BM25, embeddings, PageRank
- **[Decisões Técnicas](research/implementation-decisions.md)** — Trade-offs considerados

---

## 📚 Subcomandos

### [`ctx map`](map/README.md) — Repository Map
Gera mapa curado da estrutura de código para LLMs.

- **[Como funciona](map/how-it-works.md)** — Pipeline: Scanner → Extractor → Ranking
- **[Ranking Algorithm](map/ranking-algorithm.md)** — BM25 + Personalized PageRank
- **[Exemplos](map/examples.md)** — Casos de uso comuns

### [`ctx search` (Catalog)](search/README.md) — Busca Semântica
RAG local para documentação: indexação, embeddings, busca + re-ranking.

- **[Overview](search/overview.md)** — O que é, componentes, fluxo
- **[Indexação](search/indexing.md)** — Descoberta de documentos, chunking
- **[Embeddings](search/embeddings.md)** — Geração de vetores semânticos
- **[Re-ranking](search/reranking.md)** — Re-ranking contextual
- **[Exemplos](search/examples.md)** — Casos de uso
- **[Especificação](search/specification.md)** — Regras de negócio detalhadas
- **[Implementação](search/implementation.md)** — Detalhes técnicos

### [`ctx exec`](exec/overview.md) — Command Output Compression
Comprime output de comandos shell mantendo essencial, economizando tokens.

- **[Overview](exec/overview.md)** — O que é, como funciona
- **[Filtering Pipeline](exec/filtering-pipeline.md)** — Os 8 estágios
- **[Configuration](exec/configuration.md)** — Customização
- **[Metrics](exec/metrics.md)** — Rastreamento de economia

### [`ctx graph`](graph/README.md) — Grafo de Chamadas Semântico
Mapeamento e navegação de fluxo e símbolos (callers, callees, trace, impact).

- **[Overview](graph/README.md)** — Como funciona, subcomandos e framework routing

---

## 🏗️ Arquitetura & Extensão

| Tópico | Link |
|--------|------|
| **Visão Geral de Pipelines** | [architecture/pipelines.md](architecture/pipelines.md) |
| **Referência de Módulos** | [architecture/modules.md](architecture/modules.md) |
| **Como Estender** | [architecture/extending.md](architecture/extending.md) |
| **Decisões Arquiteturais** | [architecture/design-decisions.md](architecture/design-decisions.md) |
| **Padrões de Código** | [architecture/design-patterns.md](architecture/design-patterns.md) |

---

## 📖 Guides

- **[Visão do Produto](guides/vision.md)** — Por que existe, status, roadmap
- **[Roadmap](guides/roadmap.md)** — Próximas features
- **[Integração com Agentes](guides/agent-integration.md)** — `ctx install` / `ctx uninstall` (hooks Claude Code)
- **[Troubleshooting](guides/troubleshooting.md)** — Problemas comuns e soluções

---

## 🗂️ Estrutura de Documentação

```
docs/
├── INDEX.md                       ← Você está aqui
├── guides/                        ← How-to, quick-start, troubleshooting
│   ├── quick-start.md
│   ├── cli-reference.md
│   ├── vision.md
│   ├── roadmap.md
│   └── troubleshooting.md
├── map/                           ← Subcomando `ctx map`
│   ├── README.md
│   ├── how-it-works.md
│   ├── ranking-algorithm.md
│   └── examples.md
├── search/                        ← Subcomando `ctx search` (catalog)
│   ├── README.md
│   ├── overview.md
│   ├── indexing.md
│   ├── embeddings.md
│   ├── reranking.md
│   ├── examples.md
│   ├── specification.md
│   └── implementation.md
├── exec/                          ← Subcomando `ctx exec`
│   ├── overview.md
│   ├── filtering-pipeline.md
│   ├── configuration.md
│   └── metrics.md
├── graph/                         ← Subcomando `ctx graph`
│   └── README.md
├── architecture/                  ← Design interno
│   ├── README.md
│   ├── pipelines.md
│   ├── modules.md
│   ├── extending.md
│   ├── design-decisions.md
│   └── design-patterns.md
├── api/                           ← Rust crate reference
│   └── README.md
└── research/                      ← Pesquisa & decisões
    ├── README.md
    ├── code-search-sota.md
    └── implementation-decisions.md
```

---

## 📋 Mapa de Decisões

| Cenário | Comece Aqui | Depois |
|---------|-------------|--------|
| **Usar `ctx map`** | [map/README.md](map/README.md) | [map/how-it-works.md](map/how-it-works.md) |
| **Usar `ctx search`** | [search/README.md](search/README.md) | [search/overview.md](search/overview.md) |
| **Usar `ctx exec`** | [exec/overview.md](exec/overview.md) | [exec/configuration.md](exec/configuration.md) |
| **Usar `ctx graph`** | [graph/README.md](graph/README.md) | [graph/README.md](graph/README.md) |
| **Entender projeto** | [guides/vision.md](guides/vision.md) | [architecture/README.md](architecture/README.md) |
| **Implementar feature** | [architecture/design-patterns.md](architecture/design-patterns.md) | Código + testes |
| **Adicionar linguagem** | [architecture/extending.md](architecture/extending.md) | `src/pipelines/map/extractors/<lang>.rs` |
| **Integrar com agente** | [guides/agent-integration.md](guides/agent-integration.md) | `src/integrations/agents/<agente>.rs` |
| **Troubleshoot** | [guides/troubleshooting.md](guides/troubleshooting.md) | Logs com `RUST_LOG` |
| **Pesquisa técnica** | [research/README.md](research/README.md) | PRs no git |

---

## 🤝 Contribuindo

**Documentação:**
- Português nos comentários e docs
- Código em inglês
- Links sempre relativos (`path/to/file.md`)
- Atualizar este INDEX.md quando adicionar arquivo novo

**Mudanças de Código:**
- Se mudar arquitetura → atualizar `architecture/`
- Se mudar CLI → atualizar `guides/cli-reference.md`
- Se mudar spec → atualizar `search/specification.md` ou `map/how-it-works.md`
