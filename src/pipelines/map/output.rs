use crate::pipelines::map::extractors::extract_signatures;
use std::path::{Path, PathBuf};

fn relative(path: &Path, base_dirs: &[String]) -> String {
    for base in base_dirs {
        if let Ok(rel) = path.strip_prefix(base) {
            return rel.to_string_lossy().into_owned();
        }
    }
    path.to_string_lossy().into_owned()
}

pub fn format_repo_map(ranked: &[(PathBuf, f64)], base_dirs: &[String]) -> String {
    if ranked.is_empty() {
        return String::new();
    }

    let mut lines = vec!["<repo_map>".to_string()];
    for (path, _score) in ranked {
        let display = relative(path, base_dirs);
        let sigs = extract_signatures(path);
        lines.push(format!("\n{}:", display));
        if sigs.is_empty() {
            lines.push("  (sem assinaturas extraídas)".to_string());
        } else {
            lines.push("⋮".to_string());
            for s in &sigs {
                lines.push(format!("│{}", s));
            }
            lines.push("⋮".to_string());
        }
    }
    lines.push("\n</repo_map>".to_string());
    lines.join("\n")
}

pub fn format_json(ranked: &[(PathBuf, f64)]) -> String {
    let items: Vec<serde_json::Value> = ranked
        .iter()
        .map(|(path, score)| {
            let sigs = extract_signatures(path);
            serde_json::json!({
                "path": path.to_string_lossy(),
                "score": (score * 10000.0).round() / 10000.0,
                "signatures": sigs,
            })
        })
        .collect();
    serde_json::to_string_pretty(&items).unwrap_or_default()
}
