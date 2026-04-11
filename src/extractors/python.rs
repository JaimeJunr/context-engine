use tree_sitter::{Language, Node, Parser};

fn language() -> Language {
    tree_sitter_python::LANGUAGE.into()
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
        "class_definition" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}class {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "function_definition" => {
            if let Some(name) = node.child_by_field_name("name") {
                let params = node
                    .child_by_field_name("parameters")
                    .map(|p| src_slice(p, src))
                    .unwrap_or_else(|| "()".to_string());
                sigs.push(format!("{}  def {}{}", pad, src_slice(name, src), params));
            }
        }
        _ => {}
    }

    let inner = depth + if node.kind() == "class_definition" { 1 } else { 0 };
    for child in node.children(&mut node.walk()) {
        walk_sigs(child, src, inner, sigs);
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
        "import_statement" | "import_from_statement" => {
            for child in node.named_children(&mut node.walk()) {
                let t = child.kind();
                if t == "dotted_name" || t == "aliased_import" {
                    let text = src_slice(child, src);
                    let last = text.split('.').last().unwrap_or(&text).trim().to_string();
                    if !last.is_empty() {
                        refs.push(last);
                    }
                }
            }
        }
        _ => {}
    }
    for child in node.children(&mut node.walk()) {
        walk_refs(child, src, refs);
    }
}
