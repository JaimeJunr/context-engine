// Handlers para o subcomando interno `ctx __hook <name>`.
//
// Lê JSON do stdin (formato Claude Code), decide se o comando Bash está coberto
// pelo registry de `exec`, e devolve JSON ao stdout. Em qualquer erro inesperado
// o handler degrada para passthrough silencioso (`{}`) para não quebrar a
// sessão do agente.

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{Read, Write};

use crate::pipelines::exec::registry;

/// Despacha o hook nomeado. Sempre retorna `Ok(())`; falhas internas viram
/// passthrough (`{}`) com log em stderr.
pub fn dispatch(name: &str) -> Result<()> {
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);

    let output = match name {
        "claude-code-pre-tool-use" => claude_code_pre_tool_use(&input),
        other => {
            eprintln!("ctx __hook: handler desconhecido '{}'", other);
            json!({})
        }
    };

    let serialized = serde_json::to_string(&output).unwrap_or_else(|_| "{}".to_string());
    let _ = writeln!(std::io::stdout(), "{}", serialized);
    Ok(())
}

/// Decide se o comando Bash recebido deve ser redirecionado para `ctx exec`.
/// Retorna `{}` quando nada deve mudar (passthrough).
fn claude_code_pre_tool_use(stdin_json: &str) -> Value {
    let parsed: Value = match serde_json::from_str(stdin_json) {
        Ok(v) => v,
        Err(_) => return json!({}),
    };

    // Só nos interessamos por Bash.
    let tool_name = parsed.get("tool_name").and_then(|v| v.as_str());
    if tool_name != Some("Bash") {
        return json!({});
    }

    let command = match parsed
        .get("tool_input")
        .and_then(|t| t.get("command"))
        .and_then(|c| c.as_str())
    {
        Some(c) if !c.trim().is_empty() => c,
        _ => return json!({}),
    };

    // Não redirecionamos comandos que já são `ctx exec ...` — evita loop infinito.
    if is_already_ctx_exec(command) {
        return json!({});
    }

    // Parse robusto respeitando aspas/escapes.
    let tokens = match shell_words::split(command) {
        Ok(t) if !t.is_empty() => t,
        _ => return json!({}),
    };

    let (cmd, args) = split_command(&tokens);
    if !registry::matches(cmd, &args) {
        return json!({});
    }

    let rewritten = format!("ctx exec {}", command);
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "modifiedToolInput": { "command": rewritten }
        }
    })
}

/// `ctx exec foo bar` ou `/abs/path/ctx exec foo` — não reescrever.
fn is_already_ctx_exec(command: &str) -> bool {
    let trimmed = command.trim_start();
    let first = trimmed.split_whitespace().next().unwrap_or("");
    let basename = std::path::Path::new(first)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(first);
    if basename != "ctx" {
        return false;
    }
    let rest: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
    matches!(rest.first(), Some(&"exec") | Some(&"__hook"))
}

fn split_command(tokens: &[String]) -> (&str, Vec<String>) {
    let cmd = tokens[0].as_str();
    let args = tokens[1..].to_vec();
    (cmd, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str) -> Value {
        claude_code_pre_tool_use(input)
    }

    #[test]
    fn comando_coberto_e_redirecionado() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#;
        let out = run(input);
        assert_eq!(
            out["hookSpecificOutput"]["modifiedToolInput"]["command"],
            "ctx exec git status"
        );
    }

    #[test]
    fn comando_nao_coberto_e_passthrough() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"echo hello"}}"#;
        let out = run(input);
        assert_eq!(out, json!({}), "echo não tem filtro");
    }

    #[test]
    fn ctx_exec_nao_eh_reescrito() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"ctx exec git status"}}"#;
        let out = run(input);
        assert_eq!(out, json!({}), "evita loop infinito");
    }

    #[test]
    fn ctx_hook_nao_eh_reescrito() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"ctx __hook foo"}}"#;
        let out = run(input);
        assert_eq!(out, json!({}));
    }

    #[test]
    fn input_malformado_e_passthrough_silencioso() {
        assert_eq!(run("não é json"), json!({}));
        assert_eq!(run(""), json!({}));
        assert_eq!(run("{}"), json!({}));
    }

    #[test]
    fn tool_diferente_de_bash_e_ignorado() {
        let input = r#"{"tool_name":"Edit","tool_input":{"command":"git status"}}"#;
        assert_eq!(run(input), json!({}));
    }

    #[test]
    fn comando_com_aspas_e_split_corretamente() {
        // `git commit -m "foo bar"` deve resolver para subcomando específico de git.
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"git commit -m \"foo bar\""}}"#;
        let out = run(input);
        // git é coberto (genérico), então deve reescrever
        assert_eq!(
            out["hookSpecificOutput"]["modifiedToolInput"]["command"],
            r#"ctx exec git commit -m "foo bar""#
        );
    }

    #[test]
    fn comando_vazio_e_passthrough() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":""}}"#;
        assert_eq!(run(input), json!({}));
    }

    #[test]
    fn cargo_test_e_reescrito() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"cargo test --lib"}}"#;
        let out = run(input);
        assert_eq!(
            out["hookSpecificOutput"]["modifiedToolInput"]["command"],
            "ctx exec cargo test --lib"
        );
    }

    #[test]
    fn ctx_exec_com_path_absoluto_nao_reescreve() {
        let input =
            r#"{"tool_name":"Bash","tool_input":{"command":"/usr/local/bin/ctx exec git log"}}"#;
        assert_eq!(run(input), json!({}));
    }
}
