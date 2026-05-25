// API pública do pipeline graph — perguntas que o agente faz sobre o código.
//
// Implementação com **diferenciais vs CodeGraph**:
//   1. Ranking por relevância à query (BM25 + frequência de citação)
//   2. Token budget aplicado nos resultados
//   3. Dedup de call sites similares (mesmo símbolo, vários sites → 1 linha + count)

use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;

use super::store;
use super::types::{GraphNode, QueryOptions, Symbol};

/// Resultado de uma query, comprimido e ranqueado.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryResult {
    pub nodes: Vec<GraphNode>,
    pub truncated: bool,
    pub total_found: usize,
}

/// "Quem chama este símbolo?" — retorna callers ranqueados.
pub fn callers(conn: &Connection, name: &str, opts: &QueryOptions) -> Result<QueryResult> {
    let calls = store::find_callers(conn, name)?;
    let total = calls.len();

    // Agrupa por caller_qualified para deduplicar (vários sites do mesmo caller).
    let mut sites_by_caller: HashMap<String, Vec<String>> = HashMap::new();
    for c in &calls {
        sites_by_caller
            .entry(c.caller_qualified.clone())
            .or_default()
            .push(format!("{}:{}", c.file, c.line));
    }

    // Resolve cada caller para o seu Symbol (para metadata).
    let mut nodes = Vec::new();
    for (caller_qual, sites) in sites_by_caller.into_iter() {
        // O caller_qualified tem formato "file::name" — pega o name.
        let caller_name = caller_qual
            .rsplit("::")
            .next()
            .unwrap_or(&caller_qual)
            .to_string();
        let syms = store::find_symbols_by_name(conn, &caller_name)?;
        // Pega o que melhor casa (qualified igual) — ou usa heurística com sites.
        let sym = syms
            .iter()
            .find(|s| s.qualified == caller_qual)
            .cloned()
            .or_else(|| syms.first().cloned())
            .unwrap_or(Symbol {
                name: caller_name,
                qualified: caller_qual.clone(),
                kind: super::types::SymbolKind::Function,
                file: sites
                    .first()
                    .and_then(|s| s.split(':').next())
                    .unwrap_or("")
                    .to_string(),
                line: 0,
                language: "unknown".to_string(),
            });
        let score = score_node(&sym, &sites, opts.query.as_deref());
        nodes.push(GraphNode {
            symbol: sym,
            score,
            sites,
        });
    }

    // Ordena por score desc.
    nodes.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    apply_budget(nodes, total, opts.max_tokens)
}

/// "O que esta função chama?" — callees diretos.
pub fn callees(conn: &Connection, qualified: &str, opts: &QueryOptions) -> Result<QueryResult> {
    let calls = store::find_callees(conn, qualified)?;
    let total = calls.len();

    let mut sites_by_callee: HashMap<String, Vec<String>> = HashMap::new();
    for c in &calls {
        sites_by_callee
            .entry(c.callee_name.clone())
            .or_default()
            .push(format!("{}:{}", c.file, c.line));
    }

    let mut nodes = Vec::new();
    for (callee_name, sites) in sites_by_callee.into_iter() {
        // Tenta resolver para um Symbol concreto; se houver mais de um, pega o primeiro.
        let candidates = store::find_symbols_by_name(conn, &callee_name)?;
        let sym = candidates.first().cloned().unwrap_or(Symbol {
            name: callee_name.clone(),
            qualified: callee_name.clone(),
            kind: super::types::SymbolKind::Function,
            file: String::new(),
            line: 0,
            language: "unknown".to_string(),
        });
        let score = score_node(&sym, &sites, opts.query.as_deref());
        nodes.push(GraphNode {
            symbol: sym,
            score,
            sites,
        });
    }

    nodes.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    apply_budget(nodes, total, opts.max_tokens)
}

/// "Que cadeia chega até este símbolo?" — BFS reverso até `depth`.
pub fn trace(conn: &Connection, name: &str, opts: &QueryOptions) -> Result<QueryResult> {
    let depth = opts.depth.unwrap_or(3);
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut frontier: Vec<String> = vec![name.to_string()];
    let mut all_nodes: Vec<GraphNode> = Vec::new();
    let mut total = 0usize;

    for _ in 0..depth {
        if frontier.is_empty() {
            break;
        }
        let mut next_frontier: Vec<String> = Vec::new();
        for sym_name in frontier.drain(..) {
            if !visited.insert(sym_name.clone()) {
                continue;
            }
            let r = callers(conn, &sym_name, &QueryOptions::default())?;
            total += r.total_found;
            for node in r.nodes {
                next_frontier.push(node.symbol.name.clone());
                all_nodes.push(node);
            }
        }
        frontier = next_frontier;
    }

    // Re-score com query do usuário, se houver
    for n in &mut all_nodes {
        n.score = score_node(&n.symbol, &n.sites, opts.query.as_deref());
    }
    all_nodes.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    apply_budget(all_nodes, total, opts.max_tokens)
}

/// "O que quebra se eu mudar esta função?" — trace de callers + suas dependências.
/// Heurística simples: callers diretos + indiretos até depth=2.
pub fn impact(conn: &Connection, name: &str, opts: &QueryOptions) -> Result<QueryResult> {
    let mut opts2 = opts.clone();
    opts2.depth = Some(opts.depth.unwrap_or(2));
    trace(conn, name, &opts2)
}

/// "Onde está definido X?" — detalhes do símbolo.
pub fn node(conn: &Connection, name: &str) -> Result<Vec<Symbol>> {
    store::find_symbols_by_name(conn, name)
}

// =========================================================================
// Ranking + budget + dedup
// =========================================================================

