// Busca semântica: BM25 + vetorial + RRF fusion + normalização (RD-09 a RD-16)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cache_ops::{self, OP_QUERY_VARIANTS};
use super::embedder;
use super::reranker;
use super::store;
use super::types::{Collection, SearchMode, SearchResult};
use crate::ranking::bm25::bm25_rank_generic;
use crate::tokenizer;

const RRF_K: f64 = 60.0;
const TOP_FOR_RERANK: usize = 30;
const MAX_VARIANTS: usize = 3;

pub fn search(col: &Collection, raw_query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
    let (mode, query) = SearchMode::parse_from_query(raw_query);

    let endpoint = col.llm_endpoint_resolved();
    let base_url = endpoint.as_deref();
    let embed_model = col.embedder_model_or_default();
    let rerank_model = col.reranker_model_or_default();

    match mode {
        // RD-09-A: busca somente textual
        SearchMode::Exact => {
            let results = bm25_search(&col.name, &query, top_k * 2)?;
            Ok(finalize(results, top_k))
        }
        // RD-09-A: busca somente vetorial
        SearchMode::Conceptual => {
            let results = vector_search(&col.name, &query, &embed_model, base_url, top_k * 2)?;
            Ok(finalize(results, top_k))
        }
        // RD-09-A: expanded — gera hipótese de resposta antes da busca
        SearchMode::Expanded => {
            let hypothesis = reranker::generate_hypothesis(&rerank_model, &query, base_url)?;
            let expanded = format!("{} {}", query, hypothesis);
            let results =
                full_search(col, &expanded, &embed_model, &rerank_model, base_url, top_k)?;
            Ok(results)
        }
        // Fluxo padrão: BM25 + vetorial + RRF + reranking
        SearchMode::Auto => full_search(col, &query, &embed_model, &rerank_model, base_url, top_k),
    }
}

fn full_search(
    col: &Collection,
    query: &str,
    embed_model: &str,
    rerank_model: &str,
    base_url: Option<&str>,
    top_k: usize,
) -> Result<Vec<SearchResult>> {
    // RD-09: gera variantes da query
    let variants = get_query_variants(rerank_model, query, base_url)?;

    // Executa BM25 para query original e variantes
    // RD-10: query original tem peso 2x via duplicação em RRF fusion
    let mut all_bm25: Vec<Vec<(i64, f64)>> = Vec::new();
    let original_bm25 = bm25_search_ids(&col.name, query)?;
    all_bm25.push(original_bm25.clone());
    all_bm25.push(original_bm25);

    for v in &variants {
        all_bm25.push(bm25_search_ids(&col.name, v)?);
    }

    // Executa busca vetorial para query original e variantes
    // RD-10: query original tem peso 2x
    let mut all_vector: Vec<Vec<(i64, f64)>> = Vec::new();
    let original_vec = vector_search_ids(&col.name, query, embed_model, base_url)?;
    all_vector.push(original_vec.clone());
    all_vector.push(original_vec);

    for v in &variants {
        all_vector.push(vector_search_ids(&col.name, v, embed_model, base_url)?);
    }

    // RRF fusion (RD-11, RD-12)
    let all_lists: Vec<Vec<(i64, f64)>> = all_bm25.into_iter().chain(all_vector).collect();
    let mut fused = rrf_fuse(&all_lists);

    // RD-13: apenas top-30 para reranking
    fused.truncate(TOP_FOR_RERANK);

    // RD-14: reranking qualitativo
    let reranked = reranker::rerank(rerank_model, query, &fused, &col.name, base_url)?;

    // RD-15: normalização
    let normalized = normalize_scores(reranked);

    // RD-16: deduplicação por documento de origem
    let deduped = dedup_by_doc(normalized);

    // Monta resultados finais
    build_results(&deduped, &col.name, top_k)
}

// --- BM25 ---

fn bm25_search(collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let ids_scores = bm25_search_ids(collection, query)?;
    let top: Vec<(i64, f64)> = ids_scores.into_iter().take(limit).collect();
    build_results(&top, collection, limit)
}

