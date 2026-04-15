// Módulo de recuperação semântica de conhecimento
// Implementa a especificação funcional em equipe-suja/especificacao-funcional-qmd.md

pub mod cache_ops;
pub mod chunker;
pub mod embedder;
pub mod indexer;
pub mod reranker;
pub mod searcher;
pub mod store;
pub mod types;

use anyhow::Result;
use indexer::IndexStats;
use store::collection_health;
use types::{Collection, CollectionHealth, SearchResult};

// Registra ou atualiza um acervo (RD-17)
pub fn add_collection(col: Collection) -> Result<()> {
    // Sincroniza contextos de path no banco
    store::upsert_collection(&col)?;
    for ctx in &col.path_contexts {
        store::upsert_path_context(&col.name, ctx)?;
    }
    Ok(())
}

// Retorna acervo pelo nome
pub fn get_collection(name: &str) -> Result<Option<Collection>> {
    store::get_collection(name)
}

// Lista todos os acervos com timestamp da última indexação (RD-18)
pub fn list_collections() -> Result<Vec<(String, Option<String>)>> {
    store::list_collections()
}

// Executa pipeline de catalogação (RD-01 a RD-05)
pub fn index(name: &str) -> Result<IndexStats> {
    let col = store::get_collection(name)?
        .ok_or_else(|| anyhow::anyhow!("acervo '{}' não encontrado", name))?;
    indexer::index_collection(&col)
}

// Gera embeddings para chunks pendentes (RD-06, RD-07, RD-08)
pub fn embed_pending(name: &str, batch_size: usize) -> Result<usize> {
    let col = store::get_collection(name)?
        .ok_or_else(|| anyhow::anyhow!("acervo '{}' não encontrado", name))?;

    let model = col.embedder_model_or_default();
    let endpoint = col.llm_endpoint_resolved();
    let pending = store::get_pending_chunks(name, batch_size)?;
    let count = pending.len();

    if count == 0 {
        return Ok(0);
    }

    // RD-07: entrada = chunk + título do documento como âncora
    // Paraleliza computação de embeddings (antes da atualização serial)
    use rayon::prelude::*;

    let embeddings: Result<Vec<(i64, Vec<f32>)>> = pending
        .par_iter()
        .map(|chunk| {
            let doc_path = store::get_doc_path(chunk.doc_id)?.unwrap_or_default();
            let title = embedder::doc_title_from_path(&doc_path);
            let input = embedder::format_embed_input(&title, &chunk.content);
            let embedding = embedder::embed_text(&model, &input, endpoint.as_deref())?;
            Ok((chunk.id, embedding))
        })
        .collect();

    // Atualiza sequencialmente após computação paralela (store I/O é serial)
    for (chunk_id, embedding) in embeddings? {
        store::update_chunk_embedding(chunk_id, &embedding)?;
    }

    Ok(count)
}

// Busca semântica no acervo (RD-09 a RD-16)
pub fn search(name: &str, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
    let col = store::get_collection(name)?
        .ok_or_else(|| anyhow::anyhow!("acervo '{}' não encontrado", name))?;
    searcher::search(&col, query, top_k)
}

// Relatório de saúde do acervo (seção 4.7)
pub fn health(name: &str) -> Result<CollectionHealth> {
    collection_health(name)
}

/// Estatísticas retornadas pelo bootstrap
pub struct BootstrapStats {
    pub collection_name: String,
    pub files_discovered: usize,
    pub chunks_indexed: usize,
}

/// Registra e indexa um diretório de documentação em um único passo.
///
/// Equivalente a `ctx add` + `ctx index`, derivando o nome da collection do
/// nome do diretório quando não fornecido.
pub fn bootstrap(path: &std::path::Path, name: Option<&str>) -> Result<BootstrapStats> {
    let collection_name = name
        .map(|s| s.to_string())
        .or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "não foi possível derivar nome da collection de '{}'",
                path.display()
            )
        })?;

    let col = types::Collection {
        name: collection_name.clone(),
        sources: vec![path.to_string_lossy().to_string()],
        include_patterns: vec![
            "**/*.md".to_string(),
            "**/*.txt".to_string(),
            "**/*.rst".to_string(),
            "**/README*".to_string(),
            "**/CHANGELOG*".to_string(),
        ],
        exclude_patterns: vec![],
        path_contexts: vec![],
        pre_index_cmd: None,
        embedder_model: None,
        reranker_model: None,
        llm_endpoint: None,
    };

    add_collection(col)?;
    let stats = index(&collection_name)?;

    Ok(BootstrapStats {
        collection_name,
        files_discovered: stats.scanned,
        chunks_indexed: stats.indexed,
    })
}

// Compactação do repositório interno (RD-30)
pub fn compact(name: &str) -> Result<()> {
    // Valida que o acervo existe
    store::get_collection(name)?
        .ok_or_else(|| anyhow::anyhow!("acervo '{}' não encontrado", name))?;
    store::compact()
}
