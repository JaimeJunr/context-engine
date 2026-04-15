# Visão do Projeto

## O que é `ctx`?

`ctx` é uma CLI em Rust que resolve um problema fundamental ao trabalhar com LLMs em codebases grandes:

**LLMs chegam "cegos" a um repositório** — precisam descobrir por conta própria quais arquivos são relevantes, gastando tokens e turns em exploração desnecessária.

`ctx` resolve isso entregando **apenas o contexto relevante** através de:

1. **`ctx map`** — Extrai assinaturas de código e ranqueia por relevância
2. **`ctx search`** — Busca semântica em documentação
3. **`ctx exec`** — Comprime output de comandos (economiza 60-90% de tokens)

---

## Status Atual

### ✅ H1 — Core (Completo)
- `ctx map`: Scanner, Extractor, BM25 + PageRank, Budget-aware output
- Suporte: TypeScript, Python, Ruby, Groovy
- Cache persistente em SQLite
- CLI funcional e estável

### ✅ H2 — RAG (Completo)
- `ctx search`: Indexação, chunking, embeddings semânticos
- Suporte Ollama (nomic-embed-text + llama3.2)
- Re-ranking contextual
- Busca semântica, exata, conceitual
- Totally offline

### 🔄 H3 — Compression & Integration (Em Andamento)
- `ctx exec`: Command output compression (8 estágios de filtragem)
- MCP server (não iniciado)
- Integração com Claude Code hooks

---

## Mapa de Features

| Feature | Status | Documentação |
|---------|--------|--------------|
| Repository mapping | ✅ | [map/](../map/README.md) |
| BM25 ranking | ✅ | [map/ranking-algorithm.md](../map/ranking-algorithm.md) |
| Personalized PageRank | ✅ | [map/ranking-algorithm.md](../map/ranking-algorithm.md) |
| Document indexing | ✅ | [search/indexing.md](../search/indexing.md) |
| Semantic embeddings | ✅ | [search/embeddings.md](../search/embeddings.md) |
| Contextual re-ranking | ✅ | [search/reranking.md](../search/reranking.md) |
| Output compression | ✅ | [exec/overview.md](../exec/overview.md) |
| TypeScript extraction | ✅ | Via Tree-Sitter |
| Python extraction | ✅ | Via Tree-Sitter |
| Ruby extraction | ✅ | Via Tree-Sitter |
| Groovy extraction | ✅ | Via custom grammar |
| Ollama integration | ✅ | [search/embeddings.md](../search/embeddings.md) |
| MCP server | 🚧 | Roadmap |
| Streaming output | 🚧 | Roadmap |

---

## Por Que Usar `ctx`?

### Para LLMs
- **Contexto preciso** → Melhor qualidade de resposta
- **Menos tokens** → Custo reduzido
- **Menos turns** → Fluxo mais rápido

### Para Desenvolvedores
- **Exploit busca semântica** → Encontre código relacionado por intenção
- **Offline** → Sem API keys externas, privacidade garantida
- **Extensível** → Adicione linguagens, customize ranking

### Para Agentes
- **Entenda repositórios** → Mapeamento automático antes de ação
- **Busca inteligente** → Encontre documentação por conceito
- **Compressão automática** → Logs e output listos para LLM

---

## Performance

- **Parsing:** Multi-threaded com `rayon`
- **Cache:** SQLite persistente, invalidado por SHA256
- **Ranking:** BM25 + PageRank em ~100ms para 1000 arquivos
- **Embeddings:** Batch processing com Ollama (384D)
- **Compression:** 8 estágios de filtragem inteligente

---

## Licença

MIT
