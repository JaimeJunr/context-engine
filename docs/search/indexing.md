# Indexação

## Fluxo Completo

1. **Discovery** — Scanner encontra arquivos usando patterns
2. **Stat Check** — Valida mtime/tamanho vs cache
3. **Parse** — Lê conteúdo do arquivo
4. **Chunk** — Divide em pedaços semânticos
5. **Store** — Persiste em SQLite

## Padrões Glob

```bash
ctx add docs \
  --source ./docs ./guides \
  --include "**/*.md" "**/*.txt" \
  --exclude "**/draft/**" "**/temp/**"
```

- `--source` — múltiplos paths permitidos
- `--include` — padrões de inclusão (AND)
- `--exclude` — padrões de exclusão (priority)

## Opções

- `--pre_index_cmd` — Comando a executar antes (ex: `git pull`)
- `--batch_size` — Tamanho de lote para embeddings

Exemplo:
```bash
ctx add docs \
  --source ./docs \
  --include "**/*.md" \
  --pre_index_cmd "git pull origin main" \
  --embedder_model "all-minilm-l6-v2"

ctx index docs --with-embed --batch_size 100
```
