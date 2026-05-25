// Rails route detector: parseia `config/routes.rb` extraindo o DSL
// (`resources`, `resource`, `get`, `post`, `put`, `patch`, `delete`, `match`,
// `namespace`, `scope`).
//
// O Rails resolve a string `"users#show"` para `UsersController#show`. Como o
// extractor base só conhece o nome simples, geramos `callee_name = "show"`
// no `CallSite` e o resolver liga ao símbolo `UsersController::show` via
// nossa heurística normal.

use std::path::Path;

use super::{FrameworkRouter, RouteMapping};

pub struct RailsRouter;

impl FrameworkRouter for RailsRouter {
    fn detect(&self, path: &Path, content: &str) -> bool {
        // Casos comuns:
        //   config/routes.rb
        //   config/routes/<area>.rb (engine routes)
        let path_str = path.to_string_lossy();
        if !path_str.ends_with(".rb") {
            return false;
        }
        let in_routes_dir = path_str.contains("/config/routes.rb")
            || path_str.contains("/config/routes/")
            || path_str.ends_with("routes.rb");
        if !in_routes_dir {
            return false;
        }
        // Sanity: deve conter `Rails.application.routes.draw` ou DSL típica.
        content.contains("routes.draw")
            || content.contains("resources")
            || content.contains("resource ")
            || content_has_verb_dsl(content)
    }

    fn extract(&self, path: &Path, content: &str) -> Vec<RouteMapping> {
        let file = path.to_string_lossy().to_string();
        let mut mappings = Vec::new();

        // Stack de namespaces ativos (afeta o prefixo de URL).
        let mut ns_stack: Vec<String> = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_no = (idx + 1) as u32;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // namespace :admin do
            if let Some(name) = parse_namespace(trimmed) {
                ns_stack.push(name);
                continue;
            }
            // end (sai do namespace mais interno)
            if trimmed == "end" && !ns_stack.is_empty() {
                ns_stack.pop();
                continue;
            }

            // resources :users  ou  resources :users, only: [:index, :show]
            if let Some(resource_name) = parse_resources(trimmed) {
                let prefix = ns_prefix(&ns_stack);
                let plural = &resource_name;
                let singular = singularize(&resource_name);
                let controller = format!("{}_controller", plural);
                let actions = [
                    (
                        "GET",
                        format!("/{}{}", prefix_path(&prefix), plural),
                        "index",
                    ),
                    (
                        "GET",
                        format!("/{}{}/new", prefix_path(&prefix), plural),
                        "new",
                    ),
                    (
                        "POST",
                        format!("/{}{}", prefix_path(&prefix), plural),
                        "create",
                    ),
                    (
                        "GET",
                        format!("/{}{}/:id", prefix_path(&prefix), plural),
                        "show",
                    ),
                    (
                        "GET",
                        format!("/{}{}/:id/edit", prefix_path(&prefix), plural),
                        "edit",
                    ),
                    (
                        "PATCH",
                        format!("/{}{}/:id", prefix_path(&prefix), plural),
                        "update",
                    ),
                    (
                        "PUT",
                        format!("/{}{}/:id", prefix_path(&prefix), plural),
                        "update",
                    ),
                    (
                        "DELETE",
                        format!("/{}{}/:id", prefix_path(&prefix), plural),
                        "destroy",
                    ),
                ];
                let _ = singular; // singular fica como nota; Rails usa por padrão o plural na URL
                for (method, url, action) in actions {
                    mappings.push(RouteMapping {
                        method: method.to_string(),
                        path: url,
                        handler: format!("{}::{}", controller, action),
                        source_file: file.clone(),
                        source_line: line_no,
                    });
                }
                continue;
            }

            // get 'login', to: 'sessions#new'  ou  get '/users/:id' => 'users#show'
            if let Some((method, url, ctrl, action)) = parse_verb_dsl(trimmed) {
                let prefix = ns_prefix(&ns_stack);
                let full_path = if url.starts_with('/') {
                    format!("/{}{}", prefix_path(&prefix), url.trim_start_matches('/'))
                } else {
                    format!("/{}{}", prefix_path(&prefix), url)
                };
                mappings.push(RouteMapping {
                    method,
                    path: full_path,
                    handler: format!("{}_controller::{}", ctrl, action),
                    source_file: file.clone(),
                    source_line: line_no,
                });
            }
        }

        mappings
    }

    fn language(&self) -> &'static str {
        "ruby"
    }
}

fn ns_prefix(stack: &[String]) -> String {
    if stack.is_empty() {
        String::new()
    } else {
        stack.join("/")
    }
}

fn prefix_path(prefix: &str) -> String {
    if prefix.is_empty() {
        String::new()
    } else {
        format!("{}/", prefix)
    }
}

fn parse_namespace(line: &str) -> Option<String> {
    // namespace :admin do
    let after = line.strip_prefix("namespace")?.trim();
    let name = after.strip_prefix(':')?;
    let name = name.split_whitespace().next()?;
    let name = name.trim_end_matches(',');
    Some(name.to_string())
}