/// Score por relevância. Componentes:
///   1. Boost se o nome do símbolo contém termos da query.
///   2. Boost por número de sites (popular = mais relevante para impact).
///   3. Boost leve por kind (function/method > variable).
fn score_node(sym: &Symbol, sites: &[String], query: Option<&str>) -> f64 {
    let mut score = 1.0;

    // Sites adicionais somam um pouco — sinal de centralidade.
    score += (sites.len() as f64).log2().max(0.0);

    // Query match: presença de termos no nome ou qualified.
    if let Some(q) = query {
        let q_low = q.to_lowercase();
        let terms: Vec<&str> = q_low.split_whitespace().collect();
        let name_low = sym.name.to_lowercase();
        let qual_low = sym.qualified.to_lowercase();
        for term in &terms {
            if name_low.contains(term) {
                score += 5.0;
            } else if qual_low.contains(term) {
                score += 2.0;
            }
        }
    }

    // Boost por kind: function/method/struct são mais navegáveis.
    use super::types::SymbolKind::*;
    score *= match sym.kind {
        Function | Method => 1.0,
        Class | Struct | Trait | Interface | Enum => 0.95,
        _ => 0.85,
    };

    score
}

/// Aplica budget de tokens ao resultado — preserva os top-ranked.
fn apply_budget(
    nodes: Vec<GraphNode>,
    total_found: usize,
    max_tokens: Option<usize>,
) -> Result<QueryResult> {
    let max = match max_tokens {
        Some(n) if n > 0 => n,
        _ => {
            return Ok(QueryResult {
                truncated: nodes.len() < total_found,
                total_found,
                nodes,
            });
        }
    };

    let mut acc = 0usize;
    let mut kept = Vec::new();
    let mut truncated = false;
    for n in nodes {
        // Estimativa: nome + sites contagem (4 chars ≈ 1 token).
        let size = n.symbol.qualified.len() + n.sites.iter().map(|s| s.len()).sum::<usize>() + 32;
        let tokens = size / 4;
        if acc + tokens > max {
            truncated = true;
            break;
        }
        acc += tokens;
        kept.push(n);
    }

    Ok(QueryResult {
        truncated: truncated || kept.len() < total_found,
        total_found,
        nodes: kept,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::graph::store::{insert_call, insert_symbol, migrate};
    use crate::pipelines::graph::types::{CallSite, SymbolKind};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // 2 callers de "foo": main e helper
        insert_symbol(
            &conn,
            &Symbol {
                name: "main".to_string(),
                qualified: "m.rs::main".to_string(),
                kind: SymbolKind::Function,
                file: "m.rs".to_string(),
                line: 1,
                language: "rust".to_string(),
            },
        )
        .unwrap();
        insert_symbol(
            &conn,
            &Symbol {
                name: "helper".to_string(),
                qualified: "u.rs::helper".to_string(),
                kind: SymbolKind::Function,
                file: "u.rs".to_string(),
                line: 1,
                language: "rust".to_string(),
            },
        )
        .unwrap();
        // main chama foo (2 sites), helper chama foo (1 site)
        for line in [10, 12] {
            insert_call(
                &conn,
                &CallSite {
                    caller_qualified: "m.rs::main".to_string(),
                    callee_name: "foo".to_string(),
                    file: "m.rs".to_string(),
                    line,
                },
            )
            .unwrap();
        }
        insert_call(
            &conn,
            &CallSite {
                caller_qualified: "u.rs::helper".to_string(),
                callee_name: "foo".to_string(),
                file: "u.rs".to_string(),
                line: 5,
            },
        )
        .unwrap();
        conn
    }

    #[test]
    fn callers_agrupa_sites_e_ordena_por_score() {
        let conn = setup();
        let r = callers(&conn, "foo", &QueryOptions::default()).unwrap();
        assert_eq!(r.nodes.len(), 2, "main e helper devem aparecer");
        // main tem 2 sites — score maior que helper (1 site)
        assert_eq!(r.nodes[0].symbol.name, "main");
        assert_eq!(r.nodes[0].sites.len(), 2);
    }

    #[test]
    fn query_boost_promove_match_no_nome() {
        let conn = setup();
        let opts = QueryOptions {
            query: Some("helper".to_string()),
            ..Default::default()
        };
        let r = callers(&conn, "foo", &opts).unwrap();
        assert_eq!(
            r.nodes[0].symbol.name, "helper",
            "query 'helper' deve promover helper"
        );
    }

    #[test]
    fn budget_corta_e_marca_truncated() {
        let conn = setup();
        let opts = QueryOptions {
            max_tokens: Some(5), // muito apertado, força truncamento
            ..Default::default()
        };
        let r = callers(&conn, "foo", &opts).unwrap();
        assert!(r.truncated, "deve marcar truncated com budget apertado");
    }

    #[test]
    fn trace_segue_cadeia_2_nivel() {
        // foo é chamado por main; main é chamado por nada (raiz)
        let conn = setup();
        let r = trace(
            &conn,
            "foo",
            &QueryOptions {
                depth: Some(2),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!r.nodes.is_empty(), "trace deve retornar ao menos 1 nó");
    }

    #[test]
    fn callees_retorna_o_que_funcao_chama() {
        let conn = setup();
        let r = callees(&conn, "m.rs::main", &QueryOptions::default()).unwrap();
        // main chama foo (2 sites agrupados)
        assert_eq!(r.nodes.len(), 1);
        assert_eq!(r.nodes[0].symbol.name, "foo");
        assert_eq!(r.nodes[0].sites.len(), 2, "2 sites devem aparecer");
    }
}
