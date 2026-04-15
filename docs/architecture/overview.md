# Context Engine — Referência Técnica

Referência técnica para engenheiros que precisam entender, manter ou estender o `ctx` (context-engine).

- **Para visão de produto e roadmap:** veja `docs/produto.md`
- **Para padrões de engenharia e invariantes:** veja `docs/patterns.md`
- **Para pesquisa e decisões de design:** veja `docs/pesquisa/`

---

## Visão Geral

O `context-engine` implementa **dois binários** com responsabilidades diferentes:

| Binário | Responsabilidade | Pipeline |
|---------|------------------|----------|
| **`ctx`** | Gerar repo_maps curados | `Scanner → Extractor → Cache → Ranker → Output` |
| **`ctx-search`** | Recuperação semântica local (RAG) | `Chunker → Indexer → Embedder → Searcher` |

Ambos compartilham infraestrutura (SQLite cache, paralelismo rayon) e respeita as duas invariantes: **token budget** e **memória SQLite** (ver `docs/patterns.md`).

---

## 1️⃣ Pipeline: `ctx` (Repo Map)

Entrada: `--title "Adicionar 2FA" --dirs "src/auth,src/models" --max-tokens 4096`

```
Ticket (título/descrição)
  ↓
[Fase 1] Scanner (scanner.rs)
  • Varre diretórios via --dirs
  • Respeita .gitignore (crate `ignore`)
  • Ignora test/, spec/, contrib/ automaticamente
  ↓
[Fase 2] Extractor + Cache (extractors/ + cache.rs)
  • Extrai assinaturas via Tree-sitter (classes, funções, campos)
  • Extrai referências cross-file (grafo de dependências)
  • Cache persistente SQLite (~/.cache/context_engine/cache.db)
  • Invalidação por SHA256 do conteúdo (reutiliza se arquivo não mudou)
  • Redução: ~50 tokens (função completa) → ~8 tokens (assinatura)
  • Parse paralelo via rayon
  ↓
[Fase 3] Ranking (ranking/)
  • BM25 (bm25.rs): ranqueia arquivos por relevância textual vs title
  • Se --seeds: Personalized PageRank (pagerank.rs)
    - Constrói grafo de dependências
    - Seeds recebem peso 50x, top-5 BM25 recebem 10x
    - Iterativo: alpha=0.85, max_iter=200
  • Budget Fitting (budget.rs): encaixa saída no --max-tokens
  ↓
[Fase 4] Output (output.rs)
  • Formato text: repo_map com assinaturas agrupadas por diretório
  • Formato json: array de {path, score, signatures}
  ↓
LLM recebe contexto curado → economia de tokens e turns
```

**Exemplo de saída:**
```text
src/auth/jwt.ts
  ├─ function generateToken(payload: JwtPayload): string
  ├─ function verifyToken(token: string): JwtPayload | null
  └─ interface JwtPayload { userId: string; exp: number }

src/auth/2fa.ts
  ├─ class TwoFactorAuth
  ├─ method enable(userId: string): Promise<string>
  └─ method verify(userId: string, code: string): Promise<boolean>
```

---

---

## 2️⃣ Pipeline: `ctx-search` (Recuperação Semântica)

Entrada: `ctx-search add meus-docs --source ./docs --include "**/*.md"`

```
Documentação (markdown, specs, guias)
  ↓
[Fase 1] Chunker (catalog/chunker.rs)
  • Segmenta markdown respeitando estrutura (headings)
  • Sobreposição 15% entre chunks (contexto fluido)
  • Preserva metadados (caminho, heading, índice)
  ↓
[Fase 2] Indexer (catalog/indexer.rs)
  • Cataloga chunks em SQLite
  • SHA256 do conteúdo → recatalogação seletiva (evita duplicação)
  • Suporta múltiplas collections (acervos nomeados)
  ↓
[Fase 3] Embedder (catalog/embedder.rs)
  • Gera embeddings semânticos via Ollama (local)
  • Lazy load: roda sob demanda com cache
  • Modelos: nomic-embed-text (encoder) + llama3.2 (reranker)
  ↓
[Fase 4] Searcher (catalog/searcher.rs)
  • BM25 (termo-a-termo) + busca vetorial (semântica)
  • RRF (Reciprocal Rank Fusion) para combinar resultados
  • Position bonus: chunks iniciais (com heading) recebem score extra
  ↓
[Fase 5] Reranker (catalog/reranker.rs)
  • Top-30 resultados → julgamento qualitativo via LLM
  • Verifica: "Este chunk realmente responde a pergunta?"
  ↓
Usuário obtém: chunks ranqueados por relevância semântica
```

---

## Estrutura do Código

