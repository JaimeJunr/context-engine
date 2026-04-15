# Instruções: Módulo `src/`

Este arquivo guia desenvolvimento no núcleo de `ctx`.

## Estrutura

```
src/
├── main.rs            # CLI (clap) — subcomandos map, catalog, exec
├── lib.rs             # Orquestração do pipeline map
├── config.rs          # Configuração global, caminhos
├── scanner.rs         # Scanner de arquivos (.gitignore, paralelismo)
├── cache.rs           # Cache SQLite (~/.cache/context_engine/)
├── tokenizer.rs       # Tokenização para BM25
├── output.rs          # Formatação (text/JSON)
│
├── extractors/        # Extração de assinaturas (Tree-Sitter)
│   ├── mod.rs         # Trait Extractor + dispatch
│   ├── typescript.rs  # TS/TSX
│   ├── python.rs      # Python
│   ├── ruby.rs        # Ruby
│   └── groovy.rs      # Groovy (gramática customizada)
│
├── ranking/           # Ranking híbrido (BM25 + PageRank)
│   ├── mod.rs         # Orquestração
│   ├── bm25.rs        # TF-IDF scoring
│   ├── pagerank.rs    # Personalized PageRank
│   └── budget.rs      # Binary search para token budget
│
├── catalog/           # RAG local (busca semântica)
│   ├── mod.rs         # API pública
│   ├── types.rs       # Tipos de dados
│   ├── store.rs       # SQLite (collections, docs, chunks, embeddings)
│   ├── cache_ops.rs   # Operações de cache
│   ├── chunker.rs     # Chunking semântico
│   ├── embedder.rs    # OpenAI-compatible endpoint
│   ├── indexer.rs     # Pipeline de indexação
│   ├── searcher.rs    # Busca vetorial
│   └── reranker.rs    # Re-ranking contextual
│
└── exec/              # Compressão inteligente de output
    ├── mod.rs         # API pública
    ├── pipeline.rs    # 8 estágios de filtragem
    ├── types.rs       # Configuração
    └── metrics.rs     # Tracking de economia
```

## Padrões

### Imutabilidade (CRÍTICO)

Sempre criar novos objetos, nunca mutar:

```rust
// ❌ ERRADO
fn modify_config(mut cfg: Config) {
    cfg.timeout = 5000;
}

// ✅ CORRETO
fn with_timeout(cfg: &Config, timeout_ms: u64) -> Config {
    Config {
        timeout: timeout_ms,
        ..cfg.clone()
    }
}
```

### Error Handling

Use `Result<T>` + custom `Error` enum com contexto:

```rust
use anyhow::{Result, Context};

fn extract_signatures(path: &Path) -> Result<Vec<Signature>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    // ...
    Ok(sigs)
}
```

### Cache Invalidation

Cache é invalidado por SHA256 do arquivo. Atualizar quando semanticamente relevante:

```rust
// Em cache.rs ou modules que usam cache
let cache_key = format!("{}:{}", file_hash, version);
```

### Token Budget

1 token ≈ 4 caracteres. Budget binary search em `src/ranking/budget.rs`:

```rust
// Em ranking/budget.rs
let max_tokens = 4096;
let max_chars = max_tokens * 4;
let files = budget::maximize_files(&ranked, max_chars)?;
```

## Fluxos Comuns

### Adicionar Suporte a Linguagem

**Arquivos a Editar:**
1. `src/extractors/<lang>.rs` — nova implementação
2. `src/extractors/mod.rs` — registrar em dispatch
3. `tests/integration.rs` — testes

**Padrão:**

```rust
// src/extractors/golang.rs
pub struct GoExtractor;

impl Extractor for GoExtractor {
    fn extract(&self, code: &str, path: &Path) -> Result<Vec<Signature>> {
        // Use tree-sitter parser: tree_sitter_go
        // Return Vec<Signature> com funções, tipos, etc.
    }
}
```

Registrar em `src/extractors/mod.rs`:

