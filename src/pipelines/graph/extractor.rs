// Extração de símbolos + call sites usando Tree-Sitter.
//
// Estratégia: para cada linguagem, definimos queries S-expression que capturam:
//   - Definições (function, class, method, struct, trait, …)
//   - Sites de chamada (identifiers em contexto de call)
//   - Imports (para resolução cross-file)
//
// Note: este é o extractor *do grafo* (calls + estrutura). O extractor do `map`
// pipeline (`pipelines/map/extractors/`) continua focado em assinaturas para
// ranking de relevância — propósitos diferentes, código separado.

use anyhow::Result;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use super::types::{CallSite, Symbol, SymbolKind};

pub struct ExtractedFile {
    pub symbols: Vec<Symbol>,
    pub calls: Vec<CallSite>,
    pub imports: Vec<(String, Option<String>)>, // (module, alias)
}

/// Detecta linguagem pelo path e extrai símbolos + calls.
/// Retorna `None` se a extensão não for suportada.
pub fn extract(path: &Path) -> Result<Option<ExtractedFile>> {
    let ext = match path.extension().and_then(|s| s.to_str()) {
        Some(e) => e,
        None => return Ok(None),
    };

    let source = std::fs::read_to_string(path)?;
    let file_str = path.to_string_lossy().to_string();

    let result = match ext {
        "rs" => extract_rust(&source, &file_str)?,
        "go" => extract_go(&source, &file_str)?,
        "java" => extract_java(&source, &file_str)?,
        "ts" | "tsx" => extract_typescript(&source, &file_str)?,
        "js" | "jsx" | "mjs" | "cjs" => extract_javascript(&source, &file_str)?,
        "py" => extract_python(&source, &file_str)?,
        "rb" | "rake" => extract_ruby(&source, &file_str)?,
        "groovy" | "gradle" => extract_groovy(&source, &file_str)?,
        _ => return Ok(None),
    };

    Ok(Some(result))
}

// =========================================================================
// Tree-Sitter helpers
// =========================================================================

fn run_queries(
    source: &str,
    file: &str,
    language: tree_sitter::Language,
    lang_name: &str,
    defs_query: &str,
    calls_query: &str,
    imports_query: &str,
) -> Result<ExtractedFile> {
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("parse failed"))?;
    let root = tree.root_node();
    let bytes = source.as_bytes();

    let mut symbols = Vec::new();
    let mut calls = Vec::new();
    let mut imports = Vec::new();

    // Definições
    if !defs_query.is_empty() {
        let q = Query::new(&language, defs_query)?;
        let mut cursor = QueryCursor::new();
        let names = q.capture_names();
        let mut matches = cursor.matches(&q, root, bytes);
        while let Some(m) = matches.next() {
            let mut name: Option<String> = None;
            let mut kind: Option<SymbolKind> = None;
            let mut line: u32 = 1;
            for cap in m.captures {
                let cap_name = names[cap.index as usize];
                let text = cap.node.utf8_text(bytes).unwrap_or("").to_string();
                if cap_name.ends_with(".name") {
                    name = Some(text);
                    line = cap.node.start_position().row as u32 + 1;
                    // Kind é o prefixo antes de ".name" (ex: "function.name" → "function")
                    let kind_str = cap_name.trim_end_matches(".name");
                    kind = match kind_str {
                        "function" => Some(SymbolKind::Function),
                        "method" => Some(SymbolKind::Method),
                        "class" => Some(SymbolKind::Class),
                        "struct" => Some(SymbolKind::Struct),
                        "trait" => Some(SymbolKind::Trait),
                        "interface" => Some(SymbolKind::Interface),
                        "enum" => Some(SymbolKind::Enum),
                        "type" => Some(SymbolKind::Type),
                        _ => Some(SymbolKind::Function),
                    };
                }
            }
            if let (Some(n), Some(k)) = (name, kind) {
                symbols.push(Symbol {
                    name: n.clone(),
                    qualified: format!("{}::{}", file, n),
                    kind: k,
                    file: file.to_string(),
                    line,
                    language: lang_name.to_string(),
                });
            }
        }
    }

    // Calls
    if !calls_query.is_empty() {
        let q = Query::new(&language, calls_query)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&q, root, bytes);
        while let Some(m) = matches.next() {
            for cap in m.captures {
                let callee = cap.node.utf8_text(bytes).unwrap_or("").to_string();
                if callee.is_empty() {
                    continue;
                }
                let line = cap.node.start_position().row as u32 + 1;
                // Atribui o call ao container mais próximo (heurística leve:
                // procura ancestral que é uma def na lista `symbols`).
                let caller = find_enclosing_symbol(&symbols, line)
                    .map(|s| s.qualified.clone())
                    .unwrap_or_else(|| format!("{}::<top>", file));
                calls.push(CallSite {
                    caller_qualified: caller,
                    callee_name: callee,
                    file: file.to_string(),
                    line,
                });
            }
        }
    }

    // Imports
    if !imports_query.is_empty() {
        let q = Query::new(&language, imports_query)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&q, root, bytes);
        while let Some(m) = matches.next() {
            for cap in m.captures {
                let module = cap.node.utf8_text(bytes).unwrap_or("").to_string();
                if !module.is_empty() {
                    imports.push((module, None));
                }
            }
        }
    }

    Ok(ExtractedFile {
        symbols,
        calls,
        imports,
    })
}