```
src/
  ├─ main.rs                # CLI `ctx` (repo_map)
  ├─ lib.rs                 # Orquestração principal
  │
  ├─ scanner.rs             # File discovery (.gitignore)
  ├─ cache.rs               # SQLite: signatures + refs (SHA256 invalidation)
  ├─ tokenizer.rs           # Tokenização para BM25
  ├─ output.rs              # Formatação text/json
  │
  ├─ extractors/            # Tree-sitter: extração de assinaturas
  │  ├─ mod.rs              # Dispatch por extensão (.ts, .rb, .py, .groovy)
  │  ├─ typescript.rs       # TypeScript/TSX via tree-sitter-typescript
  │  ├─ python.rs           # Python via tree-sitter-python
  │  ├─ ruby.rs             # Ruby via tree-sitter-ruby
  │  └─ groovy.rs           # Groovy via gramática customizada (build.rs)
  │
  ├─ ranking/               # Ranking: BM25 + PageRank
  │  ├─ mod.rs              # Re-exports
  │  ├─ bm25.rs             # TF-IDF ranking
  │  ├─ pagerank.rs         # Personalized PageRank
  │  └─ budget.rs           # Token budget fitting
  │
  ├─ catalog/               # RAG local (ctx-search)
  │  ├─ mod.rs              # API pública do módulo
  │  ├─ types.rs            # Collection, Document, Chunk, SearchResult
  │  ├─ store.rs            # SQLite schema + CRUD
  │  ├─ chunker.rs          # Segmentação markdown-aware
  │  ├─ indexer.rs          # Pipeline de catalogação + SHA256
  │  ├─ embedder.rs         # Client Ollama (lazy load, 5min timeout)
  │  ├─ searcher.rs         # BM25 + vetorial + RRF fusion
  │  ├─ reranker.rs         # Julgamento qualitativo via LLM
  │  └─ cache_ops.rs        # Cache de operações custosas
  │
  ├─ bin/
  │  └─ ctx_search.rs       # CLI `ctx-search` (recuperação semântica)
  │
  └─ grammars/              # Gramáticas Tree-sitter customizadas
     └─ groovy/             # Compilada via build.rs
```

**Módulos por responsabilidade:**
- **Core (lib.rs):** orquestração, run(title, dirs, ...) → string
- **Código (scanner, extractors, ranking):** repo_map curado
- **Busca (catalog):** indexação + recuperação semântica
- **CLI:** main.rs (ctx) + bin/ctx_search.rs (ctx-search)

---

## Linguagens Suportadas

| Linguagem | Extractor | Extensões |
|---|---|---|
| Groovy | `groovy.rs` + grammar custom (`grammars/`) | `.groovy` |
| Ruby | `ruby.rs` | `.rb` |
| Python | `python.rs` | `.py` |
| TypeScript/TSX | `typescript.rs` | `.ts`, `.tsx` |

---

## CLI: `ctx` — Repo Map

```bash
# Básico: gerar repo_map para um ticket
ctx --title "Adicionar validação de CPF" \
    --dirs "src/validators,src/models"

# Com token budget (default: 4096)
ctx --title "Adicionar 2FA" \
    --dirs "src/auth" \
    --max-tokens 2000

# Com Personalized PageRank (muda resultado de top-5)
ctx --title "Adicionar 2FA" \
    --dirs "src" \
    --seeds "src/auth/core.ts,src/auth/jwt.ts"

# Saída em JSON (útil para ferramentas)
ctx --title "Bug: login loop infinito" \
    --dirs "src/auth,src/session" \
    --format json | jq '.[] | {path, score}'
```

**Flags:**

| Flag | Default | Descrição |
|---|---|---|
| `--title` | (obrigatório) | Título/descrição do ticket |
| `--dirs` | (obrigatório) | Diretórios alvo, separados por vírgula |
| `--max-tokens` | `4096` | Budget máximo de tokens para saída |
| `--top` | `0` | Top-N fixo (0 = usar token budget em vez de contar) |
| `--format` | `text` | Saída: `text` ou `json` |
| `--seeds` | (opcional) | Ativar Personalized PageRank (dirs seed, vírgula-separado) |

---

## CLI: `ctx-search` — Recuperação Semântica

```bash
# Registrar novo acervo
ctx-search add meu-projeto \
    --source ./docs \
    --include "**/*.md"

# Indexar e gerar embeddings (requer Ollama)
ctx-search index meu-projeto --with-embed

# Buscar
ctx-search search meu-projeto "Como configurar OAuth?"

# Ver status
ctx-search status meu-projeto

# Limpar storage (compactar DB)
ctx-search compact meu-projeto

# Listar todos os acervos
ctx-search list
```

