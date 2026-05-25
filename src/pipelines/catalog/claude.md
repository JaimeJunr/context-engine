# Instruções: `src/catalog/` — RAG Local (Busca Semântica)

Módulo de **busca semântica local** — indexa documentação em chunks, gera embeddings vetoriais e permite busca por intenção (não apenas palavras-chave).

## Estrutura

```
catalog/
├── mod.rs         # API pública (add_collection, index, search, health)
├── types.rs       # Tipos (Collection, SearchResult, CollectionHealth)
├── store.rs       # Persistência SQLite (collections, docs, chunks, embeddings)
├── cache_ops.rs   # Operações em cache (read, write, invalidate)
├── chunker.rs     # Divisão de documentos em chunks semanticamente coerentes
├── embedder.rs    # Geração de embeddings via endpoint OpenAI-compatible
├── indexer.rs     # Pipeline (chunking → embedding → persistência)
├── searcher.rs    # Busca vetorial por similaridade cosine
└── reranker.rs    # Re-ranking contextual dos resultados
```

## Fluxo

```
Documentos (markdown, PDF, etc)
    ↓ (chunker.rs)
Chunks semanticamente coerentes (~500 tokens cada)
    ↓ (embedder.rs)
Embeddings vetoriais (OpenAI-compatible: nomic-embed-text, etc)
    ↓ (store.rs)
SQLite + vector extension (pgvector-like indexing)
    ↓ (searcher.rs)
Busca por similaridade cosine
    ↓ (reranker.rs)
Re-ranking contextual + cross-encoder
    ↓
Top-K resultados
```

## API Pública (mod.rs)

```rust
pub fn add_collection(
    name: &str,
    source_path: &Path,
    include_glob: Option<&str>,
    exclude_glob: Option<&str>,
) -> Result<()>

pub fn index(
    collection_name: &str,
    with_embed: bool,
) -> Result<IndexResult>

pub fn embed(
    collection_name: &str,
    batch_size: usize,
) -> Result<EmbedResult>

pub fn search(
    collection_name: &str,
    query: &str,
    top_k: usize,
) -> Result<Vec<SearchResult>>

pub fn health(collection_name: &str) -> Result<CollectionHealth>
```

## Tipos (types.rs)

```rust
pub struct Collection {
    pub id: String,
    pub name: String,
    pub source_path: PathBuf,
    pub include_glob: Option<String>,
    pub exclude_glob: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_indexed: Option<DateTime<Utc>>,
}

pub struct SearchResult {
    pub chunk_id: String,
    pub doc_id: String,
    pub doc_path: String,
    pub snippet: String,  // first 200 chars
    pub full_text: String,
    pub score: f32,  // 0..1 similarity
    pub line_start: usize,
    pub line_end: usize,
}

pub struct CollectionHealth {
    pub total_docs: usize,
    pub indexed_docs: usize,
    pub chunks_total: usize,
    pub embeddings_pending: usize,
    pub last_indexed: Option<DateTime<Utc>>,
}
```

## Chunking (chunker.rs)

Divide documentos em chunks **semanticamente coerentes** (~500 tokens cada).

```rust
pub fn chunk_document(
    content: &str,
    max_chunk_size: usize,
) -> Vec<Chunk>
```

**Estratégia:**
1. Split por parágrafos/seções
2. Se seção > max_chunk_size, split por sentences
3. Se ainda > max_chunk_size, split por linhas
4. Overlap de 50 tokens entre chunks para contexto

**Exemplo:**

```
Documento: 2000 chars
    ↓ chunk_size=500
Chunk 1: 450 chars (parágrafo 1)
Chunk 2: 480 chars (parágrafo 2, overlap 50 chars com chunk 1)
Chunk 3: 420 chars (parágrafo 3, overlap 50 chars com chunk 2)
Chunk 4: 350 chars (resto, overlap 50 chars com chunk 3)
```

Implementação:

```rust
pub fn chunk_document(content: &str, max_chunk_size: usize) -> Vec<Chunk> {
    let paragraphs = content.split("\n\n");
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    
    for para in paragraphs {
        if current_chunk.len() + para.len() > max_chunk_size {
            chunks.push(Chunk { text: current_chunk.clone() });
            // overlap
            let overlap_tokens = max_chunk_size / 2;
            current_chunk = current_chunk[current_chunk.len() - overlap_tokens..].to_string();
        }
        current_chunk.push_str(para);
    }
    
    chunks.push(Chunk { text: current_chunk });
    chunks
}
```

## Embeddings (embedder.rs)

Gera **embeddings vetoriais** via endpoint OpenAI-compatible (ex: Ollama local).

```rust
pub fn embed_chunk(text: &str) -> Result<Vec<f32>> {
    let client = OpenAIClient::new()?;
    let embedding = client.embeddings(
        model: "nomic-embed-text",
        input: vec![text],
    )?;
    Ok(embedding.data[0].embedding)
}
```

**Endpoint configurável em `src/config.rs`:**

```rust
pub struct EmbedderConfig {
    pub endpoint: String,           // default: http://localhost:11434
    pub model: String,              // default: nomic-embed-text
    pub batch_size: usize,          // default: 10
}
```

**Exemplo com Ollama local:**