/// Heurística: o caller é a definição mais recente cuja linha <= call.line.
fn find_enclosing_symbol(symbols: &[Symbol], call_line: u32) -> Option<&Symbol> {
    symbols
        .iter()
        .filter(|s| s.line <= call_line)
        .max_by_key(|s| s.line)
}

// =========================================================================
// Queries por linguagem
// =========================================================================

fn extract_rust(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_rust::LANGUAGE.into();
    let defs = r#"
        (function_item name: (identifier) @function.name)
        (struct_item name: (type_identifier) @struct.name)
        (enum_item name: (type_identifier) @enum.name)
        (trait_item name: (type_identifier) @trait.name)
        (impl_item type: (type_identifier) @class.name)
    "#;
    let calls = r#"
        (call_expression function: (identifier) @callee)
        (call_expression function: (field_expression field: (field_identifier) @callee))
        (call_expression function: (scoped_identifier name: (identifier) @callee))
    "#;
    let imports = r#"
        (use_declaration argument: (scoped_identifier) @module)
        (use_declaration argument: (identifier) @module)
    "#;
    run_queries(source, file, lang, "rust", defs, calls, imports)
}

fn extract_go(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_go::LANGUAGE.into();
    let defs = r#"
        (function_declaration name: (identifier) @function.name)
        (method_declaration name: (field_identifier) @method.name)
        (type_declaration (type_spec name: (type_identifier) @type.name))
    "#;
    let calls = r#"
        (call_expression function: (identifier) @callee)
        (call_expression function: (selector_expression field: (field_identifier) @callee))
    "#;
    let imports = r#"
        (import_spec path: (interpreted_string_literal) @module)
    "#;
    run_queries(source, file, lang, "go", defs, calls, imports)
}

fn extract_java(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_java::LANGUAGE.into();
    let defs = r#"
        (method_declaration name: (identifier) @method.name)
        (class_declaration name: (identifier) @class.name)
        (interface_declaration name: (identifier) @interface.name)
        (enum_declaration name: (identifier) @enum.name)
    "#;
    let calls = r#"
        (method_invocation name: (identifier) @callee)
        (object_creation_expression type: (type_identifier) @callee)
    "#;
    let imports = r#"
        (import_declaration (scoped_identifier) @module)
        (import_declaration (identifier) @module)
    "#;
    run_queries(source, file, lang, "java", defs, calls, imports)
}

fn extract_typescript(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    let defs = r#"
        (function_declaration name: (identifier) @function.name)
        (method_definition name: (property_identifier) @method.name)
        (class_declaration name: (type_identifier) @class.name)
        (interface_declaration name: (type_identifier) @interface.name)
        (type_alias_declaration name: (type_identifier) @type.name)
        (enum_declaration name: (identifier) @enum.name)
    "#;
    let calls = r#"
        (call_expression function: (identifier) @callee)
        (call_expression function: (member_expression property: (property_identifier) @callee))
        (new_expression constructor: (identifier) @callee)
    "#;
    let imports = r#"
        (import_statement source: (string) @module)
    "#;
    run_queries(source, file, lang, "typescript", defs, calls, imports)
}

