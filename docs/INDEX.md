# Documentação — Context Engine

> **Objetivo:** Guia central para `ctx` (repo map curado) + `ctx-search` (RAG local).

---

## Índice de Documentação

### 🚀 Para Começar

- **[`README.md`](../README.md)** — Quick start: instalação, exemplos `ctx` e `ctx-search`
- **[Produto](produto.md)** — Por que existe, status H1-H3, features implementadas vs roadmap

### 🏗️ Para Contribuidores

- **[Patterns](patterns.md)** — Invariantes (token budget + SQLite), modularidade, como estender
- **[Arquitetura](arquitetura.md)** — Dois pipelines, módulos, CLI, troubleshooting

### 📚 Features Implementadas

#### `ctx` — Repo Map
- **[Arquitetura: pipeline ctx](arquitetura.md#1️⃣-pipeline-ctx-repo-map)** — Scanner → Extractor → Ranking → Output
- **[Patterns: composição](patterns.md#padrões-de-código)** — Como reutilizar Scanner/Extractor em novos comandos

#### `ctx-search` — RAG Local
- **[Especificação RAG](especificacao-rag.md)** — Regras de negócio, conceitos entrada/saída
- **[Implementação RAG](implementacao-rag.md)** — Arquivos criados, testes, CLI
- **[Arquitetura: pipeline ctx-search](arquitetura.md#2️⃣-pipeline-ctx-search-recuperação-semântica)** — Chunker → Indexer → Embedder → Searcher → Reranker

#### `ctx exec` — Command Output Compression
- **[Overview](exec/overview.md)** — O que é, como funciona, domínios suportados
- **[Filtering Pipeline](exec/filtering-pipeline.md)** — Os 8 estágios de transformação de saída
- **[Configuration](exec/configuration.md)** — Customizar filtros e comportamento
- **[Metrics](exec/metrics.md)** — Rastrear economia de tokens

### 🔬 Pesquisa & Decisões

- **[State of the Art: Code Search](pesquisa/code-search-state-of-art.md)** — Survey de BM25, embeddings, PageRank
- **[Decisões de Implementação](pesquisa/decisoes-implementacao.md)** — Trade-offs técnicos

---

## Como Usar Esta Documentação

| Cenário | Comece Aqui | Depois |
|---------|-------------|--------|
| **Usar `ctx` ou `ctx-search`** | [`README.md`](../README.md) | [`Arquitetura`](arquitetura.md) para detalhes |
| **Entender o projeto** | [`Produto`](produto.md) | [`Patterns`](patterns.md) (filosofia) |
| **Usar `ctx exec`** | [`exec/Overview`](exec/overview.md) | [`exec/Configuration`](exec/configuration.md) para customizar |
| **Implementar nova feature** | [`Patterns`](patterns.md) → [`Arquitetura`](arquitetura.md) | Código + testes |
| **Adicionar linguagem** | [`Patterns` § "Como Adicionar Suporte"](patterns.md#como-adicionar-suporte-a-nova-linguagem) | `src/extractors/<lang>.rs` |
| **Estender ctx-search** | [`Patterns` § "Como Estender ctx-search"](patterns.md#como-estender-ctx-search-novos-subcomandos) | `catalog/mod.rs` |
| **Debug/Troubleshooting** | [`Arquitetura` § "Troubleshooting"](arquitetura.md#troubleshooting) | Logs, SQLite cache |
| **Entender decisões técnicas** | [`Pesquisa`](pesquisa/) | PRs relevantes no git |

---

## Status Atual

✅ **H1 — Core:** `ctx` + `ctx-search` implementados, modular, testado  
🔄 **H2 — RAG:** BM25 + embeddings + re-ranking, pronto para produção  
🔄 **H3 — MCP server + output compression:** roadmap  

Veja [`Produto § "Status Atual"`](produto.md#status-atual--mapa-de-features) para features detail.

---

## Contribuindo

- **Documentação:** português, código: inglês
- **Atualizar junto:** PR muda código + docs/especificacao-rag.md ou docs/arquitetura.md conforme necessário
- **Adicionar doc novo?** Atualize este INDEX.md com link
- **Mudança arquitetural?** Atualizar [`Patterns`](patterns.md) e/ou [`Arquitetura`](arquitetura.md)