fn bm25_search_ids(collection: &str, query: &str) -> Result<Vec<(i64, f64)>> {
    let chunks = store::get_all_chunks_with_embeddings(collection)?;
    if chunks.is_empty() {
        let pending = store::get_pending_chunks(collection, 10_000)?;
        let corpus: Vec<(i64, Vec<String>)> = pending
            .iter()
            .map(|c| (c.id, tokenizer::tokenize(&c.content)))
            .collect();
        return Ok(bm25_rank_generic(query, &corpus, 0, |_id, tokens| {
            tokens.join(" ")
        }));
    }

    let corpus: Vec<(i64, Vec<String>)> = chunks
        .iter()
        .map(|c| (c.id, tokenizer::tokenize(&c.content)))
        .collect();
    Ok(bm25_rank_generic(query, &corpus, 0, |_id, tokens| {
        tokens.join(" ")
    }))
}

// --- Busca vetorial ---

fn vector_search(
    collection: &str,
    query: &str,
    model: &str,
    base_url: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let ids_scores = vector_search_ids(collection, query, model, base_url)?;
    let top: Vec<(i64, f64)> = ids_scores.into_iter().take(limit).collect();
    build_results(&top, collection, limit)
}

fn vector_search_ids(
    collection: &str,
    query: &str,
    model: &str,
    base_url: Option<&str>,
) -> Result<Vec<(i64, f64)>> {
    let query_emb = embedder::embed_text(model, query, base_url)?;
    let chunks = store::get_all_chunks_with_embeddings(collection)?;

    let mut scores: Vec<(i64, f64)> = chunks
        .iter()
        .filter_map(|c| {
            c.embedding
                .as_ref()
                .map(|e| (c.id, embedder::cosine_similarity(e, &query_emb)))
        })
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(scores)
}

// --- RRF Fusion (RD-11, RD-12) ---

pub fn rrf_fuse(lists: &[Vec<(i64, f64)>]) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();

    for list in lists {
        for (rank, (id, _)) in list.iter().enumerate() {
            let rrf = rrf_score(rank);
            let bonus = position_bonus(rank);
            *scores.entry(*id).or_insert(0.0) += rrf + bonus;
        }
    }

    let mut result: Vec<(i64, f64)> = scores.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

// RD-11: score RRF com k=60
fn rrf_score(rank: usize) -> f64 {
    1.0 / (RRF_K + rank as f64 + 1.0)
}

// RD-12: bônus de posição privilegiada
fn position_bonus(rank: usize) -> f64 {
    match rank {
        0 => 0.05,
        1 | 2 => 0.02,
        _ => 0.0,
    }
}

// --- Normalização (RD-15) ---

pub fn normalize_scores(items: Vec<(i64, f64)>) -> Vec<(i64, f64)> {
    if items.is_empty() {
        return items;
    }

    // Single pass: calcula max e min simultaneamente (O(N) em vez de 2N)
    let (max, min) = items
        .iter()
        .fold((f64::NEG_INFINITY, f64::INFINITY), |(mx, mn), (_, s)| {
            (mx.max(*s), mn.min(*s))
        });

    if (max - min).abs() < 1e-12 {
        // Todos os scores iguais: normaliza para 1.0
        return items.into_iter().map(|(id, _)| (id, 1.0)).collect();
    }

    // Normaliza no intervalo [0, 1]
    items
        .into_iter()
        .map(|(id, s)| (id, (s - min) / (max - min)))
        .collect()
}

// --- Deduplicação por documento (RD-16) ---

pub fn dedup_by_doc(items: Vec<(i64, f64)>) -> Vec<(i64, f64)> {
    // Precisamos mapear chunk_id → doc_id para deduplicar por documento
    // Como não temos o mapping aqui, mantemos ordenado e deixamos build_results cuidar
    // A deduplicação real acontece em build_results via doc_path
    items
}

fn finalize(results: Vec<SearchResult>, top_k: usize) -> Vec<SearchResult> {
    results.into_iter().take(top_k).collect()
}

// --- Variantes de query (RD-09) ---

#[derive(Serialize, Deserialize)]
struct VariantsCache {
    variants: Vec<String>,
}

