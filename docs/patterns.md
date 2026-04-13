# ctx — Patterns de Engenharia

> Padrões de design e arquitetura do projeto. Leitura obrigatória antes de implementar qualquer nova capacidade. De cima para baixo: da filosofia até os detalhes.

---

## Filosofia

O `ctx` existe para resolver um problema de escala: quanto maior a codebase, mais caro fica para um agente de IA descobrir o que é relevante. Nossa resposta é **inteligência local-first** — algoritmos que rodam na máquina do usuário, sem dependência de APIs externas, com estado persistido localmente.

O mercado caminha para economia de tokens. Nossa vantagem é fazer isso com mais inteligência: não apenas limpar ruído, mas entender o que é relevante e entregar só isso.

---

## As Duas Invariantes Imutáveis

Toda capacidade que o `ctx` implementar deve respeitar estas duas regras — **sem exceção, sem negociação**.

### 1. Token Budget (economizar)

Toda saída deve respeitar um orçamento de tokens. O LLM nunca recebe mais do que precisa.

- **Para `ctx`:** output passa por `budget.rs` antes de ser entregue
- **Para `ctx-search`:** searcher retorna chunks, reranker valida top-30
- `--max-tokens` é o contrato com o chamador
- Sem budget definido, use default — nunca retorne ilimitado

### 2. SQLite Persistência (reutilizar)

Toda informação computada que possa ser reutilizada **DEVE** ser persistida em SQLite. Recomputar é desperdício.

- **`ctx`:** assinaturas extraídas → cache por SHA256
- **`ctx-search`:** chunks, embeddings, search results → SQLite
- Cache mora em `~/.cache/context_engine/`
- Invalidação por conteúdo (SHA256), **nunca** por tempo
- Cache hit esperado: ~80% em repos estáveis, 100% em collections (ctx-search)

---

## Arquitetura Modular: Dois Pipelines Independentes

O `context-engine` implementa **dois pipelines separados** com módulos reutilizáveis:

### Pipeline 1: Repo Map (`ctx`)

```
Scanner → Extractor → Cache → Ranker → Output (lib.rs)
```

Módulos **não acoplados:**
- `Scanner` conhece `.gitignore`, não conhece Extractor
- `Extractor` não faz I/O de arquivo (recebe string, retorna signatures)
- `Cache` não conhece Ranker (só persiste/recupera)
- `Ranker` não faz parsing (aplica BM25/PageRank a assinaturas existentes)

**Composição futura possível:**
- `ctx map`: todos (padrão hoje)
- `ctx grep`: Scanner + Cache + Output (sem parsing)
- `ctx show-deps`: Scanner + Extractor + refs (sem Ranker)

### Pipeline 2: Busca Semântica (`ctx-search`)

```
Chunker → Indexer → Embedder → Searcher → Reranker (catalog/)
```

Módulos **independentes:**
- `Chunker` não conhece DB (recebe markdown, retorna chunks)
- `Indexer` persiste chunks em SQLite (reutiliza se SHA256 igual)
- `Embedder` chama Ollama uma vez (lazy load, cache)
- `Searcher` combina BM25 + vetorial + RRF (sem Ollama)
- `Reranker` valida top-30 com LLM (opcional, offline)

**Padrão: cada módulo tem uma responsabilidade, não acessa adjacentes direto.**

### Regra Fundamental

> Nenhum módulo deve importar de outro módulo do **mesmo pipeline** diretamente. Orquestração fica exclusivamente em `lib.rs` (para `ctx`) ou `catalog/mod.rs` (para `ctx-search`).

---

## Inteligência Acumulada (Composição)

Quando adicionamos novos comandos, **não perdemos inteligência** — a reutilizamos.

### Exemplo 1: `ctx-search` (implementado)

`ctx-search search` não é grep simples. É:
1. **BM25** sobre chunks (busca por relevância textual)
2. **Busca vetorial** via Ollama (semântica)
3. **RRF fusion** combinando os dois
4. **Reranking** via LLM para validação qualitativa
5. **Token budget** implícito (retorna top-K)
6. **SQLite cache** de embeddings (evita recomputar)

→ Resultado: busca semântica local, offline, rápida.

### Exemplo 2: Futuro `ctx grep` (padrão)

Não seria grep simples. Seria:
1. Scanner encontra arquivos (respeitando .gitignore)
2. Cache avisa quais já foram parseados
3. BM25 prioriza arquivos relevantes
4. Output respeitando token budget

→ Não é cópia da lógica de `ctx`, é **composição de módulos existentes**.

### Estratégias de Redução de Contexto

Todo output deve aplicar ao menos uma (aplicar múltiplas quando possível):

| Estratégia | Aplica a | Exemplo |
|---|---|---|
| **Signature Extraction** | `ctx` | Função inteira (50 tokens) → signature (8 tokens) |
| **Chunking** | `ctx-search` | Doc inteira → chunks semanticamente coerentes |
| **Relevance Ranking** | ambos | BM25 prioriza top-N antes de output |
| **Smart Filtering** | `ctx` | Remove comentários, whitespace, boilerplate |
| **Grouping** | `ctx` | Agrupa assinaturas por arquivo/diretório |
| **Reranking** | `ctx-search` | LLM valida top-30 (qualidade antes de quantidade) |
| **Deduplication** | `ctx` | Colapsa refs repetidas com contadores |
| **Truncation** | `ctx-search` | Mantém heading + início de chunk, corta fim se >256 tokens |

