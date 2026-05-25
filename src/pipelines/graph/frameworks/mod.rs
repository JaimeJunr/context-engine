// Framework-aware routing: detecta endpoints HTTP (URL → handler) e injeta
// símbolos sintéticos no grafo para que `ctx graph node "GET /users/:id"`
// retorne o controller/action correspondente.
//
// Cada framework é um `FrameworkRouter` que olha um arquivo (path + conteúdo)
// e devolve uma lista de `RouteMapping`s. O `index` passa os mappings ao store
// como `Symbol`s especiais (`SymbolKind::Function` com qualified começando em
// `route::`) + `CallSite`s ligando a rota ao handler.

use std::path::Path;

use super::types::{CallSite, Symbol, SymbolKind};

pub mod grails;
pub mod nestjs;
pub mod rails;

/// Uma rota HTTP detectada.
#[derive(Debug, Clone)]
pub struct RouteMapping {
    /// Método HTTP (GET, POST, PUT, DELETE, PATCH, ANY).
    pub method: String,
    /// Padrão de URL (ex: `/users/:id`).
    pub path: String,
    /// Identificador qualificado do handler (ex: `app/controllers/users_controller.rb::show`).
    pub handler: String,
    /// Arquivo onde a rota está definida.
    pub source_file: String,
    /// Linha onde a rota está definida.
    pub source_line: u32,
}

impl RouteMapping {
    /// Converte para símbolo sintético + call site no grafo.
    pub fn into_graph_entries(self, language: &str) -> (Symbol, CallSite) {
        let route_qualified = format!("route::{} {}", self.method, self.path);
        let sym = Symbol {
            name: format!("{} {}", self.method, self.path),
            qualified: route_qualified.clone(),
            kind: SymbolKind::Function,
            file: self.source_file.clone(),
            line: self.source_line,
            language: language.to_string(),
        };
        let call = CallSite {
            caller_qualified: route_qualified,
            callee_name: handler_name(&self.handler),
            file: self.source_file,
            line: self.source_line,
        };
        (sym, call)
    }
}

/// Extrai o nome simples do callee a partir do qualified do handler.
/// Ex: `app/controllers/users_controller.rb::show` → `show`.
fn handler_name(qualified: &str) -> String {
    qualified
        .rsplit("::")
        .next()
        .unwrap_or(qualified)
        .to_string()
}

/// Contrato comum para detectores de framework.
pub trait FrameworkRouter {
    /// Indica se este router se aplica ao arquivo dado.
    fn detect(&self, path: &Path, content: &str) -> bool;
    /// Extrai mappings de URL → handler do arquivo.
    fn extract(&self, path: &Path, content: &str) -> Vec<RouteMapping>;
    /// Linguagem destes mappings (para popular `Symbol.language`).
    fn language(&self) -> &'static str;
}

/// Executa todos os routers conhecidos contra um arquivo e devolve as rotas
/// encontradas. Os routers são mutuamente exclusivos por path/conteúdo, então
/// na prática só um devolve resultados por arquivo.
pub fn detect_routes(path: &Path, content: &str) -> Vec<(RouteMapping, &'static str)> {
    let routers: Vec<Box<dyn FrameworkRouter>> = vec![
        Box::new(rails::RailsRouter),
        Box::new(grails::GrailsRouter),
        Box::new(nestjs::NestJsRouter),
    ];

    let mut all = Vec::new();
    for r in &routers {
        if r.detect(path, content) {
            for mapping in r.extract(path, content) {
                all.push((mapping, r.language()));
            }
        }
    }
    all
}
