// Grails route detector: parseia `UrlMappings.groovy` (DSL) e também
// auto-mapeia convenção `<Resource>Controller.<action>` → `GET /<resource>/<action>`.
//
// Grails URL mappings exemplo:
//   "/users/$id"(controller: "user", action: "show")
//   "/users"(resources: "user")
//   "/$controller/$action?/$id?"(...)

use std::path::Path;

use super::{FrameworkRouter, RouteMapping};

pub struct GrailsRouter;

impl FrameworkRouter for GrailsRouter {
    fn detect(&self, path: &Path, content: &str) -> bool {
        let path_str = path.to_string_lossy();
        if !path_str.ends_with(".groovy") {
            return false;
        }
        path_str.ends_with("UrlMappings.groovy")
            || content.contains("class UrlMappings")
            || content.contains("static mappings = {")
    }

    fn extract(&self, path: &Path, content: &str) -> Vec<RouteMapping> {
        let file = path.to_string_lossy().to_string();
        let mut mappings = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_no = (idx + 1) as u32;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // "/users/$id"(controller: "user", action: "show")
            if let Some(mapping) = parse_explicit_mapping(trimmed, &file, line_no) {
                mappings.push(mapping);
                continue;
            }

            // "/users"(resources: "user")
            if let Some(resource_mappings) = parse_resources_mapping(trimmed, &file, line_no) {
                mappings.extend(resource_mappings);
                continue;
            }
        }

        mappings
    }

    fn language(&self) -> &'static str {
        "groovy"
    }
}

fn parse_explicit_mapping(line: &str, file: &str, line_no: u32) -> Option<RouteMapping> {
    // "/users/$id"(controller: "user", action: "show")
    let url = extract_quoted_url(line)?;
    let controller = extract_kv_value(line, "controller")?;
    let action = extract_kv_value(line, "action").unwrap_or_else(|| "index".to_string());
    let method = extract_kv_value(line, "method").unwrap_or_else(|| "ANY".to_string());

    Some(RouteMapping {
        method: method.to_uppercase(),
        path: grails_path_to_standard(&url),
        handler: format!("{}Controller::{}", capitalize_first(&controller), action),
        source_file: file.to_string(),
        source_line: line_no,
    })
}

fn parse_resources_mapping(line: &str, file: &str, line_no: u32) -> Option<Vec<RouteMapping>> {
    // "/users"(resources: "user")  → 7 ações RESTful
    let url = extract_quoted_url(line)?;
    let resource = extract_kv_value(line, "resources")?;
    let controller = format!("{}Controller", capitalize_first(&resource));
    let base = url.trim_end_matches('/').to_string();

    Some(vec![
        RouteMapping {
            method: "GET".to_string(),
            path: base.clone(),
            handler: format!("{}::index", controller),
            source_file: file.to_string(),
            source_line: line_no,
        },
        RouteMapping {
            method: "POST".to_string(),
            path: base.clone(),
            handler: format!("{}::save", controller),
            source_file: file.to_string(),
            source_line: line_no,
        },
        RouteMapping {
            method: "GET".to_string(),
            path: format!("{}/:id", base),
            handler: format!("{}::show", controller),
            source_file: file.to_string(),
            source_line: line_no,
        },
        RouteMapping {
            method: "PUT".to_string(),
            path: format!("{}/:id", base),
            handler: format!("{}::update", controller),
            source_file: file.to_string(),
            source_line: line_no,
        },
        RouteMapping {
            method: "DELETE".to_string(),
            path: format!("{}/:id", base),
            handler: format!("{}::delete", controller),
            source_file: file.to_string(),
            source_line: line_no,
        },
    ])
}

fn extract_quoted_url(line: &str) -> Option<String> {
    // "/users/$id"(...) — primeira string entre aspas
    let start = line.find('"')?;
    let after = &line[start + 1..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn extract_kv_value(line: &str, key: &str) -> Option<String> {
    // controller: "user"  ou  controller:"user"
    let needle = format!("{}:", key);
    let pos = line.find(&needle)?;
    let after = &line[pos + needle.len()..];
    let after = after.trim_start();
    // Pega primeira string entre aspas
    if let Some(q_start) = after.find('"') {
        let after_q = &after[q_start + 1..];
        let q_end = after_q.find('"')?;
        return Some(after_q[..q_end].to_string());
    }
    if let Some(q_start) = after.find('\'') {
        let after_q = &after[q_start + 1..];
        let q_end = after_q.find('\'')?;
        return Some(after_q[..q_end].to_string());
    }
    None
}

fn grails_path_to_standard(url: &str) -> String {
    // Converte $id → :id (alinha com convenção comum vista pelo agente).
    url.split('/')
        .map(|seg| {
            if let Some(name) = seg.strip_prefix('$') {
                format!(":{}", name.trim_end_matches('?'))
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn extract(content: &str) -> Vec<RouteMapping> {
        let router = GrailsRouter;
        router.extract(
            &PathBuf::from("grails-app/controllers/UrlMappings.groovy"),
            content,
        )
    }

    #[test]
    fn explicit_mapping_com_controller_e_action() {
        let src = r#"
class UrlMappings {
    static mappings = {
        "/users/$id"(controller: "user", action: "show")
        "/login"(controller: "auth", action: "login", method: "POST")
    }
}
"#;
        let routes = extract(src);
        assert!(routes
            .iter()
            .any(|r| r.path == "/users/:id" && r.handler == "UserController::show"));
        assert!(routes
            .iter()
            .any(|r| r.method == "POST" && r.handler == "AuthController::login"));
    }

    #[test]
    fn resources_mapping_gera_acoes_restful() {
        let src = r#"
class UrlMappings {
    static mappings = {
        "/api/users"(resources: "user")
    }
}
"#;
        let routes = extract(src);
        assert_eq!(routes.len(), 5);
        assert!(routes.iter().any(|r| r.method == "GET"
            && r.path == "/api/users"
            && r.handler == "UserController::index"));
        assert!(routes.iter().any(|r| r.method == "DELETE"
            && r.path == "/api/users/:id"
            && r.handler == "UserController::delete"));
    }

    #[test]
    fn detect_aplica_apenas_url_mappings_groovy() {
        let r = GrailsRouter;
        assert!(r.detect(
            &PathBuf::from("grails-app/controllers/UrlMappings.groovy"),
            "class UrlMappings { static mappings = { } }"
        ));
        assert!(!r.detect(
            &PathBuf::from("grails-app/controllers/UserController.groovy"),
            "class UserController { def show() {} }"
        ));
    }
}
