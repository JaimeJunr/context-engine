use std::collections::HashMap;
use std::path::PathBuf;

/// Power-iteration Personalized PageRank (alpha=0.85).
pub fn pagerank_pure(
    graph: &HashMap<PathBuf, Vec<PathBuf>>,
    predecessors: &HashMap<PathBuf, Vec<PathBuf>>,
    personalization: &HashMap<PathBuf, f64>,
    alpha: f64,
    max_iter: usize,
    tol: f64,
) -> HashMap<PathBuf, f64> {
    let nodes: Vec<PathBuf> = graph.keys().cloned().collect();
    let n = nodes.len();
    if n == 0 {
        return HashMap::new();
    }

    let out_deg: HashMap<PathBuf, usize> = nodes
        .iter()
        .map(|node| (node.clone(), graph[node].len()))
        .collect();

    let init_val = 1.0 / n as f64;
    let mut x: HashMap<PathBuf, f64> = nodes.iter().map(|node| (node.clone(), init_val)).collect();

    for _ in 0..max_iter {
        let xlast = x.clone();
        let dangling: f64 = nodes
            .iter()
            .filter(|node| out_deg[*node] == 0)
            .map(|node| xlast[node])
            .sum();

        let mut xnew: HashMap<PathBuf, f64> = nodes.iter().map(|n| (n.clone(), 0.0)).collect();
        for node in &nodes {
            if let Some(preds) = predecessors.get(node) {
                for pred in preds {
                    let od = out_deg.get(pred).copied().unwrap_or(1).max(1) as f64;
                    if let Some(prev) = xlast.get(pred) {
                        *xnew.get_mut(node).unwrap() += alpha * prev / od;
                    }
                }
            }
            let p_val = personalization.get(node).copied().unwrap_or(0.0);
            *xnew.get_mut(node).unwrap() += alpha * dangling * p_val + (1.0 - alpha) * p_val;
        }

        let err: f64 = nodes.iter().map(|node| (xnew[node] - xlast[node]).abs()).sum();
        x = xnew;
        if err < n as f64 * tol {
            break;
        }
    }

    x
}

pub fn build_graph(
    corpus: &[(PathBuf, Vec<String>)],
    symbol_to_files: &HashMap<String, Vec<PathBuf>>,
) -> (
    HashMap<PathBuf, Vec<PathBuf>>,
    HashMap<PathBuf, Vec<PathBuf>>,
) {
    let mut graph: HashMap<PathBuf, Vec<PathBuf>> = corpus
        .iter()
        .map(|(p, _)| (p.clone(), vec![]))
        .collect();
    let mut predecessors: HashMap<PathBuf, Vec<PathBuf>> = corpus
        .iter()
        .map(|(p, _)| (p.clone(), vec![]))
        .collect();

    for (src_path, _) in corpus {
        let refs = crate::extractors::extract_refs(src_path);
        for sym in refs {
            if let Some(targets) = symbol_to_files.get(&sym) {
                for tgt in targets {
                    if tgt != src_path {
                        graph.entry(src_path.clone()).or_default().push(tgt.clone());
                        predecessors.entry(tgt.clone()).or_default().push(src_path.clone());
                    }
                }
            }
        }
    }
    (graph, predecessors)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALPHA: f64 = 0.85;
    const MAX_ITER: usize = 200;
    const TOL: f64 = 1e-6;

    fn p(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    #[test]
    fn empty_graph_returns_empty() {
        let result = pagerank_pure(
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            ALPHA,
            MAX_ITER,
            TOL,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn single_dangling_node_converges_to_one() {
        let node = p("a.rs");
        let mut graph = HashMap::new();
        graph.insert(node.clone(), vec![]);
        let mut preds = HashMap::new();
        preds.insert(node.clone(), vec![]);
        let mut pers = HashMap::new();
        pers.insert(node.clone(), 1.0);

        let result = pagerank_pure(&graph, &preds, &pers, ALPHA, MAX_ITER, TOL);
        assert!((result[&node] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn two_nodes_cycle_scores_are_equal() {
        let a = p("a.rs");
        let b = p("b.rs");
        let mut graph = HashMap::new();
        graph.insert(a.clone(), vec![b.clone()]);
        graph.insert(b.clone(), vec![a.clone()]);
        let mut preds = HashMap::new();
        preds.insert(a.clone(), vec![b.clone()]);
        preds.insert(b.clone(), vec![a.clone()]);
        let mut pers = HashMap::new();
        pers.insert(a.clone(), 0.5);
        pers.insert(b.clone(), 0.5);

        let result = pagerank_pure(&graph, &preds, &pers, ALPHA, MAX_ITER, TOL);
        assert!((result[&a] - result[&b]).abs() < 1e-4);
    }

    #[test]
    fn scores_sum_to_one_when_personalization_sums_to_one() {
        let a = p("a.rs");
        let b = p("b.rs");
        let mut graph = HashMap::new();
        graph.insert(a.clone(), vec![b.clone()]);
        graph.insert(b.clone(), vec![a.clone()]);
        let mut preds = HashMap::new();
        preds.insert(a.clone(), vec![b.clone()]);
        preds.insert(b.clone(), vec![a.clone()]);
        let mut pers = HashMap::new();
        pers.insert(a.clone(), 0.5);
        pers.insert(b.clone(), 0.5);

        let result = pagerank_pure(&graph, &preds, &pers, ALPHA, MAX_ITER, TOL);
        let sum: f64 = result.values().sum();
        assert!((sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn personalization_biases_score() {
        // nó 'a' tem personalization maior → deve ter score maior
        let a = p("a.rs");
        let b = p("b.rs");
        let mut graph = HashMap::new();
        graph.insert(a.clone(), vec![b.clone()]);
        graph.insert(b.clone(), vec![a.clone()]);
        let mut preds = HashMap::new();
        preds.insert(a.clone(), vec![b.clone()]);
        preds.insert(b.clone(), vec![a.clone()]);
        let mut pers = HashMap::new();
        pers.insert(a.clone(), 0.8);
        pers.insert(b.clone(), 0.2);

        let result = pagerank_pure(&graph, &preds, &pers, ALPHA, MAX_ITER, TOL);
        assert!(result[&a] > result[&b]);
    }

    #[test]
    fn dangling_node_does_not_panic() {
        // nó 'a' aponta para 'b', mas 'b' não aponta para ninguém (dangling)
        let a = p("a.rs");
        let b = p("b.rs");
        let mut graph = HashMap::new();
        graph.insert(a.clone(), vec![b.clone()]);
        graph.insert(b.clone(), vec![]);
        let mut preds = HashMap::new();
        preds.insert(a.clone(), vec![]);
        preds.insert(b.clone(), vec![a.clone()]);
        let mut pers = HashMap::new();
        pers.insert(a.clone(), 0.5);
        pers.insert(b.clone(), 0.5);

        let result = pagerank_pure(&graph, &preds, &pers, ALPHA, MAX_ITER, TOL);
        assert!(result.contains_key(&a));
        assert!(result.contains_key(&b));
    }
}
