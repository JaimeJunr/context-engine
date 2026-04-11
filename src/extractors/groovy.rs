use tree_sitter::{Language, Node, Parser};
use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_groovy() -> *const ();
}

fn language() -> Language {
    unsafe { LanguageFn::from_raw(tree_sitter_groovy) }.into()
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
            if let Some(name) = node.child_by_field_name("name") {
                let params = node
                    .child_by_field_name("parameters")
                    .map(|p| src_slice(p, src))
                    .unwrap_or_default();
                sigs.push(format!("{}  def {}{}", pad, src_slice(name, src), params));
            }
        }
        "field_declaration" => {
            if depth > 0 {
                let text = src_slice(node, src);
                let first_line = text.split('\n').next().unwrap_or("").trim().to_string();
                if !first_line.is_empty() {
                    sigs.push(format!("{}  {}", pad, first_line));
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
    match node.kind() {
        "import_declaration" => {
            let text = src_slice(node, src);
            let cleaned = text.replace("import", "").trim().trim_end_matches(';').to_string();
            let last = cleaned.split('.').last().unwrap_or("").trim().to_string();
            if !last.is_empty() && last != "*" {
                refs.push(last);
            }
        }
        "class_declaration" => {
            if let Some(superclass) = node.child_by_field_name("superclass") {
                let name = src_slice(superclass, src);
                let last = name.split('.').last().unwrap_or(&name).trim().to_string();
                if last.starts_with(|c: char| c.is_uppercase()) {
                    refs.push(last);
                }
            }
        }
        _ => {}
    }
    for child in node.children(&mut node.walk()) {
        walk_refs(child, src, refs);
    }
}
