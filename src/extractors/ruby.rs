use tree_sitter::{Language, Node, Parser};

fn language() -> Language {
    tree_sitter_ruby::LANGUAGE.into()
}

fn src_slice(node: Node, src: &[u8]) -> String {
    String::from_utf8_lossy(&src[node.start_byte()..node.end_byte()]).into_owned()
}

const RAILS_MACROS: &[&str] = &[
    "attr_accessor", "attr_reader", "attr_writer",
    "belongs_to", "has_many", "has_one", "has_and_belongs_to_many",
    "scope", "validates", "before_action", "after_action",
];

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
        "class" | "module" => {
            if let Some(name) = node.child_by_field_name("name") {
                sigs.push(format!("{}{} {}", pad, node.kind(), src_slice(name, src)));
            }
            for child in node.children(&mut node.walk()) {
                walk_sigs(child, src, depth + 1, sigs);
            }
            return;
        }
        "method" => {
            if let Some(name) = node.child_by_field_name("name") {
                let params = node
                    .child_by_field_name("parameters")
                    .map(|p| src_slice(p, src))
                    .unwrap_or_default();
                sigs.push(format!("{}  def {}{}", pad, src_slice(name, src), params));
            }
        }
        "singleton_method" => {
            if let Some(name) = node.child_by_field_name("name") {
                let params = node
                    .child_by_field_name("parameters")
                    .map(|p| src_slice(p, src))
                    .unwrap_or_default();
                sigs.push(format!("{}  def self.{}{}", pad, src_slice(name, src), params));
            }
        }
        "call" => {
            let method_text = node
                .children(&mut node.walk())
                .next()
                .map(|c| src_slice(c, src))
                .unwrap_or_default();
            if RAILS_MACROS.contains(&method_text.as_str()) {
                let first_line = src_slice(node, src)
                    .split('\n')
                    .next()
                    .unwrap_or("")
                    .to_string();
                sigs.push(format!("{}  {}", pad, first_line));
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
        "class" => {
            if let Some(superclass) = node.child_by_field_name("superclass") {
                let name = src_slice(superclass, src);
                let last = name.split("::").last().unwrap_or(&name).trim().to_string();
                if last.starts_with(|c: char| c.is_uppercase()) {
                    refs.push(last);
                }
            }
        }
        "call" => {
            let method_text = node
                .children(&mut node.walk())
                .next()
                .map(|c| src_slice(c, src))
                .unwrap_or_default();
            if matches!(method_text.as_str(), "include" | "extend" | "prepend") {
                if let Some(args) = node.child_by_field_name("arguments") {
                    for child in args.children(&mut args.walk()) {
                        if !matches!(child.kind(), "," | "(" | ")") {
                            let name = src_slice(child, src);
                            let last = name.split("::").last().unwrap_or(&name).trim().to_string();
                            if last.starts_with(|c: char| c.is_uppercase()) {
                                refs.push(last);
                            }
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
