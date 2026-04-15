# Embeddings Semânticos

## Como Funciona

1. **Chunk** é dividido em sentenças
2. **Embedder** (modelo LLM) gera vetor numérico (384D ou 768D)
3. **Store** persiste vetor em SQLite com suporte vetorial
4. **Search** calcula similaridade coseno entre query embedding e docs

## Modelos Suportados

Qualquer modelo com endpoint OpenAI-compatible:

### Padrão: Ollama Local
```bash
ollama serve
ollama pull nomic-embed-text  # 384D embeddings
```

### Alternativas
- `all-minilm-l6-v2` — 384D, mais rápido
- `e5-large` — 1024D, mais preciso (requer Ollama 0.3+)
- Endpoint remoto: `ctx add ... --llm_endpoint http://192.168.1.10:8080`

## Batch Processing

Embeddings são processados em lotes para eficiência:

```bash
ctx index meu-projeto --with-embed --batch_size 50
```

Aumentar batch size → mais rápido, mais RAM
Diminuir batch size → mais lento, menos RAM

## Armazenamento

SQLite com extensão vetorial (`sqlite-vec`):
- Tabela `embeddings`: chunk_id, embedding (vector)
- Índice de busca: otimizado para nearest neighbor
- Localização: `~/.cache/context_engine/collections/<nome>/`
