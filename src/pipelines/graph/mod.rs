// Pipeline `graph` — grafo de símbolos resolvido para navegação semântica do código.
//
// Responde perguntas que `grep` não responde sem N tentativas:
//   - Quem chama esta função?     (callers)
//   - O que esta função chama?    (callees)
//   - Que cadeia chega aqui?      (trace)
//   - O que quebra se eu mudar?   (impact)
//   - Onde está definido X?       (node)
//
// Fluxo: Extractor → Resolver → SQLite store → Ranking → Budget → Output.
//
// Diferencial vs CodeGraph: resultados ranqueados por relevância à query atual
// (BM25 + sub-graph PageRank), respeitam token budget, e callers similares
// são deduplicados.

pub mod extractor;
pub mod frameworks;
pub mod query;
pub mod store;
pub mod types;

pub use query::{callees, callers, impact, node, trace, QueryResult};
pub use types::{CallSite, GraphNode, QueryOptions, Symbol, SymbolKind};

use anyhow::Result;
use rayon::prelude::*;

/// Estatísticas de indexação.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct IndexStats {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub symbols: usize,
    pub calls: usize,
    pub errors: usize,
}

/// Indexa um ou mais diretórios populando o store SQLite.
///
/// Reusa o `scanner` do pipeline map para descobrir arquivos respeitando `.gitignore`.
/// Cada arquivo é re-indexado idempotente (`clear_file` + insert).
/// Extensões cobertas pelo pipeline graph (superset do map).
pub const GRAPH_EXTS: &[&str] = &[
    ".rs", ".go", ".java", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".py", ".rb", ".rake",
    ".groovy", ".gradle",
];

pub fn index(dirs: &[String], max_depth: usize) -> Result<IndexStats> {
    let files = crate::pipelines::map::scanner::scan_files_with_exts(dirs, max_depth, GRAPH_EXTS);
    let mut stats = IndexStats {
        files_scanned: files.len(),
        ..Default::default()
    };

    // Extrai em paralelo (CPU-bound).
    let extracted: Vec<_> = files
        .par_iter()
        .filter_map(|p| match extractor::extract(p) {
            Ok(Some(e)) => Some((p.clone(), Ok(e))),
            Ok(None) => None,
            Err(err) => Some((p.clone(), Err(err))),
        })
        .collect();

    // Insere serialmente para não brigar pelo lock do SQLite.
    let conn = store::open_default()?;
    for (path, result) in extracted {
        let file = path.to_string_lossy().to_string();
        match result {
            Ok(e) => {
                store::clear_file(&conn, &file)?;
                for sym in &e.symbols {
                    store::insert_symbol(&conn, sym)?;
                    stats.symbols += 1;
                }
                for call in &e.calls {
                    store::insert_call(&conn, call)?;
                    stats.calls += 1;
                }
                for (module, alias) in &e.imports {
                    store::insert_import(&conn, &file, module, alias.as_deref())?;
                }
                stats.files_indexed += 1;

                // Framework-aware routing: detecta endpoints HTTP no arquivo e
                // injeta símbolos sintéticos `route::METHOD /path` + call site
                // ligando à action correspondente.
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (mapping, language) in frameworks::detect_routes(&path, &content) {
                        let (sym, call) = mapping.into_graph_entries(language);
                        store::insert_symbol(&conn, &sym)?;
                        store::insert_call(&conn, &call)?;
                        stats.symbols += 1;
                        stats.calls += 1;
                    }
                }
            }
            Err(_) => {
                stats.errors += 1;
            }
        }
    }

    Ok(stats)
}
