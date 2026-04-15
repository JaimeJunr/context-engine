# Instruções: `src/extractors/` — Extração de Assinaturas

Módulo responsável por **extrair assinaturas de código** (funções, classes, tipos) via Tree-Sitter.

## Estrutura

```
extractors/
├── mod.rs          # Trait Extractor + dispatch por extensão
├── typescript.rs   # TypeScript/TSX (interfaces, classes, tipos)
├── python.rs       # Python (classes, funções, tipos)
├── ruby.rs         # Ruby (classes, métodos, módulos)
└── groovy.rs       # Groovy (sintaxe customizada em grammars/)
```

## Trait `Extractor`

```rust
pub trait Extractor: Send + Sync {
    fn extract(&self, code: &str, path: &Path) -> Result<Vec<Signature>>;
}

pub struct Signature {
    pub name: String,
    pub kind: String,      // "function", "class", "interface", etc.
    pub line: u32,
    pub doc: Option<String>,
}
```

## Adicionar Linguagem

### 1. Criar `src/extractors/<lang>.rs`

```rust
use tree_sitter::{Language, Parser};
use anyhow::{Result, Context};

pub struct GoLangExtractor;

impl Extractor for GoLangExtractor {
    fn extract(&self, code: &str, path: &Path) -> Result<Vec<Signature>> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_go::language())
            .context("Failed to set Go language")?;
        
        let tree = parser.parse(code, None)
            .ok_or(anyhow::anyhow!("Failed to parse code"))?;
        
        let mut sigs = Vec::new();
        visit_node(tree.root_node(), code, &mut sigs);
        Ok(sigs)
    }
}

fn visit_node(node: Node, code: &str, sigs: &mut Vec<Signature>) {
    match node.kind() {
        "function_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                sigs.push(Signature {
                    name,
                    kind: "function".to_string(),
                    line: node.start_point().row as u32 + 1,
                    doc: None,
                });
            }
        }
        "type_declaration" => {
            // Handle type definitions
        }
        _ => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    visit_node(child, code, sigs);
                }
            }
        }
    }
}
```

### 2. Registrar em `src/extractors/mod.rs`

```rust
fn extractor_for_ext(ext: &str) -> Option<Box<dyn Extractor>> {
    match ext {
        "go" => Some(Box::new(GoLangExtractor)),
        "ts" | "tsx" => Some(Box::new(TypeScriptExtractor)),
        "py" => Some(Box::new(PythonExtractor)),
        "rb" => Some(Box::new(RubyExtractor)),
        "groovy" => Some(Box::new(GroovyExtractor)),
        _ => None,
    }
}
```

### 3. Testar

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_function_extraction() {
        let code = r#"
        func Add(a, b int) int {
            return a + b
        }
        "#;
        
        let extractor = GoLangExtractor;
        let sigs = extractor.extract(code, Path::new("test.go")).unwrap();
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "Add");
        assert_eq!(sigs[0].kind, "function");
    }
}
```

Executar:

```bash
cargo test extractors::go::tests::test_go_function_extraction
```

## Padrões

### Tree-Sitter

- **Parser**: instanciar 1x e reutilizar
- **Language**: `tree_sitter_<lang>::language()`
- **Nodes**: `node.kind()`, `node.child_by_field_name()`, `node.utf8_text()`

### Documentação

Extrair docs (comentários acima de definições):

```rust
fn extract_doc(node: &Node, code: &str) -> Option<String> {
    if let Some(prev) = node.prev_sibling() {
        if prev.kind() == "comment" {
            return Some(prev.utf8_text(code.as_bytes()).unwrap_or("").to_string());
        }
    }
    None
}
```

### Error Handling

Sempre retornar `Result<Vec<Signature>>`:

```rust
impl Extractor for MyExtractor {
    fn extract(&self, code: &str, path: &Path) -> Result<Vec<Signature>> {
        let mut parser = Parser::new();
        parser.set_language(my_language())
            .with_context(|| format!("Failed to set language for {}", path.display()))?;
        
        let tree = parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Parse error in {}", path.display()))?;
        
        Ok(extract_signatures(tree.root_node(), code))
    }
}
```

## Performance

### Caching

Cache de arquivos já parseados — **não re-extrair se arquivo não mudou**:

- SHA256 do arquivo é chave do cache
- Se SHA256 == cached, usar resultado anterior
- Implementado em `src/cache.rs`

### Paralelismo

`src/scanner.rs` usa `rayon` para paralelismo:

```rust
files.par_iter()
    .map(|f| {
        let extractor = extractor_for_ext(&f.ext)?;
        extractor.extract(&f.content, &f.path)
    })
    .collect()
```

## Linguagens Suportadas

| Linguagem | Arquivo | Parser | Status |
|-----------|---------|--------|--------|
| TypeScript | `typescript.rs` | `tree-sitter-typescript` | ✅ |
| Python | `python.rs` | `tree-sitter-python` | ✅ |
| Ruby | `ruby.rs` | `tree-sitter-ruby` | ✅ |
| Groovy | `groovy.rs` | Customizada em `grammars/` | ✅ |
| Go | — | Pendente | ❌ |
| Rust | — | Pendente | ❌ |

---

**Última atualização**: 2026-04-14
