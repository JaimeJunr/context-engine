# Subcomando `ctx search` (Catalog)

Busca semântica local em documentação, especificações e guias. Oferece RAG (Retrieval-Augmented Generation) completamente offline.

## Rápido

```bash
ctx add meu-projeto --source ./docs --include "**/*.md"
ctx index meu-projeto --with-embed
ctx search meu-projeto "como funciona o sistema de pagamento?"
```

## Conteúdo

- **[Overview](overview.md)** — O que é catalog, componentes, fluxo
- **[Indexing](indexing.md)** — Descoberta de documentos, chunking
- **[Embeddings](embeddings.md)** — Geração de vetores semânticos
- **[Reranking](reranking.md)** — Re-ranking contextual de resultados
- **[Examples](examples.md)** — Casos de uso
- **[Specification](specification.md)** — Regras de negócio detalhadas
- **[Implementation](implementation.md)** — Detalhes técnicos de implementação
