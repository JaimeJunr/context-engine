// Extração de assinaturas Rust para o pipeline `map`.
//
// Captura: fn, struct, enum, trait, impl blocks, mods. Sem bodies.

use tree_sitter::{Language, Node, Parser};

fn language() -> Language {
    tree_sitter_rust::LANGUAGE.into()
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
        "function_item" => {
            // Reconstrói assinatura: visibilidade + fn + nome + params + return.
            let mut header = String::new();
            for child in node.children(&mut node.walk()) {
                match child.kind() {
                    "visibility_modifier" => {
                        header.push_str(&src_slice(child, src));
                        header.push(' ');
                    }
                    "function_modifiers" => {
                        header.push_str(&src_slice(child, src));
                        header.push(' ');
                    }
                    _ => {}
                }
            }
            header.push_str("fn ");
            if let Some(name) = node.child_by_field_name("name") {
                header.push_str(&src_slice(name, src));
            }
            if let Some(params) = node.child_by_field_name("parameters") {
                header.push_str(&src_slice(params, src));
            }
            if let Some(ret) = node.child_by_field_name("return_type") {
                header.push_str(" -> ");
                header.push_str(&src_slice(ret, src));
            }
            sigs.push(format!("{}  {}", pad, header));
        }
        "struct_item" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}struct {}", pad, src_slice(name, src)));
            }
        }
        "enum_item" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}enum {}", pad, src_slice(name, src)));
            }
        }
        "trait_item" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}trait {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "impl_item" => {
            // Captura "impl Type" ou "impl Trait for Type".
            let header_text = src_slice(node, src);
            let first_line = header_text.lines().next().unwrap_or("");
            // Mantém só até a abertura de bloco "{"
            let header = first_line.split('{').next().unwrap_or(first_line).trim();
            sigs.push(format!("{}{}", pad, header));
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "mod_item" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}mod {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
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
                "field_expression" => {
                    if let Some(field) = func.child_by_field_name("field") {
                        refs.push(src_slice(field, src));
                    }
                }
                "scoped_identifier" => {
                    if let Some(name) = func.child_by_field_name("name") {
                        refs.push(src_slice(name, src));
                    }
                }
                _ => {}
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
    fn extrai_function_struct_enum_trait() {
        let src = b"
pub struct User { id: u64 }
pub enum Role { Admin, User }
pub trait Repo { fn find(&self, id: u64) -> Option<User>; }
pub fn create_user(name: &str) -> User { User { id: 1 } }
";
        let sigs = extract(src);
        assert!(
            sigs.iter().any(|s| s.contains("struct User")),
            "missing struct: {:?}",
            sigs
        );
        assert!(sigs.iter().any(|s| s.contains("enum Role")));
        assert!(sigs.iter().any(|s| s.contains("trait Repo")));
        assert!(sigs.iter().any(|s| s.contains("fn create_user")));
    }

    #[test]
    fn impl_block_captura_methods() {
        let src = b"
impl User {
    pub fn id(&self) -> u64 { self.id }
}
";
        let sigs = extract(src);
        assert!(sigs
            .iter()
            .any(|s| s.starts_with("impl") && s.contains("User")));
        assert!(sigs.iter().any(|s| s.contains("fn id")));
    }

    #[test]
    fn extract_refs_call_e_method() {
        let src = b"fn main() { create_user(\"a\"); db.find(1); }";
        let refs = extract_refs(src);
        assert!(refs.iter().any(|r| r == "create_user"));
        assert!(refs.iter().any(|r| r == "find"));
    }
}
