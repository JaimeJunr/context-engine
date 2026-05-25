// Tipos compartilhados pelo pipeline graph.

use serde::{Deserialize, Serialize};

/// Categoria do símbolo no grafo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Trait,
    Interface,
    Enum,
    Module,
    Constant,
    Variable,
    Type,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Class => "class",
            SymbolKind::Struct => "struct",
            SymbolKind::Trait => "trait",
            SymbolKind::Interface => "interface",
            SymbolKind::Enum => "enum",
            SymbolKind::Module => "module",
            SymbolKind::Constant => "constant",
            SymbolKind::Variable => "variable",
            SymbolKind::Type => "type",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "function" => SymbolKind::Function,
            "method" => SymbolKind::Method,
            "class" => SymbolKind::Class,
            "struct" => SymbolKind::Struct,
            "trait" => SymbolKind::Trait,
            "interface" => SymbolKind::Interface,
            "enum" => SymbolKind::Enum,
            "module" => SymbolKind::Module,
            "constant" => SymbolKind::Constant,
            "variable" => SymbolKind::Variable,
            "type" => SymbolKind::Type,
            _ => return None,
        })
    }
}

/// Definição de um símbolo no grafo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: u32,
    /// Identificador qualificado: `module::Class::method` — útil para distinguir
    /// símbolos com mesmo nome em escopos diferentes.
    pub qualified: String,
    /// Linguagem do arquivo, ex: "rust", "typescript".
    pub language: String,
}

/// Local de uma chamada (call site) — caller invoca callee em `file:line`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSite {
    pub caller_qualified: String,
    pub callee_name: String,
    pub file: String,
    pub line: u32,
}

/// Nó do grafo retornado pelas queries (símbolo + metadata útil para o agente).
#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub symbol: Symbol,
    /// Score de relevância (BM25 + PageRank). Maior = mais relevante.
    pub score: f64,
    /// Locais onde o símbolo é referenciado (file:line).
    pub sites: Vec<String>,
}

/// Opções comuns para queries do grafo.
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Query opcional usada para BM25 ranking dos resultados.
    pub query: Option<String>,
    /// Budget de tokens (aproximado, 1 token ≈ 4 chars).
    pub max_tokens: Option<usize>,
    /// Profundidade máxima de trace (default: 3).
    pub depth: Option<usize>,
}
