# ctx — CLI para repo map e recuperação semântica

`ctx` oferece dois modos principais:

1. **`ctx map`** — Entender estrutura de código (pipeline: scan → extract → rank → budget)
2. **`ctx search`** — Busca semântica em documentação indexada (embeddings + SQLite)
3. **Gerenciamento de acervos** — `add`, `index`, `embed`, `list`, `status`, `compact`

## Reescritas Automáticas (ctx-rewrite.sh)

O hook `ctx-rewrite.sh` reescreve automaticamente comandos para usar `ctx`:

| Padrão | Reescrita |
|--------|-----------|
| `find . -name "*.rs"` ou `find . -type f` | `ctx map --title "..." --dirs .` |
| `grep -r "pattern"` ou `rg "pattern"` | `ctx search acervo "pattern"` |
| Qualquer `ls -la`, `tree`, estrutura repo | `ctx map --top N --dirs .` |
| Histórico git `git log`, diffs | `ctx map --seeds . --dirs .` |

---

## 🎯 ESTRATÉGIA: Quando usar cada comando

### 1. `ctx map` — Entender código rapidamente

**Use quando:**
- Precisa entender a estrutura de um diretório
- Vai refatorar e precisa saber dependências
- Quer ver quais arquivos são importantes

**Detecta:**
- Palavras-chave no prompt: "entender", "explorar", "mapa de", "estrutura", "arquitetura", "overview"
- Comandos shell: `find`, `ls -la`, `tree`, `lsof`

```bash
# Básico (budget padrão: 4096 tokens)
ctx map --title "entender autenticação" --dirs src/auth

# Budget maior para repos grandes (8000 tokens)
ctx map --title "refatorar ranking" --dirs ./src --max-tokens 8000

# Múltiplos diretórios
ctx map --title "catalog + search" --dirs src/catalog,src/ranking

# Focar em arquivos-chave (top N em relevância)
ctx map --title "entry points" --dirs . --top 15

# Personalized PageRank: prioriza arquivos próximos aos seeds
ctx map --title "mudança no ranker" --dirs . --seeds src/ranking

# Forçar re-parse (ignorar cache após refactor)
ctx map --title "após refactor" --dirs . --no-cache

# Saída JSON para processamento
ctx map --title "análise" --dirs . --format json
```

**Flags:**

| Flag | Padrão | Descrição |
|---|---|---|
| `--title` | obrigatório | Descrição/contexto da tarefa (usado no ranking de relevância) |
| `--dirs` | obrigatório | Diretórios separados por vírgula |
| `--max-tokens` | 4096 | Budget máximo de tokens para saída |
| `--top` | 0 (usa budget) | Número fixo de arquivos (ignora budget) |
| `--format` | text | `text` ou `json` |
| `--seeds` | — | Diretórios seed para Personalized PageRank (prioriza próximos) |
| `--no-cache` | false | Força re-parse, ignorando cache |

---

### 2. `ctx search` — Busca semântica em documentação

**Use quando:**
- Precisa encontrar informação em documentação indexada
- Vai implementar algo e quer referências
- Precisa lembrar como funciona um padrão

**Detecta:**
- Palavras-chave: "encontrar", "buscar", "procurar", "onde", "como funciona"
- Comandos shell: `grep -r`, `rg`, busca de padrões

```bash
# Busca simples (auto-moda)
ctx search docs "como funciona autenticação"
ctx search api-wiki "JWT token validation"

# Modos de busca explícitos
ctx search docs "exact: erro 401"                    # correspondência exata
ctx search docs "conceptual: segurança de sessão"    # busca semântica/conceitual
ctx search docs "expanded: rate limiting"            # busca com expansão

# Mais resultados
ctx search docs "deploy pipeline" -k 20

# Fragmento completo da resposta
ctx search docs "configuração do banco" --full

# Reranking com modelo customizado
ctx search docs "autenticação" --reranker-model llama3.2
```

**Modos de busca:**
- **auto** (padrão): Escolhe entre exato/vetorial automaticamente
- **exact**: Busca por palavra-chave exata (BM25)
- **conceptual**: Busca semântica por embeddings
- **expanded**: Expande query com hipóteses antes de buscar

---

### 3. Gerenciar acervos documentais (Pipeline: add → index → embed → search)

#### `ctx add` — Registrar novo acervo

```bash
# Básico
ctx add minha-wiki --source /path/to/docs

# Com filtros glob
ctx add docs-api --source ./docs --include "**/*.md" --exclude "**/drafts/**"

# Com endpoint OpenAI-compatible (ex: Ollama local)
ctx add docs-llm --source ./docs \
  --embedder-model nomic-embed-text \
  --reranker-model llama3.2 \
  --llm-endpoint http://localhost:11434

# Com comando pré-indexação (ex: atualizar repo antes)
ctx add docs-projeto --source ./docs --pre-index-cmd "git pull"
```

#### `ctx index` — Indexar documentos (novos e modificados)

```bash
ctx index minha-wiki                          # Apenas indexar
ctx index minha-wiki --with-embed             # Indexar + gerar embeddings
ctx index minha-wiki --with-embed --batch-size 100
```

#### `ctx embed` — Gerar embeddings pendentes

```bash
ctx embed minha-wiki
ctx embed minha-wiki --batch-size 100         # Controlar batch size
```

#### `ctx list` — Listar acervos registrados

```bash
ctx list                  # Lista todos os acervos
```

#### `ctx status` — Verificar saúde do acervo

```bash
ctx status minha-wiki     # Mostra: docs, chunks, embeddings pendentes, última indexação
```

#### `ctx compact` — Limpar dados obsoletos

```bash
ctx compact minha-wiki    # Remove documentos deletados, chunks órfãos, etc
```

---

## 📋 Matriz de Decisão Rápida

| Situação | Comando | Exemplo |
|----------|---------|---------|
| Entender estrutura repo | `ctx map` | `ctx map --title "entender catalog" --dirs src/catalog` |
| Entender área específica | `ctx map` + seeds | `ctx map --title "ranker" --dirs . --seeds src/ranking` |
| Buscar padrão em docs | `ctx search` | `ctx search docs "como implementar cache"` |
| Criar novo acervo | `ctx add` | `ctx add wiki --source ./docs` |
| Indexar mudanças | `ctx index` | `ctx index wiki --with-embed` |
| Verificar status | `ctx status` | `ctx status wiki` |
| Listar tudo | `ctx list` | `ctx list` |

---

## ⚡ Dicas de Performance

- **`ctx map` é rápido** (análise local, sem rede)
- **`ctx search` é poderoso** mas requer índice pronto (`ctx index` + `ctx embed`)
- **Reuse Seeds**: Se vai trabalhar em `src/catalog`, use `--seeds src/catalog` para ranking inteligente
- **Budget vs Top**: Use `--max-tokens` para controlar saída, ou `--top N` para número fixo
- **Cache**: Reutilizado automaticamente; use `--no-cache` apenas após refactors grandes
