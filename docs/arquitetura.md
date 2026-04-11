# Context Engine — Referência Técnica

Referência técnica para engenheiros que precisam entender, usar ou estender o `ctx` (context-engine). Para pesquisa e decisões, veja `docs/pesquisa/`. Para roadmap e visão de produto, veja `docs/produto.md`. Para padrões de engenharia, veja `docs/patterns.md`.

> **Nota:** O context-engine foi reescrito de Python (`context_engine.py`) para Rust (`ctx`). Esta documentação reflete a versão Rust atual.

---

## Intenção Arquitetural

O pipeline `Scanner → Extractor → Cache → Ranker → Output` foi desenhado para que cada módulo seja **quase independente**. O objetivo é que novos comandos (`ctx grep`, `ctx ls`, `ctx diff`) possam compor os módulos existentes em combinações diferentes, sem precisar replicar lógica.

A orquestração entre módulos fica exclusivamente em `lib.rs`. Os módulos não se conhecem entre si.

As duas invariantes que todo caminho de execução deve respeitar: **token budget** (via `budget.rs`) e **memória SQLite** (via `cache.rs`). Ver `docs/patterns.md` para detalhes.

## Pipeline Híbrida

```
Ticket (título/descrição)
  |
  Fase 1: Scanner (scanner.rs)
    - Varre diretórios fornecidos via --dirs
    - Respeita .gitignore (crate `ignore`)
    - Skip test/, spec/, contrib/
  |
  Fase 2: Tree-sitter + Cache (extractors/ + cache.rs)
    - Extrai assinaturas via Tree-sitter (classes, métodos, campos)
    - Extrai referências cross-file (extract_refs)
    - Cache persistente SQLite (~/.cache/context_engine/cache.db)
    - Invalidação por hash do conteúdo (sha256)
    - Reduz ~50 tokens de função para ~8 tokens de interface
    - Parse paralelo via rayon
  |
  Fase 3: Ranking (ranking/)
    - BM25 ranqueia arquivos contra título do ticket (bm25.rs)
    - Se --seeds fornecido: Personalized PageRank (pagerank.rs)
      - Constrói grafo de dependências (símbolo -> arquivos)
      - Seeds recebem peso 50x, BM25 top-5 recebem 10x
      - PageRank iterativo (alpha=0.85, max_iter=200)
    - Budget fitting (budget.rs): encaixa no --max-tokens
  |
  Fase 4: Output (output.rs)
    - Formato text: repo_map com assinaturas agrupadas por dir
    - Formato json: array de {path, score, signatures}
```

**Resultado:** LLM recebe contexto curado sem gastar turns explorando.

---

## Estrutura do Código

```
src/
  main.rs          CLI (clap) — entry point
  lib.rs           Orquestração: scan -> parse -> rank -> output
  scanner.rs       File discovery com .gitignore support
  cache.rs         SQLite cache (sigs + refs, invalidação por sha256)
  tokenizer.rs     Tokenização para BM25
  output.rs        Formatação text/json
  extractors/
    mod.rs         Dispatch por extensão de arquivo
    groovy.rs      Tree-sitter Groovy (grammar custom via build.rs)
    ruby.rs        Tree-sitter Ruby
    python.rs      Tree-sitter Python
    typescript.rs  Tree-sitter TypeScript/TSX
  ranking/
    mod.rs         Re-exports
    bm25.rs        BM25 (TF-IDF) ranking
    pagerank.rs    Personalized PageRank sobre grafo de dependências
    budget.rs      Fit to token budget
```

---

## Linguagens Suportadas

| Linguagem | Extractor | Extensões |
|---|---|---|
| Groovy | `groovy.rs` + grammar custom (`grammars/`) | `.groovy` |
| Ruby | `ruby.rs` | `.rb` |
| Python | `python.rs` | `.py` |
| TypeScript/TSX | `typescript.rs` | `.ts`, `.tsx` |

---

## CLI

```bash
# Gerar repo_map para um ticket
ctx --title "CAP-123: Adicionar suporte a fundo de feriado" \
    --dirs "performit-rails/app/models,performit-rails/app/controllers" \
    --max-tokens 4096

# Com Personalized PageRank
ctx --title "CAP-123: Adicionar suporte a fundo de feriado" \
    --dirs "performit-rails" \
    --seeds "performit-rails/app/models,performit-rails/app/controllers" \
    --format json

# Top-N fixo (ignora budget)
ctx --title "CAP-123" --dirs "performit-rails" --top 10

# Sem cache (força re-parse)
ctx --title "CAP-123" --dirs "performit-rails" --no-cache
```

### Flags

| Flag | Default | Descrição |
|---|---|---|
| `--title` | (obrigatório) | Título/descrição do ticket |
| `--dirs` | (obrigatório) | Diretórios alvo, separados por vírgula |
| `--top` | `0` | Número fixo de arquivos (0 = usar --max-tokens) |
| `--max-tokens` | `4096` | Budget máximo de tokens para o repo_map |
| `--format` | `text` | Formato de saída: `text` ou `json` |
| `--seeds` | (opcional) | Dirs seed para PPR, separados por vírgula |
| `--no-cache` | `false` | Ignorar cache, forçar re-parse |

---

## Cache SQLite

O cache mora em `~/.cache/context_engine/cache.db` (criado automaticamente).

**Invalidação:** por sha256 do conteúdo do arquivo — se conteúdo não muda, cache reutiliza.

```bash
# Ver cache
sqlite3 ~/.cache/context_engine/cache.db "SELECT COUNT(*) FROM cache;"

# Limpar cache (força reparse total)
rm -f ~/.cache/context_engine/cache.db
```

---

## Como Avaliar se Está Funcionando

### Indicador 1: Turns de Exploração

```bash
# Ver quantos turns o LLM gastou explorando
# Ideal: <3 tool calls (cat, find)
# Ruim: >8 tool calls (explorando cegamente)
```

### Indicador 2: repo_map Coverage

Se repo_map está vazio ou ausente: scanner não encontrou arquivos, ou extractors falharam.

### Indicador 3: Plan Quality

Se plano menciona arquivos certos nas primeiras linhas: context_engine acertou.
Se plano começa com exploração (ls, find): context_engine falhou.

### Indicador 4: Cache Hit Rate

```bash
sqlite3 ~/.cache/context_engine/cache.db "SELECT COUNT(*) FROM cache;"
# Run 1: cold (parse tudo)
# Run 2+: ~80% hit em repos estáveis
```

### Indicador 5: PPR Impact

```bash
# Com --seeds: top-5 incluem arquivos de seeds com scores mais altos
# Sem --seeds: top-5 ordenados apenas por BM25 (TF-IDF)
ctx --title "CAP-123" --dirs "performit-rails" \
    --seeds "performit-rails/app/models" \
    --format json | jq '.[] | {path: .path, score: .score}' | head -5
```

---

## Troubleshooting

### Problema: ctx timeout (>5s)

1. Verificar se corpus é muito grande (muitos diretórios)
2. Limpar cache: `rm ~/.cache/context_engine/cache.db`
3. Próxima execução vai demorar (cold), segunda vai rápido

### Problema: "Nenhuma assinatura extraída" para Groovy

O grammar Groovy é compilado via `build.rs` a partir de `grammars/tree-sitter-groovy/`.
Verificar que o grammar está presente e que `cargo build` completa sem erros.

### Problema: PPR não ativado (só BM25)

Causa: `--seeds` não foi passado na invocação.
PPR só ativa quando seeds são fornecidos explicitamente.
