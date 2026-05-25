mod groovy;
mod java;
mod javascript;
mod python;
mod ruby;
mod rust;
mod typescript;

use std::path::Path;

pub fn ext_to_lang(path: &Path) -> Option<&'static str> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match ext {
        "rb" | "rake" => Some("ruby"),
        "groovy" | "gradle" => Some("groovy"),
        "py" => Some("python"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "rs" => Some("rust"),
        "java" => Some("java"),
        _ if matches!(name, "Rakefile" | "Gemfile" | "Capfile") => Some("ruby"),
        _ => None,
    }
}

pub fn extract_signatures(path: &Path) -> Vec<String> {
    let Some(lang) = ext_to_lang(path) else {
        return vec![];
    };
    let Ok(src) = std::fs::read(path) else {
        return vec![];
    };
    match lang {
        "ruby" => ruby::extract(&src),
        "python" => python::extract(&src),
        "typescript" | "tsx" => typescript::extract(&src, lang == "tsx"),
        "javascript" => javascript::extract(&src),
        "groovy" => groovy::extract(&src),
        "rust" => rust::extract(&src),
        "java" => java::extract(&src),
        _ => vec![],
    }
}

pub fn extract_refs(path: &Path) -> Vec<String> {
    let Some(lang) = ext_to_lang(path) else {
        return vec![];
    };
    let Ok(src) = std::fs::read(path) else {
        return vec![];
    };
    match lang {
        "ruby" => ruby::extract_refs(&src),
        "python" => python::extract_refs(&src),
        "typescript" | "tsx" => typescript::extract_refs(&src, lang == "tsx"),
        "javascript" => javascript::extract_refs(&src),
        "groovy" => groovy::extract_refs(&src),
        "rust" => rust::extract_refs(&src),
        "java" => java::extract_refs(&src),
        _ => vec![],
    }
}
