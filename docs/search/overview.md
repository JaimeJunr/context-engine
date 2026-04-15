# Catalog: Visão Geral

## O que é?

Catalog é um sistema **RAG (Retrieval-Augmented Generation)** local e offline:
- Indexa documentação, especificações, guias
- Permite busca por **intenção** (não apenas palavras-chave)
- Gera embeddings semânticos e re-ranks resultados
- Completamente local (nenhuma API externa)

## Fluxo

```
1. add     → Registra coleção documental (paths, patterns, modelos)
2. index   → Descobre e chunka documentos
3. embed   → Gera embeddings vetoriais (via Ollama)
4. search  → Busca semântica + re-ranking
```

## Começar

```bash
# 1. Registrar
ctx add meu-projeto --source ./docs --include "**/*.md"

# 2. Indexar + Embeddings
ctx index meu-projeto --with-embed

# 3. Buscar
ctx search meu-projeto "como configurar SSL?"
```

## Componentes

- **Indexer** — Descobre documentos (scaneia FS, respeita patterns)
- **Chunker** — Divide docs em chunks semanticamente coerentes
- **Embedder** — Gera vetores via endpoint LLM (Ollama por padrão)
- **Store** — Persistência em SQLite com suporte vetorial
- **Searcher** — Busca por similaridade semântica
- **Reranker** — Re-ranking contextual de top-k resultados

## Configuração

Padrão:
- **Embedder:** nomic-embed-text (via Ollama)
- **Reranker:** llama3.2 (via Ollama)
- **Endpoint:** http://localhost:11434

Customizar:
```bash
ctx add meu-projeto \
  --source ./docs \
  --include "**/*.md" \
  --embedder_model "all-minilm-l6-v2" \
  --reranker_model "mistral" \
  --llm_endpoint "http://192.168.1.10:8080"
```