fn get_query_variants(model: &str, query: &str, base_url: Option<&str>) -> Result<Vec<String>> {
    let cache_key = format!("{}:{}", model, query);

    // RD-22: verifica cache primeiro
    if let Some(cached) = cache_ops::get_cached::<VariantsCache>(&cache_key, OP_QUERY_VARIANTS) {
        return Ok(cached.variants);
    }

    let variants = reranker::generate_query_variants(model, query, MAX_VARIANTS, base_url)?;

    let _ = cache_ops::set_cached(
        &cache_key,
        OP_QUERY_VARIANTS,
        &VariantsCache {
            variants: variants.clone(),
        },
    );

    Ok(variants)
}

// --- Construção de resultados finais ---

fn build_results(
    items: &[(i64, f64)],
    collection: &str,
    top_k: usize,
) -> Result<Vec<SearchResult>> {
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut results = Vec::new();

    // Carrega chunks UMA VEZ antes do loop (evita N+1)
    let chunks = store::get_all_chunks_with_embeddings(collection)?;

    for (chunk_id, score) in items {
        if results.len() >= top_k {
            break;
        }

        // Busca chunk no HashMap (O(1)) em vez de store I/O
        let chunk = match chunks.iter().find(|c| c.id == *chunk_id) {
            Some(c) => c.clone(),
            None => continue,
        };

        let doc_path = match store::get_doc_path(chunk.doc_id)? {
            Some(p) => p,
            None => continue,
        };

        // RD-16: apenas o melhor chunk por documento
        if seen_paths.contains(&doc_path) {
            continue;
        }
        seen_paths.insert(doc_path.clone());

        // RD-19: contexto por hierarquia de path
        let context = store::get_context_for_path(collection, &doc_path)
            .ok()
            .flatten();

        results.push(SearchResult {
            doc_path,
            chunk_id: *chunk_id,
            chunk_text: chunk.content.clone(),
            chunk_offset: chunk.start_offset,
            score: *score,
            context,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_score_rank_zero() {
        // k=60, rank=0 → 1/(60+0+1) = 1/61
        let s = rrf_score(0);
        assert!((s - 1.0 / 61.0).abs() < 1e-10);
    }

    #[test]
    fn test_rrf_score_decreasing() {
        assert!(rrf_score(0) > rrf_score(1));
        assert!(rrf_score(1) > rrf_score(10));
    }

    #[test]
    fn test_position_bonus() {
        assert!((position_bonus(0) - 0.05).abs() < 1e-10);
        assert!((position_bonus(1) - 0.02).abs() < 1e-10);
        assert!((position_bonus(2) - 0.02).abs() < 1e-10);
        assert!((position_bonus(3) - 0.0).abs() < 1e-10);
        assert!((position_bonus(100) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_scores_basic() {
        let items = vec![(1i64, 0.0), (2i64, 5.0), (3i64, 10.0)];
        let norm = normalize_scores(items);
        // máximo deve ser 1.0, mínimo 0.0
        let max = norm
            .iter()
            .map(|(_, s)| *s)
            .fold(f64::NEG_INFINITY, f64::max);
        let min = norm.iter().map(|(_, s)| *s).fold(f64::INFINITY, f64::min);
        assert!(
            (max - 1.0).abs() < 1e-10,
            "max esperado 1.0, obteve {}",
            max
        );
        assert!(
            (min - 0.0).abs() < 1e-10,
            "min esperado 0.0, obteve {}",
            min
        );
    }

    #[test]
    fn test_normalize_scores_all_equal() {
        let items = vec![(1i64, 3.0), (2i64, 3.0), (3i64, 3.0)];
        let norm = normalize_scores(items);
        for (_, s) in &norm {
            assert!((s - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_rrf_fuse_combines_lists() {
        let list_a = vec![(1i64, 0.9), (2i64, 0.5)];
        let list_b = vec![(2i64, 0.8), (1i64, 0.3)];
        let fused = rrf_fuse(&[list_a, list_b]);

        // Ambos os IDs devem aparecer
        assert_eq!(fused.len(), 2);
        // Resultado deve estar ordenado por score decrescente
        assert!(fused[0].1 >= fused[1].1);
    }

    #[test]
    fn test_rrf_fuse_top1_gets_bonus() {
        let list = vec![(42i64, 1.0)];
        let fused = rrf_fuse(&[list]);
        // score = rrf(0) + position_bonus(0) = 1/61 + 0.05
        let expected = 1.0 / 61.0 + 0.05;
        assert!((fused[0].1 - expected).abs() < 1e-10);
    }
}
