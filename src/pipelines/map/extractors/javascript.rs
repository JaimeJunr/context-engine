// Extração de assinaturas JavaScript puro (.js, .jsx) para o pipeline `map`.
//
// Tree-sitter para JS é diferente do de TS — JS não tem `type_alias_declaration`,
// `interface_declaration`, etc. Compartilha boa parte mas vale ter extractor
// próprio para evitar warnings de query nodes inexistentes.

use tree_sitter::{Language, Node, Parser};

fn language() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}

fn src_slice(node: Node, src: &[u8]) -> String {
    String::from_utf8_lossy(&src[node.start_byte()..node.end_byte()]).into_owned()
}

pub fn extract(src: &[u8]) -> Vec<String> {
    let mut parser = Parser::new();
    parser.set_language(&language()).unwrap();
    let Some(tree) = parser.parse(src, None) else {
        return vec![];
    };
    let mut sigs = Vec::new();
    walk_sigs(tree.root_node(), src, 0, &mut sigs);
    sigs
}

fn walk_sigs(node: Node, src: &[u8], depth: usize, sigs: &mut Vec<String>) {
    let pad = "  ".repeat(depth);
    match node.kind() {
        "class_declaration" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}class {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "function_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| src_slice(n, src))
                .unwrap_or_default();
            let params = node
                .child_by_field_name("parameters")
                .map(|n| src_slice(n, src))
                .unwrap_or_else(|| "()".to_string());
            sigs.push(format!("{}function {}{}", pad, name, params));
        }
        "method_definition" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| src_slice(n, src))
                .unwrap_or_default();
            let params = node
                .child_by_field_name("parameters")
                .map(|n| src_slice(n, src))
                .unwrap_or_else(|| "()".to_string());
            sigs.push(format!("{}  {}{}", pad, name, params));
        }
        "lexical_declaration" | "variable_declaration" => {
            // const foo = (x) => ... — captura nomes "top-level" como sinais.
            for child in node.children(&mut node.walk()) {
                if child.kind() == "variable_declarator" {
                    if let Some(name) = child.child_by_field_name("name") {
                        let value_kind = child
                            .child_by_field_name("value")
                            .map(|v| v.kind())
                            .unwrap_or("");
                        if matches!(value_kind, "arrow_function" | "function_expression") {
                            sigs.push(format!("{}const {} = ...", pad, src_slice(name, src)));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        walk_sigs(child, src, depth, sigs);
    }
}

pub fn extract_refs(src: &[u8]) -> Vec<String> {
    let mut parser = Parser::new();
    parser.set_language(&language()).unwrap();
    let Some(tree) = parser.parse(src, None) else {
        return vec![];
    };
    let mut refs = Vec::new();
    walk_refs(tree.root_node(), src, &mut refs);
    refs
}

fn walk_refs(node: Node, src: &[u8], refs: &mut Vec<String>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            match func.kind() {
                "identifier" => refs.push(src_slice(func, src)),
                "member_expression" => {
                    if let Some(prop) = func.child_by_field_name("property") {
                        refs.push(src_slice(prop, src));
                    }
                }
                _ => {}
            }
        }
    }
    if node.kind() == "new_expression" {
        if let Some(constr) = node.child_by_field_name("constructor") {
            if constr.kind() == "identifier" {
                refs.push(src_slice(constr, src));
            }
        }
    }
    for child in node.children(&mut node.walk()) {
        walk_refs(child, src, refs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrai_function_class_arrow() {
        let src = b"
function greet(name) { return 'hi ' + name; }
class Foo { bar() { return 1; } }
const baz = () => 42;
";
        let sigs = extract(src);
        assert!(
            sigs.iter().any(|s| s.contains("function greet")),
            "missing greet: {:?}",
            sigs
        );
        assert!(sigs.iter().any(|s| s.contains("class Foo")));
        assert!(sigs.iter().any(|s| s.contains("baz")));
    }

    #[test]
    fn extract_refs_chama_funcoes_e_metodos() {
        let src = b"
function main() {
  greet('Alice');
  user.save();
  new Logger();
}
";
        let refs = extract_refs(src);
        assert!(refs.iter().any(|r| r == "greet"));
        assert!(refs.iter().any(|r| r == "save"));
        assert!(refs.iter().any(|r| r == "Logger"));
    }
}