```rust
fn extractor_for_ext(ext: &str) -> Option<Box<dyn Extractor>> {
    match ext {
        "go" => Some(Box::new(GoExtractor)),
        "ts" | "tsx" => Some(Box::new(TypeScriptExtractor)),
        // ...
    }
}
```

### Estender Ranking

**Para novo algoritmo de scoring:**

1. Criar função em `src/ranking/bm25.rs` ou novo módulo
2. Orquestrar em `src/ranking/mod.rs`
3. Testar isoladamente: `cargo test ranking::new_algo`

**Padrão:**

```rust
// src/ranking/mod.rs
pub fn rank_files(
    files: &[File],
    query: &str,
    bm25_weight: f32,
    pagerank_weight: f32,
) -> Vec<(File, f32)> {
    let bm25_scores = bm25::score(files, query);
    let pr_scores = pagerank::score(files);
    
    // Blend
    files.iter()
        .map(|f| {
            let score = bm25_weight * bm25_scores[f.id]
                      + pagerank_weight * pr_scores[f.id];
            (f.clone(), score)
        })
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal))
        .collect()
}
```

### Estender Busca Semântica (`catalog`)

**Para novo filtro de busca:**

1. Editar `src/catalog/types.rs` (novo tipo)
2. Implementar em `src/catalog/searcher.rs` (nova lógica)
3. Expor em `src/catalog/mod.rs` (API pública)

**Padrão:**

```rust
// src/catalog/searcher.rs
pub fn search_with_filter(
    store: &Store,
    query: &str,
    filter: SearchFilter,
) -> Result<Vec<SearchResult>> {
    let embeddings = embedder.embed(query)?;
    let mut results = store.search_by_similarity(embeddings, 10)?;
    
    // Aplicar filtro
    results = results.into_iter()
        .filter(|r| filter.matches(&r))
        .collect();
    
    Ok(results)
}
```

Expor em `src/catalog/mod.rs`:

```rust
pub fn search(
    collection_name: &str,
    query: &str,
    filter: Option<SearchFilter>,
) -> Result<Vec<SearchResult>> {
    let store = Store::open()?;
    if let Some(f) = filter {
        searcher::search_with_filter(&store, query, f)
    } else {
        searcher::search_all(&store, query)
    }
}
```

### Estender Compressão (`exec`)

**Para novo filtro:**

1. Editar `src/exec/types.rs` (novo tipo FilterConfig)
2. Implementar em `src/exec/pipeline.rs` (novo estágio)
3. Adicionar métricas em `src/exec/metrics.rs`

**Padrão:**

```rust
// src/exec/pipeline.rs
fn apply_filter_summary(output: &str, cfg: &SummaryConfig) -> String {
    // Resumir output mantendo linhas-chave
    // Economizar tokens incrementando metrics
    metrics::record_compression("summary", input_len, output_len);
    output
}
```

## Testes

### Estrutura

Tests vivem em cada módulo:

```rust
// src/ranking/bm25.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_score_exact_match() {
        let docs = vec![/* ... */];
        let scores = score(&docs, "exact phrase");
        assert!(scores[0] > 0.8);
    }
}
```

### Executar

```bash
cargo test                    # Todos
cargo test ranking::          # Módulo específico
cargo test -- --nocapture    # Com output
cargo test -- --test-threads=1  # Serial (para cache)
```

## Performance

### Paralelismo

`Scanner` e `Extractors` usam `rayon` para paralelismo:

```rust
// Em scanner.rs
use rayon::prelude::*;
files.par_iter()
    .map(|f| extract(f))
    .collect()
```

### Cache

Cache em `~/.cache/context_engine/` (SQLite):
- Invalidado por SHA256 do arquivo
- Otimizado para reads frequentes
- Limpeza: `rm -rf ~/.cache/context_engine/`

### Profiling

```bash
# Flamegraph
cargo install flamegraph
cargo flamegraph -- map --dirs src

# Perf
perf record -F 99 ./target/release/ctx map --dirs src
perf report
```

## Documentação

- Padrões arquiteturais: `../docs/arquitetura.md`
- Como estender: `../docs/patterns.md`
- Decisões: `../docs/pesquisa/`

---

**Última atualização**: 2026-04-14
