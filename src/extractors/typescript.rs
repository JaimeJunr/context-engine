use tree_sitter::{Language, Node, Parser};

fn ts_language() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

fn tsx_language() -> Language {
    tree_sitter_typescript::LANGUAGE_TSX.into()
}

fn src_slice(node: Node, src: &[u8]) -> String {
    String::from_utf8_lossy(&src[node.start_byte()..node.end_byte()]).into_owned()
}

pub fn extract(src: &[u8], is_tsx: bool) -> Vec<String> {
    let mut parser = Parser::new();
    let lang = if is_tsx { tsx_language() } else { ts_language() };
    parser.set_language(&lang).unwrap();
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
        "interface_declaration" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}interface {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "class_declaration" | "class" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}class {}", pad, src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "type_alias_declaration" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}type {}", pad, src_slice(name, src)));
            }
        }
        "function_declaration" | "method_definition" => {
            if let Some(name) = node.child_by_field_name("name") {
                let params = node
                    .child_by_field_name("parameters")
                    .map(|p| src_slice(p, src))
                    .unwrap_or_default();
                sigs.push(format!("{}  function {}{}", pad, src_slice(name, src), params));
            }
        }
        "export_statement" => {
            // Pass through to children
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth, sigs);
            }
            return;
        }
        _ => {}
    }

    for child in node.children(&mut node.walk()) {
        walk_sigs(child, src, depth, sigs);
    }
}

pub fn extract_refs(src: &[u8], is_tsx: bool) -> Vec<String> {
    let mut parser = Parser::new();
    let lang = if is_tsx { tsx_language() } else { ts_language() };
    parser.set_language(&lang).unwrap();
    let Some(tree) = parser.parse(src, None) else {
        return vec![];
    };
    let mut refs = Vec::new();
    walk_refs(tree.root_node(), src, &mut refs);
    refs
}

fn walk_refs(node: Node, src: &[u8], refs: &mut Vec<String>) {
    match node.kind() {
        "import_statement" => {
            // Look for import clause
            if let Some(clause) = node.child_by_field_name("import") {
                for child in clause.named_children(&mut clause.walk()) {
                    if child.kind() == "import_specifier" {
                        if let Some(name) = child.child_by_field_name("name") {
                            refs.push(src_slice(name, src));
                        }
                    }
                }
            } else {
                for child in node.children(&mut node.walk()) {
                    if matches!(child.kind(), "identifier" | "type_identifier") {
                        refs.push(src_slice(child, src));
                    }
                }
            }
        }
        "class_declaration" | "class" => {
            for child in node.children(&mut node.walk()) {
                if child.kind() == "class_heritage" {
                    for c in child.children(&mut child.walk()) {
                        if matches!(c.kind(), "identifier" | "type_identifier") {
                            refs.push(src_slice(c, src));
                        }
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
