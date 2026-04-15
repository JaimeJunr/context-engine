# Como Funciona `ctx map`

## Visão Geral

`ctx map` executa um pipeline em 5 estágios:

1. **Scanner** — Descobre arquivos respeitando `.gitignore`
2. **Extractor** — Extrai assinaturas (funções, classes, tipos) via Tree-Sitter
3. **Cache** — Reutiliza resultados entre execuções (validado por SHA256)
4. **Ranker** — Ranqueia por relevância usando BM25 + Personalized PageRank
5. **Budget** — Seleciona top N arquivos respeitando token budget
6. **Output** — Formata em texto ou JSON

## Exemplo de Fluxo

```
$ ctx map --title "Adicionar autenticação 2FA" \
  --dirs src/auth,src/models \
  --seeds src/middleware \
  --max-tokens 3000

1. Scanner descobre arquivos em src/auth e src/models
2. Extractor extrai: classes User, AuthService, etc
3. Cache valida ou re-parseia (SHA256)
4. Ranker aplica:
   - BM25 scoring com query "autenticação 2FA"
   - Personalized PageRank com src/middleware como seed (prioriza arquivos próximos)
5. Budget seleciona top 15 arquivos (cabe em 3000 tokens)
6. Output formata texto com assinaturas
```

## Ranking Híbrido: BM25 + PageRank

- **BM25** — Relevância léxica (match com query)
- **Personalized PageRank** — Relevância estrutural (proximidade com seed dirs)

Quando você usa `--seeds`, o ranking favorece arquivos que:
1. Matcham a query (BM25)
2. Estão próximos aos seed directories (PPR)

## Cache

SQLite em `~/.cache/context_engine/`:
- Armazena conteúdo parseado
- Validado por SHA256 do arquivo
- Invalidado automaticamente se arquivo mudar

Force refresh: `--no-cache`
