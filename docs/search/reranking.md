# Re-ranking Contextual

## Pipeline de Busca

1. **Semantic Search** — Busca vetorial retorna top-100 por similaridade
2. **Reranking** — Modelo re-classifica usando contexto completo
3. **Return** — Retorna top-k (default: 10)

## Estratégias de Reranking

### 1. Semantic Relevance (Padrão)
Re-calcula score usando embeddings da query + contexto de cada resultado.

```bash
ctx search meu-projeto "como fazer deploy?"
# Reranker lê cada top-100 resultado e re-classifica
```

### 2. Exact Match Boosting
Boost resultados que matcham query exatamente.

Use prefixo:
```bash
ctx search meu-projeto "exact:como fazer deploy"
```

### 3. Conceptual Expansion
Expande query com sinônimos antes de buscar.

```bash
ctx search meu-projeto "conceptual:deploy" 
# Busca: "deploy", "deployment", "release", "publish", ...
```

## Modelo Padrão

**Llama 3.2** (via Ollama):
- Prompt: "Rank these documents by relevance to query: ..."
- Output: Ranking ordenado

Customizar:
```bash
ctx add meu-projeto --source ./docs --reranker_model "mistral"
```

## Ajustes

Controle via `ctx config`:
```bash
ctx config set reranking.enabled true
ctx config set reranking.top_k 5   # retornar top 5
ctx config set search.batch_size 20 # busca em lotes
```