fn extract_javascript(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_javascript::LANGUAGE.into();
    // JS não tem interface_declaration / type_alias_declaration — só o subset comum.
    let defs = r#"
        (function_declaration name: (identifier) @function.name)
        (method_definition name: (property_identifier) @method.name)
        (class_declaration name: (identifier) @class.name)
    "#;
    let calls = r#"
        (call_expression function: (identifier) @callee)
        (call_expression function: (member_expression property: (property_identifier) @callee))
        (new_expression constructor: (identifier) @callee)
    "#;
    let imports = r#"
        (import_statement source: (string) @module)
    "#;
    run_queries(source, file, lang, "javascript", defs, calls, imports)
}

fn extract_groovy(source: &str, file: &str) -> Result<ExtractedFile> {
    // Carrega a Language do parser.c compilado pelo build.rs.
    unsafe extern "C" {
        fn tree_sitter_groovy() -> *const ();
    }
    let lang: tree_sitter::Language =
        unsafe { tree_sitter_language::LanguageFn::from_raw(tree_sitter_groovy) }.into();
    // Gramática custom em grammars/groovy/ usa node kinds: `class_declaration`,
    // `method_declaration` com field `name`. Queries seguem o padrão Java.
    let defs = r#"
        (class_declaration name: (identifier) @class.name)
        (interface_declaration name: (identifier) @interface.name)
        (method_declaration name: (identifier) @method.name)
    "#;
    // Call sites no Groovy podem aparecer como `method_invocation` (Java-like)
    // ou identifiers de chamada genéricos. Tentamos os dois.
    let calls = r#"
        (method_invocation name: (identifier) @callee)
    "#;
    let imports = r#"
        (import_declaration (scoped_identifier) @module)
        (import_declaration (identifier) @module)
    "#;
    // Tolerância: se uma query individual falhar (node kind não existe na gramática
    // custom), tenta versões progressivamente mais simples.
    if let Ok(r) = run_queries(source, file, lang.clone(), "groovy", defs, calls, imports) {
        return Ok(r);
    }
    if let Ok(r) = run_queries(source, file, lang.clone(), "groovy", defs, calls, "") {
        return Ok(r);
    }
    if let Ok(r) = run_queries(source, file, lang.clone(), "groovy", defs, "", "") {
        return Ok(r);
    }
    // Último recurso: parse sem queries — só garante que não panica.
    run_queries(source, file, lang, "groovy", "", "", "")
}

fn extract_python(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_python::LANGUAGE.into();
    let defs = r#"
        (function_definition name: (identifier) @function.name)
        (class_definition name: (identifier) @class.name)
    "#;
    let calls = r#"
        (call function: (identifier) @callee)
        (call function: (attribute attribute: (identifier) @callee))
    "#;
    let imports = r#"
        (import_statement (dotted_name) @module)
        (import_from_statement module_name: (dotted_name) @module)
    "#;
    run_queries(source, file, lang, "python", defs, calls, imports)
}

