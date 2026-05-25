// NestJS route detector: lê decorators `@Controller`, `@Get`, `@Post`, `@Put`,
// `@Patch`, `@Delete`, `@All` em arquivos TypeScript.
//
// Heurística: o `@Controller('/users')` define um prefixo de classe; cada
// método dentro com `@Get(':id')` etc combina o prefixo + path do método.

use std::path::Path;

use super::{FrameworkRouter, RouteMapping};

pub struct NestJsRouter;

impl FrameworkRouter for NestJsRouter {
    fn detect(&self, path: &Path, content: &str) -> bool {
        let path_str = path.to_string_lossy();
        let is_ts = path_str.ends_with(".ts") || path_str.ends_with(".tsx");
        if !is_ts {
            return false;
        }
        // Sinal forte: import do @nestjs/common ou @Controller.
        content.contains("@nestjs/common") || content.contains("@Controller")
    }

    fn extract(&self, path: &Path, content: &str) -> Vec<RouteMapping> {
        let file = path.to_string_lossy().to_string();
        let mut mappings = Vec::new();

        // Estado: prefixo do controller atual + nome da classe.
        let mut current_prefix: Option<String> = None;
        let mut current_class: Option<String> = None;
        let mut pending_method_route: Option<(String, String, u32)> = None; // (verb, path, line)

        for (idx, line) in content.lines().enumerate() {
            let line_no = (idx + 1) as u32;
            let trimmed = line.trim();

            // @Controller('users')   ou   @Controller()
            if let Some(prefix) = parse_controller_decorator(trimmed) {
                current_prefix = Some(prefix);
                continue;
            }

            // export class UsersController { ... }
            if let Some(class_name) = parse_class_name(trimmed) {
                if current_prefix.is_some() {
                    current_class = Some(class_name);
                }
                continue;
            }

            // @Get(':id') / @Post() / @Patch('update') ...
            if let Some((verb, path)) = parse_verb_decorator(trimmed) {
                pending_method_route = Some((verb, path, line_no));
                continue;
            }

            // método: findOne(...) ou async findOne(...)
            if let Some(method_name) = parse_method_definition(trimmed) {
                if let (Some(prefix), Some(class_name), Some((verb, m_path, route_line))) =
                    (&current_prefix, &current_class, pending_method_route.take())
                {
                    let full_path = combine_paths(prefix, &m_path);
                    mappings.push(RouteMapping {
                        method: verb,
                        path: full_path,
                        handler: format!("{}::{}", class_name, method_name),
                        source_file: file.clone(),
                        source_line: route_line,
                    });
                }
            }
        }

        mappings
    }

    fn language(&self) -> &'static str {
        "typescript"
    }
}

fn parse_controller_decorator(line: &str) -> Option<String> {
    // @Controller('users') | @Controller("users") | @Controller() | @Controller({ ... })
    if !line.starts_with("@Controller") {
        return None;
    }
    // Extrai conteúdo entre parênteses
    let paren_start = line.find('(')?;
    let paren_end = line.rfind(')')?;
    let inside = &line[paren_start + 1..paren_end].trim();
    if inside.is_empty() {
        return Some(String::new());
    }
    // Pega primeira string entre aspas
    extract_first_string(inside).or(Some(String::new()))
}

