// Pipeline `map` — gera mapa curado de repositório para LLMs.
//
// Fluxo: Scanner → Extractor (Tree-Sitter) → Cache → Ranking (BM25 + PPR) → Budget → Output.

pub mod extractors;
pub mod output;
pub mod ranking;
pub mod scanner;

use std::collections::HashMap;
use std::path::PathBuf;

use rayon::prelude::*;
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::shared::cache::{
    self, batch_write, cache_get, cache_get_refs, cache_key, map_cache_get, map_cache_set,
};
use extractors::{extract_refs, extract_signatures};

fn clean_query(title: &str) -> String {
    let re = Regex::new(r"^\[[^\]]+\]\s*").unwrap();
    re.replace(title, "").trim().to_string()
}

fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(&hasher.finalize()[..])
}

pub fn run(
    title: &str,
    dirs: &[String],
    top_n: usize,
    max_tokens: usize,
    seeds: Option<&[String]>,
    output_format: &str,
    max_depth: usize,
) -> String {
    // Verificar cache de mapa por commit SHA
    let cache_key_map = {
        let seeds_str = seeds.map(|s| s.join(",")).unwrap_or_default();
        let raw = format!(
            "{}|{}|{}|{}|{}|{}",
            cache::git_commit_key(dirs),
            dirs.join(","),
            title,
            top_n,
            max_tokens,
            seeds_str,
        );
        sha256_bytes(raw.as_bytes())
    };

    if let Some(cached) = map_cache_get(&cache_key_map) {
        return cached;
    }

    let files = scanner::scan_files(dirs, max_depth);
    if files.is_empty() {
        return String::new();
    }

    // Parallel parse + cache
    let results: Vec<(PathBuf, Vec<String>, String, Vec<String>)> = files
        .par_iter()
        .map(|path| {
            let key = cache_key(path);
            let sigs = if let Some(cached) = cache_get(&key) {
                cached
            } else {
                extract_signatures(path)
            };
            let ref_key = format!("r:{}", key);
            let refs = if let Some(cached) = cache_get_refs(&ref_key) {
                cached
            } else {
                extract_refs(path)
            };
            (path.clone(), sigs, key, refs)
        })
        .collect();

    // Batch write to cache
    let cache_items: Vec<(String, Vec<String>)> = results
        .iter()
        .map(|(_, sigs, key, _)| (key.clone(), sigs.clone()))
        .collect();
    let refs_items: Vec<(String, Vec<String>)> = results
        .iter()
        .map(|(_, _, key, refs)| (format!("r:{}", key), refs.clone()))
        .collect();
    batch_write(cache_items, refs_items);

    // Build corpus (only files with signatures)
    let corpus: Vec<(PathBuf, Vec<String>)> = results
        .into_iter()
        .filter_map(|(path, sigs, _, _)| {
            if sigs.is_empty() {
                None
            } else {
                Some((path, sigs))
            }
        })
        .collect();

    if corpus.is_empty() {
        return String::new();
    }

    let query = clean_query(title);
    let all_ranked = ranking::bm25::bm25_rank(&query, &corpus, 0);

    let ranked = if top_n > 0 {
        all_ranked[..top_n.min(all_ranked.len())].to_vec()
    } else if let Some(seed_dirs) = seeds {
        build_pagerank_ranked(&corpus, seed_dirs, &all_ranked, max_tokens, dirs)
    } else {
        ranking::budget::fit_to_budget(&all_ranked, dirs, max_tokens)
    };

    if let Some(hint) = ranking::diagnostics::boilerplate_hint(&ranked, top_n.max(1)) {
        eprintln!("{}", hint);
    }

    if let Some(hint) = ranking::diagnostics::low_variance_hint(&ranked, top_n.max(10)) {
        eprintln!("{}", hint);
    }

    let result = if output_format == "json" {
        output::format_json(&ranked)
    } else {
        output::format_repo_map(&ranked, dirs)
    };

    map_cache_set(&cache_key_map, &result);
    result
}

fn build_pagerank_ranked(
    corpus: &[(PathBuf, Vec<String>)],
    seed_dirs: &[String],
    bm25_top: &[(PathBuf, f64)],
    max_tokens: usize,
    base_dirs: &[String],
) -> Vec<(PathBuf, f64)> {
    // Build symbol → file index
    let mut symbol_to_files: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for (path, sigs) in corpus {
        for sig in sigs {
            // Extract symbol name (last word before parens/spaces)
            let sym = sig.split_whitespace().last().unwrap_or("").to_string();
            let sym = sym
                .trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if !sym.is_empty() {
                symbol_to_files.entry(sym).or_default().push(path.clone());
            }
        }
    }

    let (graph, predecessors) = ranking::pagerank::build_graph(corpus, &symbol_to_files);

    if graph.is_empty() {
        return ranking::budget::fit_to_budget(bm25_top, base_dirs, max_tokens);
    }

    let all_nodes: Vec<PathBuf> = graph.keys().cloned().collect();
    if all_nodes.is_empty() {
        return ranking::budget::fit_to_budget(bm25_top, base_dirs, max_tokens);
    }

    // Build personalization vector
    let mut personalization: HashMap<PathBuf, f64> =
        all_nodes.iter().map(|n| (n.clone(), 1.0)).collect();

    for (path, _) in bm25_top.iter().take(5) {
        if let Some(v) = personalization.get_mut(path) {
            *v = v.max(10.0);
        }
    }

    for path in &all_nodes {
        for seed in seed_dirs {
            if path.starts_with(seed) {
                if let Some(v) = personalization.get_mut(path) {
                    *v = v.max(50.0);
                }
                break;
            }
        }
    }

    let total: f64 = personalization.values().sum();
    let norm: HashMap<PathBuf, f64> = personalization
        .into_iter()
        .map(|(k, v)| (k, v / total))
        .collect();

    let scores = ranking::pagerank::pagerank_pure(&graph, &predecessors, &norm, 0.85, 200, 1e-6);

    let mut ranked_ppr: Vec<(PathBuf, f64)> = scores.into_iter().collect();
    ranked_ppr.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    ranking::budget::fit_to_budget(&ranked_ppr, base_dirs, max_tokens)
}
