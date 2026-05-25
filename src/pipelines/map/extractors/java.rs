// Extração de assinaturas Java para o pipeline `map`.
//
// Captura: classes, interfaces, enums, métodos, anotações próximas (úteis para
// Spring/JPA). Não extrai bodies — só assinatura compacta.

use tree_sitter::{Language, Node, Parser};

fn language() -> Language {
    tree_sitter_java::LANGUAGE.into()
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
        "interface_declaration" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}interface {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "enum_declaration" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}enum {}", pad, src_slice(name, src)));
            }
        }
        "method_declaration" => {
            let modifiers = node
                .child_by_field_name("modifiers")
                .map(|n| src_slice(n, src) + " ")
                .unwrap_or_default();
            let return_type = node
                .child_by_field_name("type")
                .map(|n| src_slice(n, src))
                .unwrap_or_default();
            let name = node
                .child_by_field_name("name")
                .map(|n| src_slice(n, src))
                .unwrap_or_default();
            let params = node
                .child_by_field_name("parameters")
                .map(|n| src_slice(n, src))
                .unwrap_or_else(|| "()".to_string());
            sigs.push(format!(
                "{}  {}{} {}{}",
                pad,
                modifiers.trim(),
                if modifiers.trim().is_empty() { "" } else { " " },
                format!("{} {}", return_type, name).trim(),
                params
            ));
        }
        "constructor_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| src_slice(n, src))
                .unwrap_or_default();
            let params = node
                .child_by_field_name("parameters")
                .map(|n| src_slice(n, src))
                .unwrap_or_else(|| "()".to_string());
            sigs.push(format!("{}  ctor {}{}", pad, name, params));
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
    if node.kind() == "method_invocation" {
        if let Some(name) = node.child_by_field_name("name") {
            refs.push(src_slice(name, src));
        }
    }
    if node.kind() == "object_creation_expression" {
        if let Some(t) = node.child_by_field_name("type") {
            refs.push(src_slice(t, src));
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
    fn extrai_classe_e_metodo() {
        let src = b"public class UserService { public User findById(Long id) { return null; } }";
        let sigs = extract(src);
        assert!(sigs.iter().any(|s| s.contains("class UserService")));
        assert!(sigs.iter().any(|s| s.contains("findById")));
    }

    #[test]
    fn extrai_interface() {
        let src = b"public interface UserRepository { User findById(Long id); }";
        let sigs = extract(src);
        assert!(sigs.iter().any(|s| s.contains("interface UserRepository")));
    }

    #[test]
    fn extrai_refs_method_call() {
        let src = b"class A { void m() { findById(1L); new User(); } }";
        let refs = extract_refs(src);
        assert!(refs.iter().any(|r| r == "findById"));
        assert!(refs.iter().any(|r| r == "User"));
    }
}