fn parse_class_name(line: &str) -> Option<String> {
    // export class UsersController { ... } ou class Foo {
    let line = line.strip_prefix("export").unwrap_or(line).trim();
    let line = line.strip_prefix("class")?.trim();
    let name = line
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_verb_decorator(line: &str) -> Option<(String, String)> {
    for verb_dec in &[
        "@Get", "@Post", "@Put", "@Patch", "@Delete", "@All", "@Options", "@Head",
    ] {
        if let Some(rest) = line.strip_prefix(verb_dec) {
            // Garante palavra inteira (próximo char deve ser '(' ).
            if !rest.starts_with('(') {
                continue;
            }
            let paren_end = rest.rfind(')').unwrap_or(rest.len());
            let inside = &rest[1..paren_end].trim();
            let path = if inside.is_empty() {
                String::new()
            } else {
                extract_first_string(inside).unwrap_or_default()
            };
            // Verb sem o '@'
            let verb = verb_dec.trim_start_matches('@').to_uppercase();
            let verb = if verb == "ALL" {
                "ANY".to_string()
            } else {
                verb
            };
            return Some((verb, path));
        }
    }
    None
}

fn parse_method_definition(line: &str) -> Option<String> {
    // async findOne(...)   |   findOne(...)   |   public findOne(...)
    let line = line
        .strip_prefix("public")
        .or_else(|| line.strip_prefix("private"))
        .or_else(|| line.strip_prefix("protected"))
        .unwrap_or(line)
        .trim();
    let line = line.strip_prefix("async").unwrap_or(line).trim();
    // Métodos não começam com palavras-chave de outras coisas
    if line.starts_with("constructor") || line.starts_with("import") || line.starts_with("//") {
        return None;
    }
    // Pega o nome até o '('
    let paren = line.find('(')?;
    let name = line[..paren].trim();
    // Filtra: precisa ser identifier válido + algo sensato (não vazio, não keyword)
    if name.is_empty()
        || name.contains(' ')
        || name.contains('=')
        || name.contains('<')
        || matches!(name, "if" | "for" | "while" | "switch" | "return" | "throw")
    {
        return None;
    }
    Some(name.to_string())
}

fn extract_first_string(s: &str) -> Option<String> {
    let quote = if s.contains('\'') { '\'' } else { '"' };
    let start = s.find(quote)?;
    let after = &s[start + 1..];
    let end = after.find(quote)?;
    Some(after[..end].to_string())
}

fn combine_paths(prefix: &str, method_path: &str) -> String {
    let p = prefix.trim_matches('/');
    let m = method_path.trim_matches('/');
    match (p.is_empty(), m.is_empty()) {
        (true, true) => "/".to_string(),
        (true, false) => format!("/{}", m),
        (false, true) => format!("/{}", p),
        (false, false) => format!("/{}/{}", p, m),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn extract(content: &str) -> Vec<RouteMapping> {
        let router = NestJsRouter;
        router.extract(&PathBuf::from("src/users/users.controller.ts"), content)
    }

    #[test]
    fn detecta_get_e_post_basico() {
        let src = r#"
import { Controller, Get, Post } from '@nestjs/common';

@Controller('users')
export class UsersController {
  @Get()
  findAll() { return []; }

  @Get(':id')
  findOne(id: string) { return id; }

  @Post()
  create() { return {}; }
}
"#;
        let routes = extract(src);
        assert_eq!(routes.len(), 3, "esperava 3 routes, obteve {:?}", routes);

        let findall = routes
            .iter()
            .find(|r| r.handler.ends_with("::findAll"))
            .unwrap();
        assert_eq!(findall.method, "GET");
        assert_eq!(findall.path, "/users");

        let findone = routes
            .iter()
            .find(|r| r.handler.ends_with("::findOne"))
            .unwrap();
        assert_eq!(findone.path, "/users/:id");

        let create = routes
            .iter()
            .find(|r| r.handler.ends_with("::create"))
            .unwrap();
        assert_eq!(create.method, "POST");
    }

    #[test]
    fn controller_sem_prefixo_gera_rota_raiz() {
        let src = r#"
import { Controller, Get } from '@nestjs/common';
@Controller()
export class AppController {
  @Get()
  hello() { return 'hi'; }
}
"#;
        let routes = extract(src);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/");
    }

    #[test]
    fn detect_so_aceita_arquivos_com_decorator() {
        let r = NestJsRouter;
        assert!(r.detect(
            &PathBuf::from("a.ts"),
            "import { Controller } from '@nestjs/common';"
        ));
        assert!(!r.detect(&PathBuf::from("a.ts"), "function foo() {}"));
        assert!(!r.detect(&PathBuf::from("a.rb"), "@Controller"));
    }
}
