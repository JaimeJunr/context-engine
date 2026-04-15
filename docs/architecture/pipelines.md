# Pipelines Principais

Há três pipelines independentes em `ctx`:

## 1. Pipeline `map` — Repository Map
Extrai assinaturas de código e ranqueia por relevância.

**Fluxo:** Scanner → Extractor → Cache → Ranker (BM25 + PageRank) → Budget → Output

## 2. Pipeline `catalog` — Semantic Search (RAG)
Indexa documentação e permite busca por intenção.

**Fluxo:** Chunker → Indexer → Embedder → Store → Searcher → Reranker

## 3. Pipeline `exec` — Command Output Compression
Comprime output de comandos shell mantendo informações essenciais.

**Fluxo:** Command → Capture → Filter Pipeline (8 estágios) → Output