```bash
ollama run nomic-embed-text
# Server runs at http://localhost:11434
```

## Store (store.rs)

Persistência em SQLite com suporte a busca vetorial.

```rust
pub struct Store {
    db: Connection,
}

impl Store {
    pub fn new(db_path: &Path) -> Result<Self>
    
    pub fn add_collection(&self, col: &Collection) -> Result<()>
    
    pub fn add_document(&self, col_id: &str, doc: &Document) -> Result<()>
    
    pub fn add_chunks(&self, doc_id: &str, chunks: &[Chunk]) -> Result<()>
    
    pub fn add_embeddings(
        &self,
        chunk_id: &str,
        embedding: &[f32],
    ) -> Result<()>
    
    pub fn search_by_similarity(
        &self,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>>
    
    pub fn pending_chunks(&self, col_id: &str) -> Result<Vec<Chunk>>
}
```

**Schema:**

```sql
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    source_path TEXT,
    created_at DATETIME,
    last_indexed DATETIME
);

CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    collection_id TEXT,
    path TEXT,
    content TEXT,
    FOREIGN KEY(collection_id) REFERENCES collections(id)
);

CREATE TABLE chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT,
    content TEXT,
    line_start INTEGER,
    line_end INTEGER,
    FOREIGN KEY(document_id) REFERENCES documents(id)
);

CREATE TABLE embeddings (
    id TEXT PRIMARY KEY,
    chunk_id TEXT,
    vector BLOB,  -- serialized f32 array
    FOREIGN KEY(chunk_id) REFERENCES chunks(id)
);

-- Vector similarity search (requires extension)
CREATE VIRTUAL TABLE embeddings_index USING vec0(
    id TEXT,
    embedding float[1536]
);
```

## Indexação (indexer.rs)

Pipeline: Descoberta de arquivos → Chunking → Embedding → Persistência

```rust
pub fn index_collection(
    collection_name: &str,
    embedder: &Embedder,
    with_embed: bool,
) -> Result<IndexResult> {
    let store = Store::open()?;
    let col = store.get_collection(collection_name)?;
    
    // 1. Discover files
    let files = discover_files(&col.source_path, &col.include_glob)?;
    
    // 2. Filter: only new/modified files
    let modified_files = files.into_iter()
        .filter(|f| !store.document_exists(&f.path))
        .collect::<Vec<_>>();
    
    // 3. Chunk documents
    for file in &modified_files {
        let chunks = chunker::chunk_document(&file.content)?;
        store.add_chunks(&file.id, &chunks)?;
    }
    
    // 4. (Optional) Embed chunks
    if with_embed {
        let pending = store.pending_chunks(collection_name)?;
        for chunk in pending {
            let embedding = embedder.embed(&chunk.text)?;
            store.add_embeddings(&chunk.id, &embedding)?;
        }
    }
    
    Ok(IndexResult {
        docs_added: modified_files.len(),
        chunks_added: total_chunks,
        embeddings_generated: if with_embed { total_chunks } else { 0 },
    })
}
```

## Busca (searcher.rs)

Busca **vetorial por similaridade** (cosine similarity).

```rust
pub fn search(
    store: &Store,
    query: &str,
    top_k: usize,
) -> Result<Vec<SearchResult>> {
    let embedder = Embedder::new()?;
    
    // 1. Embed query
    let query_embedding = embedder.embed(query)?;
    
    // 2. Vector similarity search
    let results = store.search_by_similarity(&query_embedding, top_k)?;
    
    Ok(results)
}
```

**Cosine Similarity:**

```
similarity(a, b) = dot(a, b) / (||a|| * ||b||)
```

## Re-ranking (reranker.rs)

**Re-ranking contextual** dos resultados com base em:
1. Proximidade semântica
2. Densidade de termos da query
3. Cross-encoder scores

```rust
pub fn rerank(
    results: &[SearchResult],
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>> {
    let mut scored = results.iter()
        .map(|r| {
            let semantic_score = r.score;  // from embeddings
            let term_density = compute_term_density(query, &r.full_text);
            let combined = 0.7 * semantic_score + 0.3 * term_density;
            (r.clone(), combined)
        })
        .collect::<Vec<_>>();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    Ok(scored.into_iter()
        .take(max_results)
        .map(|(r, _)| r)
        .collect())
}
```

## Testes

```bash
cargo test catalog::
cargo test catalog::chunker::tests
cargo test catalog::searcher::tests
cargo test catalog::store::tests
```

### Teste Padrão

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_semantic_boundaries() {
        let doc = "Section 1\n\nPara 1.1\n\nPara 1.2\n\nSection 2\n\nPara 2.1";
        let chunks = chunk_document(doc, 500);
        
        // Verify chunks respect paragraph boundaries
        for chunk in chunks {
            assert!(!chunk.text.contains("\n\n\n"));  // no orphaned newlines
        }
    }

    #[test]
    fn test_search_returns_relevant_results() {
        let store = Store::open(":memory:")?;
        add_test_collection(&store)?;
        
        let results = search(&store, "authentication", 10)?;
        assert!(!results.is_empty());
        assert!(results[0].score > 0.5);  // reasonable similarity
    }
}
```

---

**Última atualização**: 2026-04-14