---

## Como Adicionar um Novo Comando (Padrão)

**Exemplo: `ctx grep` (hipotético)**

1. **Defina a invariante:** buscar padrão em código, retornar linhas de matching arquivos respeitando token budget
2. **Compose módulos existentes:** Scanner (encontrar arquivos) + Extractor/Cache (já parsou) + Ranker/BM25 (priorizar relevância) + Output (respeitar budget)
3. **Não duplique lógica:** se `budget.rs` já implementa token fitting, reutilize
4. **Persista reutilizáveis:** se resultado é caro, cacheia em SQLite
5. **Orquestre em `lib.rs`:** adicione função `grep(pattern, dirs, max_tokens)` que compõe módulos
6. **Registre no CLI:** adicione variante a `Cli` struct em `main.rs`

```rust
// Em lib.rs
pub fn grep(pattern: &str, dirs: &[String], max_tokens: usize) -> String {
    let files = scanner::scan_files(dirs);  // Scanner
    let results: Vec<_> = files.par_iter()
        .flat_map(|path| {
            let content = std::fs::read_to_string(path).ok()?;
            extract_matches(&content, pattern, path)  // Extractor lógica
        })
        .collect();
    
    let ranked = bm25::rank(&results, pattern);      // Ranker
    let fitted = budget::fit(&ranked, max_tokens);   // Budget
    output::format_grep(&fitted)                     // Output
}
```

---

## Como Estender `ctx-search` (Novos Subcomandos)

**Exemplo: `ctx-search upsert` (sincronizar com repo externo)**

1. **Adicione submenu em `bin/ctx_search.rs`:**

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existentes (add, index, search)
    #[command(about = "Sincronizar acervo com repositório")]
    Upsert {
        #[arg(help = "Nome do acervo")]
        name: String,
    },
}
```

2. **Implemente handler:**

```rust
Commands::Upsert { name } => {
    let collection = catalog::get_collection(&name)?;
    // Sincronizar com source (git pull, download, etc)
    catalog::index(&name)?;  // Re-index
    println!("Collection updated");
}
```

3. **Persista em SQLite:** `catalog::store` já suporta update-on-conflict

**Padrão:** cada subcomando usa API pública de `catalog/mod.rs` (add_collection, index, search, etc). Não reimplementa lógica.

---

## Como Adicionar Suporte a Nova Linguagem

1. **Crie `src/extractors/<lang>.rs`:**

```rust
pub fn extract_signatures(source: &str) -> Vec<Signature> {
    // Parse com Tree-sitter, retorna assinaturas
}

pub fn extract_refs(source: &str) -> Vec<String> {
    // Extrai referências (imports, calls, etc)
}
```

2. **Registre em `src/extractors/mod.rs`:**

```rust
".java" => {
    Ok(java::extract_signatures(content))
}
```

3. **Se precisar grammar customizada (como Groovy):**
   - Adicione em `grammars/<lang>/`
   - Registre build em `build.rs`

4. **Teste em `tests/integration.rs`:**

```rust
#[test]
fn test_java_extraction() {
    let src = "public class Main { ... }";
    let sigs = java::extract_signatures(src);
    assert!(!sigs.is_empty());
}
```

**Regra:** Extractor = **puro** (não acessa I/O, cache, pipeline). Recebe string, retorna signatures. Testável isoladamente.

---

## Padrões de Código

### Imutabilidade

Criar novos objetos, **nunca** mutar. Ver `.claude/rules/common/coding-style.md`.

### Paralelismo

Use `rayon` para operações sobre coleções (scan, parse em `ctx`). **Não use threads manuais.**

Para `ctx-search`, Ollama é blocking — não paralelize chamadas (pool de connections é suficiente).

### Erros

Propague com `?`. **Nunca silenciar erros.** Mensagens devem ser acionáveis:

```rust
// ❌ Ruim
Err("parse error")

// ✅ Bom
Err(format!("Failed to parse {}: {}", path.display(), err))
```

### Tamanho de arquivo

Máximo 800 linhas. Se ficando grande:
- Extractors: extraia helpers privados (parse_function, parse_class)
- Searcher: separe BM25 e ranking em submódulos
- Store: já está bem separado (schema em constants)

### Async/Await

`ctx-search` usa tokio + reqwest para Ollama. **Não misture async com rayon** (deadlock).

Layout: `src/catalog/embedder.rs` é tokio (async), tudo mais é sync.

### Database Patterns

**SQLite** (ambos pipelines):
- Use parameterized queries (evita injection)
- Transações para operações em lote
- PRAGMA: `journal_mode=wal` (melhor concorrência)
- Cheque cache hit ANTES de operação custosa

```rust
// ✅ Padrão
let key = sha256(content);
if let Some(cached) = cache_get(&key) {
    return cached;  // Hit
}
let result = expensive_operation();
cache_set(&key, &result);  // Miss → compute → store
```

---

## Referências

- **`docs/arquitetura.md`** — pipeline, CLI, troubleshooting
- **`docs/produto.md`** — visão de negócio, 3 horizontes
- **`docs/especificacao-rag.md`** — regras de negócio do catalog
- **`docs/pesquisa/`** — decisões técnicas, estado da arte
- **`.claude/rules/common/coding-style.md`** — imutabilidade, linting, erros
