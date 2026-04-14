# ctx — CLI para repo map e recuperação semântica

`ctx` tem dois modos: **mapa estrutural** (entender código) e **busca semântica** (encontrar conhecimento em documentos indexados).

## Subcomandos

```
ctx map      Gera repo map curado para LLMs
ctx add      Registra ou atualiza um acervo documental
ctx index    Cataloga documentos do acervo (novos e modificados)
ctx embed    Gera embeddings para chunks pendentes
ctx search   Busca semântica no acervo
ctx list     Lista todos os acervos registrados
ctx status   Exibe relatório de saúde do acervo
ctx compact  Compacta o repositório interno, removendo dados obsoletos
```

---

## ctx map — Mapa estrutural de repositório

Use antes de implementar ou refatorar para entender a estrutura do codebase.

```bash
# Básico (budget padrão: 4096 tokens)
ctx map --title "entender autenticação" --dirs /path/to/repo

# Budget maior para repos grandes
ctx map --title "refatorar ranking" --dirs ./src --max-tokens 8000

# Múltiplos diretórios
ctx map --title "ticket XPTO" --dirs src,tests,docs

# Saída JSON
ctx map --title "análise" --dirs . --format json

# Número fixo de arquivos (ignora budget)
ctx map --title "top 20 arquivos" --dirs . --top 20

# Personalized PageRank: prioriza arquivos próximos aos seeds
ctx map --title "mudança no ranker" --dirs . --seeds src/ranking

# Forçar re-parse ignorando cache
ctx map --title "após refactor" --dirs . --no-cache
```

**Flags:**

| Flag | Padrão | Descrição |
|---|---|---|
| `--title` | obrigatório | Título/descrição da tarefa (usado no ranking) |
| `--dirs` | obrigatório | Diretórios separados por vírgula |
| `--max-tokens` | 4096 | Budget máximo de tokens |
| `--top` | 0 (usa budget) | Número fixo de arquivos retornados |
| `--format` | text | `text` ou `json` |
| `--seeds` | — | Dirs seed para Personalized PageRank |
| `--no-cache` | false | Força re-parse ignorando cache |

---

## Acervos (catalog) — Busca semântica em documentos

Pipeline: `add` → `index` → `embed` → `search`

### ctx add — Registrar acervo

```bash
ctx add minha-wiki --source /path/to/docs

# Com filtros glob
ctx add minha-wiki --source ./docs --include "**/*.md" --exclude "**/drafts/**"

# Com endpoint OpenAI-compatible (ex: Ollama local)
ctx add minha-wiki --source ./docs \
  --embedder-model nomic-embed-text \
  --reranker-model llama3.2 \
  --llm-endpoint http://localhost:11434

# Com comando pré-indexação
ctx add minha-wiki --source ./docs --pre-index-cmd "git pull"
```

### ctx index — Indexar documentos

```bash
ctx index minha-wiki                          # indexa novos e modificados
ctx index minha-wiki --with-embed             # indexa e já gera embeddings
ctx index minha-wiki --with-embed --batch-size 100
```

### ctx embed — Gerar embeddings pendentes

```bash
ctx embed minha-wiki
ctx embed minha-wiki --batch-size 100
```

### ctx search — Busca semântica

```bash
ctx search minha-wiki "como funciona autenticação"

# Prefixos de busca
ctx search minha-wiki "exact: JWT token validation"       # correspondência exata
ctx search minha-wiki "conceptual: segurança de sessão"   # busca conceitual
ctx search minha-wiki "expanded: rate limiting"           # busca expandida

ctx search minha-wiki "deploy pipeline" -k 20             # mais resultados
ctx search minha-wiki "configuração do banco" --full      # fragmento completo
```

### ctx list / status / compact

```bash
ctx list                  # lista todos os acervos
ctx status minha-wiki     # docs, embeddings pendentes, última indexação
ctx compact minha-wiki    # remove dados obsoletos
```

---

## Quando usar cada modo

| Situação | Comando |
|---|---|
| Entender estrutura de um codebase | `ctx map --title "..." --dirs <dir>` |
| Focar em área específica do código | `ctx map --title "..." --dirs . --seeds <dir>` |
| Buscar em documentação/wiki indexada | `ctx search <acervo> "<query>"` |
| Verificar estado de um acervo | `ctx status <acervo>` |
| Após mudanças grandes no repo | `ctx index <acervo> --with-embed` |

---

## ctx exec — Compressão de saída de comandos

`ctx exec` é um proxy de comando que reduz a saída de shell em **60-90%**, economizando tokens em operações típicas de desenvolvimento (testes, builds, git, listagens).

### Quando usar

Use `ctx exec run <cmd>` em:
- **Testes:** `pytest`, `cargo test`, `npm test` (output tipicamente 1000+ linhas)
- **Build/Lint:** `cargo build`, `eslint`, `clippy` (warnings verbosos)
- **Controle de versão:** `git log`, `git diff` (diffs grandes)
- **Listagem:** `ls -la`, `find .` (muitas linhas)
- **CI/CD:** `kubectl logs`, `docker logs` (output massivo)

**Não use quando:**
- Precisa de saída bruta completa (debugging, regex, binários)
- Comando muito curto (< 5 linhas)

### Sintaxe

```bash
# Execução com filtro automático
ctx exec run pytest tests/
ctx exec run cargo test
ctx exec run git log --oneline -20

# Sem filtro (apenas métricas)
ctx exec run --no-filter <cmd>

# Verificar forma filtrada antes de executar
ctx exec rewrite "cargo test"
```

### Exemplos de economia

| Comando | Redução | Antes → Depois |
|---------|---------|----------------|
| `git log` | 90% | 4,521 tokens → 450 tokens |
| `pytest` | 85% | 3,200 tokens → 480 tokens |
| `cargo test` | 92% | 2,100 tokens → 168 tokens |

### Entender métricas

Ao final, `ctx exec` mostra resumo:
```
[ctx exec] Reduction: 87% (before: 4,521 tokens → after: 587 tokens)
```

Significa:
- Saída original: ~4,521 tokens
- Saída filtrada: ~587 tokens
- **Economia: 87%**

### Meta-operações

```bash
ctx exec report              # Mostra economia acumulada
ctx exec discover            # Sugere oportunidades
ctx exec validate-config     # Valida config
```

### Integração transparente com hooks

Se hook instalado via `ctx exec install-hook`, o agente NÃO precisa prefixar:

```bash
pytest tests/  # Automaticamente reescrito para ctx exec run pytest tests/
```

---

## Integração com RTK

```bash
rtk ctx map --title "análise" --dirs .
rtk ctx search minha-wiki "autenticação"
rtk ctx exec run cargo test
```
