use crate::shared::tokenizer::tokenize;
use std::collections::HashMap;
use std::path::PathBuf;

// BM25 genérico: K é o tipo da chave (PathBuf para map, i64 para search)
pub fn bm25_rank_generic<K: Clone>(
    query: &str,
    corpus: &[(K, Vec<String>)],
    top_n: usize,
    doc_text_fn: impl Fn(&K, &[String]) -> String,
) -> Vec<(K, f64)> {
    let k1: f64 = 1.5;
    let b: f64 = 0.75;

    let docs: Vec<Vec<String>> = corpus
        .iter()
        .map(|(key, tokens)| tokenize(&doc_text_fn(key, tokens)))
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

    let mut scores: Vec<(K, f64)> = corpus
        .iter()
        .zip(docs.iter())
        .map(|((key, _), doc)| {
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
            (key.clone(), score)
        })
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if top_n > 0 {
        scores.truncate(top_n);
    }
    scores
}

/// Sufixos de arquivos auxiliares (testes, stories) que devem receber penalização no ranking.
/// Fator de penalização: 0.3 (reduz o score a 30% do valor original).
const PENALTY_FACTOR: f64 = 0.3;
const PENALIZED_SUFFIXES: &[&str] = &[
    ".stories.tsx",
    ".stories.ts",
    ".stories.js",
    ".stories.jsx",
    ".spec.ts",
    ".spec.tsx",
    ".test.ts",
    ".test.tsx",
];

fn penalty_factor(path: &std::path::Path) -> f64 {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if PENALIZED_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
    {
        PENALTY_FACTOR
    } else {
        1.0
    }
}

// Wrapper para compatibilidade com o pipeline map (PathBuf + sigs)
pub fn bm25_rank(
    query: &str,
    corpus: &[(PathBuf, Vec<String>)],
    top_n: usize,
) -> Vec<(PathBuf, f64)> {
    // Passa top_n=0 para obter todos os scores antes de penalizar
    let mut scores = bm25_rank_generic(query, corpus, 0, |path, sigs| {
        format!("{} {}", path.display(), sigs.join(" "))
    });

    // Aplica penalização para arquivos auxiliares (stories, spec, test)
    for (path, score) in &mut scores {
        *score *= penalty_factor(path);
    }

    // Re-ordena após penalização e aplica limite
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

    #[test]
    fn stories_tsx_recebe_score_menor_que_tsx_equivalente() {
        // Arquivos com mesmo conteúdo — stories deve ter score menor por penalização
        let sigs = &["UserCard", "handleSubmit", "UserCardProps"];
        let c = corpus(&[
            ("src/components/UserCard.tsx", sigs),
            ("src/components/UserCard.stories.tsx", sigs),
        ]);
        let result = bm25_rank("UserCard handleSubmit", &c, 0);
        let score_tsx = result
            .iter()
            .find(|(p, _)| p == &PathBuf::from("src/components/UserCard.tsx"))
            .map(|(_, s)| *s)
            .unwrap();
        let score_stories = result
            .iter()
            .find(|(p, _)| p == &PathBuf::from("src/components/UserCard.stories.tsx"))
            .map(|(_, s)| *s)
            .unwrap();

        // Confirma que a penalização foi aplicada: score_stories deve ser ~30% de score_tsx
        assert!(
            score_tsx > score_stories,
            "arquivo .tsx ({:.4}) deve ter score maior que .stories.tsx ({:.4})",
            score_tsx,
            score_stories
        );
        // A diferença deve ser substancial (penalização de 70%), não apenas ordem de sort
        assert!(
            score_stories < score_tsx * 0.5,
            "penalização deve reduzir score em ao menos 50% (esperado ~70%): tsx={:.4}, stories={:.4}",
            score_tsx,
            score_stories
        );
    }

    #[test]
    fn spec_ts_recebe_penalizacao() {
        let sigs = &["loginUser", "validateSession"];
        let c = corpus(&[
            ("src/auth/service.ts", sigs),
            ("src/auth/service.spec.ts", sigs),
        ]);
        let result = bm25_rank("loginUser validateSession", &c, 0);
        let score_ts = result
            .iter()
            .find(|(p, _)| p == &PathBuf::from("src/auth/service.ts"))
            .map(|(_, s)| *s)
            .unwrap();
        let score_spec = result
            .iter()
            .find(|(p, _)| p == &PathBuf::from("src/auth/service.spec.ts"))
            .map(|(_, s)| *s)
            .unwrap();

        assert!(
            score_ts > score_spec,
            "arquivo .ts ({:.4}) deve ter score maior que .spec.ts ({:.4})",
            score_ts,
            score_spec
        );
        assert!(
            score_spec < score_ts * 0.5,
            "penalização deve reduzir score em ao menos 50%: ts={:.4}, spec={:.4}",
            score_ts,
            score_spec
        );
    }

    #[test]
    fn test_ts_recebe_penalizacao() {
        let sigs = &["parseDate", "formatCurrency"];
        let c = corpus(&[
            ("src/utils/helpers.ts", sigs),
            ("src/utils/helpers.test.ts", sigs),
        ]);
        let result = bm25_rank("parseDate formatCurrency", &c, 0);
        let score_ts = result
            .iter()
            .find(|(p, _)| p == &PathBuf::from("src/utils/helpers.ts"))
            .map(|(_, s)| *s)
            .unwrap();
        let score_test = result
            .iter()
            .find(|(p, _)| p == &PathBuf::from("src/utils/helpers.test.ts"))
            .map(|(_, s)| *s)
            .unwrap();

        assert!(
            score_ts > score_test,
            "arquivo .ts ({:.4}) deve ter score maior que .test.ts ({:.4})",
            score_ts,
            score_test
        );
        assert!(
            score_test < score_ts * 0.5,
            "penalização deve reduzir score em ao menos 50%: ts={:.4}, test={:.4}",
            score_ts,
            score_test
        );
    }
}
