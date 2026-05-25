use crate::pipelines::map::output::format_repo_map;
use std::path::PathBuf;

/// Binary search para maximizar arquivos dentro do budget (1 token ≈ 4 chars).
pub fn fit_to_budget(
    ranked: &[(PathBuf, f64)],
    base_dirs: &[String],
    max_tokens: usize,
) -> Vec<(PathBuf, f64)> {
    if ranked.is_empty() {
        return vec![];
    }
    let mut lo = 1usize;
    let mut hi = ranked.len();
    let mut best_n = 1;

    while lo <= hi {
        let mid = (lo + hi) / 2;
        let candidate = format_repo_map(&ranked[..mid], base_dirs);
        let approx_tokens = candidate.len() / 4;
        if approx_tokens <= max_tokens {
            best_n = mid;
            lo = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        }
    }
    ranked[..best_n].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join(name)
    }

    #[test]
    fn empty_ranked_returns_empty() {
        assert!(fit_to_budget(&[], &[], 1000).is_empty());
    }

    #[test]
    fn large_budget_returns_all() {
        let ranked = vec![
            (fixture("sample.rb"), 1.0),
            (fixture("sample.py"), 0.8),
            (fixture("sample.ts"), 0.6),
        ];
        let result = fit_to_budget(&ranked, &[], 100_000);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn minimal_budget_returns_at_least_one() {
        // budget=1 token é insuficiente para qualquer arquivo, mas best_n começa em 1
        let ranked = vec![(fixture("sample.rb"), 1.0), (fixture("sample.py"), 0.8)];
        let result = fit_to_budget(&ranked, &[], 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, fixture("sample.rb"));
    }

    #[test]
    fn preserves_ranking_order() {
        let ranked = vec![
            (fixture("sample.rb"), 1.0),
            (fixture("sample.py"), 0.8),
            (fixture("sample.ts"), 0.6),
        ];
        let result = fit_to_budget(&ranked, &[], 100_000);
        assert_eq!(result[0].0, fixture("sample.rb"));
        assert_eq!(result[1].0, fixture("sample.py"));
        assert_eq!(result[2].0, fixture("sample.ts"));
    }
}
