mod groovy;
mod python;
mod ruby;
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
        "groovy" => groovy::extract(&src),
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
        "groovy" => groovy::extract_refs(&src),
        _ => vec![],
    }
}