fn parse_resources(line: &str) -> Option<String> {
    // resources :users  OR  resources :users, only: ...
    for keyword in &["resources", "resource"] {
        if let Some(rest) = line.strip_prefix(keyword) {
            let rest = rest.trim();
            if let Some(after_colon) = rest.strip_prefix(':') {
                let name = after_colon.split([' ', ',']).next().unwrap_or("").trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

fn parse_verb_dsl(line: &str) -> Option<(String, String, String, String)> {
    // get 'login', to: 'sessions#new'
    // post '/users' => 'users#create'
    // delete :logout, to: 'sessions#destroy'
    for verb in &["get", "post", "put", "patch", "delete", "match"] {
        if let Some(rest) = line.strip_prefix(verb) {
            // Precisamos garantir que o verb é palavra inteira (espaço/aspas após).
            let next_char = rest.chars().next().unwrap_or(' ');
            if !next_char.is_whitespace() && next_char != '(' {
                continue;
            }
            let rest = rest.trim();
            // Captura URL: primeira string entre aspas ou :symbol.
            let url = extract_first_string_or_symbol(rest)?;
            // Captura handler "controller#action".
            let handler = extract_handler(rest)?;
            let (ctrl, action) = handler.split_once('#')?;
            return Some((
                verb.to_uppercase(),
                url,
                ctrl.to_string(),
                action.to_string(),
            ));
        }
    }
    None
}

fn extract_first_string_or_symbol(s: &str) -> Option<String> {
    // 'login'  OR  "/users/:id"  OR  :logout
    if let Some(start) = s.find('\'') {
        let after = &s[start + 1..];
        let end = after.find('\'')?;
        return Some(after[..end].to_string());
    }
    if let Some(start) = s.find('"') {
        let after = &s[start + 1..];
        let end = after.find('"')?;
        return Some(after[..end].to_string());
    }
    if let Some(start) = s.find(':') {
        let after = &s[start + 1..];
        let end = after
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());
        return Some(after[..end].to_string());
    }
    None
}

fn extract_handler(s: &str) -> Option<String> {
    // 'controller#action' (com aspas)
    let after_to = s
        .find("to:")
        .map(|i| &s[i + 3..])
        .or_else(|| s.find("=>").map(|i| &s[i + 2..]))?;
    let trimmed = after_to.trim();
    if let Some(start) = trimmed.find('\'') {
        let after = &trimmed[start + 1..];
        let end = after.find('\'')?;
        let h = &after[..end];
        if h.contains('#') {
            return Some(h.to_string());
        }
    }
    if let Some(start) = trimmed.find('"') {
        let after = &trimmed[start + 1..];
        let end = after.find('"')?;
        let h = &after[..end];
        if h.contains('#') {
            return Some(h.to_string());
        }
    }
    None
}

fn content_has_verb_dsl(content: &str) -> bool {
    content.lines().any(|l| {
        let t = l.trim();
        t.starts_with("get ")
            || t.starts_with("post ")
            || t.starts_with("put ")
            || t.starts_with("patch ")
            || t.starts_with("delete ")
            || t.starts_with("match ")
    })
}

/// Heurística leve de singularização para casos comuns (users → user, categories → category).
fn singularize(plural: &str) -> String {
    if let Some(stem) = plural.strip_suffix("ies") {
        format!("{}y", stem)
    } else if let Some(stem) = plural.strip_suffix('s') {
        stem.to_string()
    } else {
        plural.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn extract_all(content: &str) -> Vec<RouteMapping> {
        let router = RailsRouter;
        router.extract(&PathBuf::from("config/routes.rb"), content)
    }

    #[test]
    fn detecta_resources_gera_7_actions_restful() {
        let routes = extract_all("Rails.application.routes.draw do\n  resources :users\nend\n");
        // 8 entries (PATCH e PUT para update geram 2)
        assert_eq!(routes.len(), 8, "esperava 8 routes, obteve {:?}", routes);
        assert!(routes
            .iter()
            .any(|r| r.method == "GET" && r.path == "/users"));
        assert!(routes
            .iter()
            .any(|r| r.method == "POST" && r.path == "/users"));
        assert!(routes
            .iter()
            .any(|r| r.method == "DELETE" && r.path == "/users/:id"));
        assert!(routes
            .iter()
            .any(|r| r.handler == "users_controller::index"));
    }

    #[test]
    fn detecta_verb_dsl_com_to() {
        let routes = extract_all("get 'login', to: 'sessions#new'\n");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[0].path, "/login");
        assert_eq!(routes[0].handler, "sessions_controller::new");
    }

    #[test]
    fn detecta_verb_dsl_com_rocket() {
        let routes = extract_all("post '/users' => 'users#create'\n");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].handler, "users_controller::create");
    }

    #[test]
    fn namespace_prefixa_path() {
        let routes = extract_all("namespace :admin do\n  resources :users\nend\n");
        assert!(
            routes.iter().any(|r| r.path == "/admin/users"),
            "esperava /admin/users em {:?}",
            routes
                .iter()
                .map(|r| (&r.method, &r.path))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn detect_aplica_apenas_em_routes_rb() {
        let r = RailsRouter;
        assert!(r.detect(
            &PathBuf::from("config/routes.rb"),
            "Rails.application.routes.draw {}"
        ));
        assert!(!r.detect(
            &PathBuf::from("app/controllers/users_controller.rb"),
            "class UsersController; end"
        ));
        assert!(!r.detect(&PathBuf::from("config/database.yml"), "any content"));
    }
}
