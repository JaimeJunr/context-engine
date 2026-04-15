# Instruções: `src/ranking/` — Ranking Híbrido (BM25 + PageRank)

Módulo de **ranking de relevância** — combina TF-IDF/BM25 com Personalized PageRank para descobrir arquivos mais relevantes.

## Estrutura

```
ranking/
├── mod.rs         # Orquestração: blend de BM25 + PageRank
├── bm25.rs        # TF-IDF / BM25 scoring por query
├── pagerank.rs    # Personalized PageRank no grafo de dependências
└── budget.rs      # Binary search para maximizar arquivos respeitando token budget
```

## Fluxo

```
Query + Arquivos
    ↓
BM25::score(query, files) → [scores by relevance to query]
    ↓
PageRank::score(files, seeds) → [importance in dependency graph]
    ↓
blend(bm25_weight=0.6, pagerank_weight=0.4)
    ↓
Sort by score descending
    ↓
Budget::maximize(files, max_tokens) → respects token limit
    ↓
Final ranked list
```

## BM25 (Term Frequency-Inverse Document Frequency)

### Função

```rust
pub fn score(documents: &[Document], query: &str) -> HashMap<DocId, f32>
```

Retorna score [0..1] para cada documento.

**Formula BM25:**
```
score(doc, query) = ∑ IDF(term) * (f(term, doc) * (k1 + 1)) / 
                         (f(term, doc) + k1 * (1 - b + b * |doc| / avg_len))
```

- `IDF(term)` = inverse document frequency
- `f(term, doc)` = term frequency em doc
- `k1`, `b` = tuning parameters (default: 1.5, 0.75)
- `|doc|` = comprimento do documento
- `avg_len` = comprimento médio

### Tokenização

Implementada em `src/tokenizer.rs`:

```rust
pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() > 1)  // ignore single-char tokens
        .collect()
}
```

1 token ≈ 4 caracteres (usado para budget).

### Exemplo

```rust
use ctx::ranking::bm25;

let docs = vec![
    Document { id: 1, content: "authentication with JWT tokens" },
    Document { id: 2, content: "user login flow implementation" },
];

let scores = bm25::score(&docs, "authentication");
// scores[1] > scores[2]  ← doc 1 é mais relevante para "authentication"
```

## PageRank (Importância em Grafo de Dependências)

### Função

```rust
pub fn score(
    files: &[File],
    dependency_graph: &Graph<FileId, ()>,
    seeds: Option<&[FileId]>,
) -> HashMap<FileId, f32>
```

Retorna score [0..1] para cada arquivo.

**Personalized PageRank:**
- Se `seeds` é None → PageRank clássico
- Se `seeds` é Some([file_ids]) → Personalized: reinicia aleatoriamente nos seeds (ênfase em arquivos próximos aos seeds)

### Grafo de Dependências

Construído em `src/scanner.rs`:

```
main.rs
 ├─→ lib.rs
 │    ├─→ config.rs
 │    └─→ scanner.rs
 ├─→ ranking/mod.rs
 └─→ extractors/mod.rs
```

**Utilidade:**
- Archivos que muitos outros dependem de → alta importância
- Com seeds → ênfase em arquivos próximos ao seed

### Exemplo

```rust
use ctx::ranking::pagerank;

let files = vec![
    File { id: 1, name: "lib.rs", .. },
    File { id: 2, name: "ranking/mod.rs", .. },
    File { id: 3, name: "ranking/bm25.rs", .. },
];

let graph = build_dependency_graph(&files);

// Sem seeds → importância global
let global_scores = pagerank::score(&files, &graph, None);

// Com seeds → importância relativa a "ranking/mod.rs"
let relative_scores = pagerank::score(&files, &graph, Some(&[2]));
// Expects: scores[3] > scores[1]  ← ranking/bm25.rs próximo a ranking/mod.rs
```

## Blending (BM25 + PageRank)

Em `src/ranking/mod.rs`:

```rust
pub fn rank_files(
    files: &[File],
    query: &str,
    bm25_weight: f32,      // default: 0.6
    pagerank_weight: f32,  // default: 0.4
) -> Vec<(File, f32)> {
    let bm25_scores = bm25::score(&files, query);
    let pr_scores = pagerank::score(&files, &dep_graph, None);
    
    // Blend
    files.iter()
        .map(|f| {
            let combined = bm25_weight * bm25_scores[f.id]
                         + pagerank_weight * pr_scores[f.id];
            (f.clone(), combined)
        })
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
        .collect()
}
```

## Token Budget

Maximizar número de arquivos respeitando limite de tokens.

### Função

```rust
pub fn maximize_files(
    ranked_files: &[(File, f32)],
    max_tokens: usize,
) -> Result<Vec<File>>
```

Usa **binary search** em `budget.rs`:

```
Low = 1 file, High = N files
while Low <= High:
    Mid = (Low + High) / 2
    tokens_used = sum(tokens(&files[0..Mid]))
    if tokens_used <= max_tokens:
        Low = Mid + 1
    else:
        High = Mid - 1
return files[0..High]
```

**1 token ≈ 4 caracteres**

### Exemplo

```rust
use ctx::ranking::budget;

let ranked = vec![
    (File { .. }, 0.95),
    (File { .. }, 0.87),
    (File { .. }, 0.72),
];

let max_tokens = 4096;  // ~16KB
let selected = budget::maximize_files(&ranked, max_tokens)?;
// Returns subset que cabe em 4096 tokens
```

## Testes

```bash
cargo test ranking::
cargo test ranking::bm25::tests
cargo test ranking::pagerank::tests
cargo test ranking::budget::tests
```

### Teste Padrão

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_exact_term() {
        let docs = vec![
            Document { id: 1, content: "authentication with JWT" },
            Document { id: 2, content: "user profile setup" },
        ];
        let scores = score(&docs, "authentication");
        assert!(scores[1] > scores[2]);
    }

    #[test]
    fn test_pagerank_seed_influence() {
        let files = build_test_files();
        let graph = build_test_graph(&files);
        
        let pr_global = pagerank::score(&files, &graph, None);
        let pr_seeded = pagerank::score(&files, &graph, Some(&[seed_id]));
        
        // Score do seed deve aumentar
        assert!(pr_seeded[seed_id] > pr_global[seed_id]);
    }

    #[test]
    fn test_budget_respects_limit() {
        let ranked = build_ranked_files();
        let files = maximize_files(&ranked, 4000)?;
        
        let total_tokens = files.iter().map(|f| f.content.len() / 4).sum();
        assert!(total_tokens <= 4000);
    }
}
```

## Performance

### BM25

- **Complexidade**: O(N * M) onde N=docs, M=query terms
- **Otimização**: cache tokenization em `src/tokenizer.rs`

### PageRank

- **Complexidade**: O(iterations * edges) — típico 20-30 iterações
- **Otimização**: sparse matrix (CSR format) para grafos grandes

### Budget

- **Complexidade**: O(log N) binary search + O(N) sum
- **Otimização**: cache de tamanho de arquivo

## Configuração

Em `src/config.rs`:

```rust
pub struct RankingConfig {
    pub bm25_weight: f32,         // 0.6 default
    pub pagerank_weight: f32,     // 0.4 default
    pub pagerank_iterations: usize, // 20 default
    pub max_tokens: usize,        // 4096 default
}
```

---

**Última atualização**: 2026-04-14