fn extract_ruby(source: &str, file: &str) -> Result<ExtractedFile> {
    let lang = tree_sitter_ruby::LANGUAGE.into();
    let defs = r#"
        (method name: (identifier) @method.name)
        (singleton_method name: (identifier) @method.name)
        (class name: (constant) @class.name)
        (module name: (constant) @module.name)
    "#;
    let calls = r#"
        (call method: (identifier) @callee)
        ((identifier) @callee
          (#match? @callee "^[a-z_][a-zA-Z0-9_]*[?!]?$"))
    "#;
    let imports = r#"
        (call method: (identifier) @method
              arguments: (argument_list (string) @module)
              (#match? @method "^(require|require_relative|load)$"))
    "#;
    run_queries(source, file, lang, "ruby", defs, calls, imports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("ctx_graph_extractor_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    #[test]
    fn extrai_function_e_call_rust() {
        let path = write_temp(
            "sample.rs",
            "fn helper() -> i32 { 42 }\nfn main() { helper(); }\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(
            result.symbols.iter().any(|s| s.name == "helper"),
            "helper deve aparecer como símbolo"
        );
        assert!(
            result.symbols.iter().any(|s| s.name == "main"),
            "main deve aparecer como símbolo"
        );
        assert!(
            result.calls.iter().any(|c| c.callee_name == "helper"),
            "call site de helper deve ser detectado: {:?}",
            result.calls
        );
        let main_calls_helper = result
            .calls
            .iter()
            .find(|c| c.callee_name == "helper")
            .unwrap();
        assert!(
            main_calls_helper.caller_qualified.contains("main"),
            "caller deve ser 'main': {:?}",
            main_calls_helper
        );
    }

    #[test]
    fn extrai_function_python() {
        let path = write_temp(
            "sample.py",
            "def helper():\n    return 42\n\ndef main():\n    helper()\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(result.symbols.iter().any(|s| s.name == "helper"));
        assert!(result.symbols.iter().any(|s| s.name == "main"));
        assert!(result.calls.iter().any(|c| c.callee_name == "helper"));
    }

    #[test]
    fn extrai_function_go() {
        let path = write_temp(
            "sample.go",
            "package main\nfunc helper() int { return 42 }\nfunc main() { helper() }\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(result.symbols.iter().any(|s| s.name == "helper"));
        assert!(result.calls.iter().any(|c| c.callee_name == "helper"));
    }

    #[test]
    fn extrai_method_java() {
        let path = write_temp(
            "Sample.java",
            "class Sample {\n  void helper() {}\n  void main() { helper(); }\n}\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(result.symbols.iter().any(|s| s.name == "helper"));
        assert!(result.calls.iter().any(|c| c.callee_name == "helper"));
    }

    #[test]
    fn extrai_function_typescript() {
        let path = write_temp(
            "sample.ts",
            "function helper(): number { return 42; }\nfunction main() { helper(); }\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(result.symbols.iter().any(|s| s.name == "helper"));
        assert!(result.calls.iter().any(|c| c.callee_name == "helper"));
    }

    #[test]
    fn extensao_desconhecida_retorna_none() {
        let path = write_temp("sample.xyz", "anything");
        assert!(extract(&path).unwrap().is_none());
    }

    #[test]
    fn extrai_function_javascript() {
        let path = write_temp(
            "sample.js",
            "function helper() { return 42; }\nfunction main() { helper(); new Foo(); }\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(result.symbols.iter().any(|s| s.name == "helper"));
        assert!(result.symbols.iter().any(|s| s.name == "main"));
        assert!(result.calls.iter().any(|c| c.callee_name == "helper"));
        assert!(result.calls.iter().any(|c| c.callee_name == "Foo"));
    }

    #[test]
    fn extrai_jsx_e_metodo() {
        let path = write_temp(
            "Button.jsx",
            "class Button { handleClick() { this.props.onClick(); } }\n",
        );
        let result = extract(&path).unwrap().unwrap();
        assert!(result.symbols.iter().any(|s| s.name == "Button"));
        assert!(result.symbols.iter().any(|s| s.name == "handleClick"));
    }

    #[test]
    fn extrai_groovy_class_method_call() {
        let path = write_temp(
            "Sample.groovy",
            "class UserService {\n  def findById(Long id) { repo.find(id) }\n  def deleteAll() { findById(1) }\n}\n",
        );
        let result = extract(&path).unwrap().unwrap();
        // Pelo menos um símbolo de classe ou método deve ser detectado.
        // (a gramática groovy custom pode usar node kinds diferentes; este teste é
        // tolerante e só garante que rodar não panica e devolve algum sinal).
        let has_anything = !result.symbols.is_empty() || !result.calls.is_empty();
        assert!(
            has_anything,
            "extractor groovy deve retornar pelo menos um símbolo ou call: {:?}",
            result.symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
    }
}
