# Subcomando `ctx map`

Gera um mapa curado e comprimido da estrutura de código de um repositório, ideal para fornecer contexto a LLMs e agentes.

## Rápido

```bash
ctx map --title "CAP-123: Adicionar 2FA" --dirs src/auth,src/models --max-tokens 4000
```

## Conteúdo

- **[How It Works](how-it-works.md)** — Pipeline: Scanner → Extractor → Ranking → Output
- **[Ranking Algorithm](ranking-algorithm.md)** — BM25 + Personalized PageRank
- **[Examples](examples.md)** — Casos de uso comuns