**Subcomandos:**

| Subcomando | Descrição |
|---|---|
| `add <nome> --source <path> --include <glob>` | Registrar novo acervo documental |
| `index <nome> [--with-embed]` | Indexar documentos (gerar embeddings se --with-embed) |
| `search <nome> <query>` | Busca semântica |
| `list` | Listar acervos registrados |
| `status <nome>` | Stats: documentos, chunks, embeddings, tamanho |
| `compact <nome>` | Otimizar storage (VACUUM SQLite) |

---

## Cache & Storage

### `ctx` — Cache de Assinaturas

Local: `~/.cache/context_engine/cache.db` (SQLite, criado automaticamente)

**Tabelas:**
- `cache` — assinaturas extraídas (key: SHA256 do arquivo)
- `refs` — referências cross-file (grafo de dependências)

**Invalidação:** por SHA256 do conteúdo — se arquivo não muda, reutiliza cache.

```bash
# Ver hits/misses
sqlite3 ~/.cache/context_engine/cache.db \
  "SELECT COUNT(*) as cached_files FROM cache;"

# Limpar (força reparse total próxima execução)
rm -f ~/.cache/context_engine/cache.db
```

**Performance esperada:**
- 1ª execução: ~50-200ms (depende do corpus)
- 2ª+ execução: ~20-50ms (cache hit)

### `ctx-search` — Store de Documentos

Local: `~/.cache/context_engine/collections/<nome>.db` (SQLite por collection)

**Tabelas:**
- `collections` — metadados do acervo
- `documents` — documentos catalogados
- `chunks` — segmentos com embeddings
- `cache_ops` — cache de embeddings (SHA256)

```bash
# Ver status de uma collection
ctx-search status meu-projeto

# Limpar dados e re-indexar
rm ~/.cache/context_engine/collections/meu-projeto.db
ctx-search index meu-projeto --with-embed
```

---

## Avaliação de Funcionamento

### Indicador 1: repo_map Gerado

```bash
# Se vazio/ausente: scanner falhou ou nenhum arquivo encontrado
ctx --title "Test" --dirs "src" | head -20
```

**Checklist:**
- [ ] Nenhuma linha → scanner não encontrou arquivos (checar --dirs)
- [ ] Poucos arquivos → BM25 filtrou mal ou corpus muito pequeno
- [ ] Assinaturas presentes → extractor funcionou ✅

### Indicador 2: PPR Impact (quando --seeds)

```bash
# Comparar com vs. sem --seeds
ctx --title "Auth flow" --dirs "src" --format json | \
  jq '.[:3] | .[] | {path, score}'

ctx --title "Auth flow" --dirs "src" --seeds "src/auth" --format json | \
  jq '.[:3] | .[] | {path, score}'
```

Esperado: top-5 com --seeds inclui seed files com scores altos.

### Indicador 3: Cache Hit Rate

```bash
# 1ª execução: cold (carregam arquivo de stats no final)
time ctx --title "Test" --dirs "src" > /dev/null

# 2ª+ execução: ~80% hit em repos estáveis
time ctx --title "Test" --dirs "src" > /dev/null
```

Esperado: 2ª execução ~3-5x mais rápida.

### Indicador 4: Busca Semântica (ctx-search)

```bash
ctx-search search meu-projeto "Como fazer login?"
# Resultado: chunks ranqueados por relevância semântica (não apenas keywords)
```

Checar: Respostas incluem chunks semanticamente relevantes, mesmo se não têm a palavra "login"?

---

## Troubleshooting

### `ctx` timeout (>2s em repo pequeno)

1. **Corpus muito grande?** Limitar `--dirs`
2. **Cache corrompido?** `rm ~/.cache/context_engine/cache.db`
3. **Parser falhando?** Checar `cargo build` para erros

### `ctx` sem saída ou saída vazia

**Causa:** Scanner não encontrou arquivos.

```bash
# Debug: checar arquivo específico
ctx --title "test" --dirs "src/main.rs"  # Caminho de arquivo em vez de diretório

# Alternativa: verificar extensão
# ctx só parsea: .ts, .tsx, .py, .rb, .groovy
ls -la src/ | grep -E '\\.(ts|tsx|py|rb|groovy)$'
```

### `ctx-search` sem embeddings

**Causa:** Ollama não está rodando.

```bash
# Verificar Ollama
curl http://localhost:11434/api/tags

# Se erro: iniciar Ollama
ollama serve

# Baixar modelos
ollama pull nomic-embed-text
ollama pull llama3.2
```

### Groovy extractor não funciona

Grammar Groovy é compilado via `build.rs`. Se falha:

```bash
cargo clean
cargo build --release
# Verificar erros do compilador
```
