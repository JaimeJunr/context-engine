use crate::tokenizer::tokenize;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn bm25_rank(
    query: &str,
    corpus: &[(PathBuf, Vec<String>)],
    top_n: usize,
) -> Vec<(PathBuf, f64)> {
    let k1: f64 = 1.5;
    let b: f64 = 0.75;

    let docs: Vec<Vec<String>> = corpus
        .iter()
        .map(|(path, sigs)| {
            let doc_text = format!("{} {}", path.display(), sigs.join(" "));
            tokenize(&doc_text)
        })
        .collect();

    let query_terms: std::collections::HashSet<String> = tokenize(query).into_iter().collect();
    if query_terms.is_empty() {
        return vec![];
    }

    let n = docs.len();
    if n == 0 {
        return vec![];
    }

    let avg_dl = docs.iter().map(|d| d.len() as f64).sum::<f64>() / n as f64;

    let mut idf: HashMap<String, f64> = HashMap::new();
    for term in &query_terms {
        let df = docs.iter().filter(|d| d.contains(term)).count() as f64;
        idf.insert(
            term.clone(),
            ((n as f64 - df + 0.5) / (df + 0.5) + 1.0).ln(),
        );
    }

    let mut scores: Vec<(PathBuf, f64)> = corpus
        .iter()
        .zip(docs.iter())
        .map(|((path, _), doc)| {
            let dl = doc.len() as f64;
            let mut freq: HashMap<&str, usize> = HashMap::new();
            for t in doc {
                *freq.entry(t.as_str()).or_default() += 1;
            }
            let score: f64 = query_terms
                .iter()
                .filter_map(|term| {
                    let tf = *freq.get(term.as_str()).unwrap_or(&0) as f64;
                    if tf == 0.0 {
                        return None;
                    }
                    let idf_val = *idf.get(term).unwrap_or(&0.0);
                    let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * dl / avg_dl));
                    Some(idf_val * tf_norm)
                })
                .sum();
            (path.clone(), score)
        })
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if top_n > 0 {
        scores.truncate(top_n);
    }
    scores
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus(items: &[(&str, &[&str])]) -> Vec<(PathBuf, Vec<String>)> {
        items
            .iter()
            .map(|(p, sigs)| {
                (
                    PathBuf::from(p),
                    sigs.iter().map(|s| s.to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn empty_corpus_returns_empty() {
        assert!(bm25_rank("authentication", &[], 10).is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        let c = corpus(&[("auth.rs", &["authenticate"])]);
        assert!(bm25_rank("", &c, 10).is_empty());
    }

    #[test]
    fn short_query_filtered_returns_empty() {
        // "to" tem len=2, filtrado por tokenize → query_terms vazio
        let c = corpus(&[("auth.rs", &["authenticate"])]);
        assert!(bm25_rank("to", &c, 10).is_empty());
    }

    #[test]
    fn top_n_zero_returns_all() {
        let c = corpus(&[
            ("a.rs", &["authenticate_user"]),
            ("b.rs", &["database_schema"]),
            ("c.rs", &["session_token"]),
        ]);
        let result = bm25_rank("authenticate", &c, 0);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn top_n_larger_than_corpus_returns_all() {
        let c = corpus(&[("a.rs", &["authenticate"]), ("b.rs", &["session"])]);
        let result = bm25_rank("authenticate", &c, 100);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn matching_doc_ranks_above_non_matching() {
        let c = corpus(&[
            ("models.rs", &["database_schema", "table_setup"]),
            ("auth.rs", &["authenticate_user", "login_session"]),
        ]);
        let result = bm25_rank("authenticate", &c, 0);
        assert_eq!(result[0].0, PathBuf::from("auth.rs"));
    }

    #[test]
    fn result_sorted_descending_by_score() {
        let c = corpus(&[
            ("a.rs", &["authenticate_user", "authenticate_token"]),
            ("b.rs", &["authenticate_user"]),
            ("c.rs", &["unrelated_module"]),
        ]);
        let result = bm25_rank("authenticate", &c, 0);
        for i in 1..result.len() {
            assert!(result[i - 1].1 >= result[i].1);
        }
    }

    #[test]
    fn identical_docs_have_equal_scores() {
        let c = corpus(&[
            ("a.rs", &["authenticate_user"]),
            ("b.rs", &["authenticate_user"]),
        ]);
        let result = bm25_rank("authenticate", &c, 0);
        assert_eq!(result.len(), 2);
        assert!((result[0].1 - result[1].1).abs() < 1e-9);
    }

    #[test]
    fn top_n_truncates_result() {
        let c = corpus(&[
            ("a.rs", &["authenticate"]),
            ("b.rs", &["authenticate"]),
            ("c.rs", &["authenticate"]),
        ]);
        let result = bm25_rank("authenticate", &c, 2);
        assert_eq!(result.len(), 2);
    }
}
