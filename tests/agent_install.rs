/// Testes de integração para `ctx install`/`ctx uninstall` e o handler `__hook`.
///
/// Tratam o binário como caixa-preta: invocam via processo, isolam o HOME
/// em um diretório temporário e validam o estado do settings.json gerado.
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ctx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("ctx")
}

/// Roda `ctx <args>` com HOME apontando para um tempdir.
fn run_with_home(home: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(ctx_bin())
        .env("HOME", home)
        .args(args)
        .output()
        .expect("falha ao executar ctx")
}

fn read_settings(path: &std::path::Path) -> Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("settings.json não existe: {}", path.display()));
    serde_json::from_str(&raw).expect("settings.json não é JSON válido")
}

// =========================================================================
// Ciclo install → hook → uninstall
// =========================================================================

#[test]
fn install_cria_settings_json_com_hook_esperado() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    std::fs::create_dir_all(home.join(".claude")).unwrap();

    let out = run_with_home(home, &["install", "--agent", "claude-code"]);
    assert!(
        out.status.success(),
        "install falhou: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let settings_path = home.join(".claude").join("settings.json");
    assert!(
        settings_path.exists(),
        "settings.json deve existir após install"
    );

    let s = read_settings(&settings_path);
    let hook = &s["hooks"]["PreToolUse"][0]["hooks"][0];
    assert_eq!(hook["command"], "ctx __hook claude-code-pre-tool-use");
    assert_eq!(hook["type"], "command");
    assert_eq!(hook["_installer"], "ctx");
    assert_eq!(s["hooks"]["PreToolUse"][0]["matcher"], "Bash");

    // MCP server também deve estar registrado
    let mcp = &s["mcpServers"]["ctx"];
    assert_eq!(mcp["command"], "ctx");
    assert_eq!(mcp["args"], serde_json::json!(["mcp", "serve"]));
    assert_eq!(mcp["_installer"], "ctx");
}

#[test]
fn install_preserva_mcp_server_alheio() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let claude = home.join(".claude");
    std::fs::create_dir_all(&claude).unwrap();
    let settings_path = claude.join("settings.json");

    // Usuário já tem outro MCP server registrado.
    let pre = json!({
        "mcpServers": {
            "outro": { "command": "/usr/bin/outro", "args": [] }
        }
    });
    std::fs::write(&settings_path, serde_json::to_string_pretty(&pre).unwrap()).unwrap();

    run_with_home(home, &["install", "--agent", "claude-code"]);
    run_with_home(home, &["uninstall", "--agent", "claude-code"]);

    let s = read_settings(&settings_path);
    let servers = s["mcpServers"].as_object().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers["outro"]["command"], "/usr/bin/outro");
}

#[test]
fn install_duplicado_nao_duplica_hook() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    std::fs::create_dir_all(home.join(".claude")).unwrap();

    run_with_home(home, &["install", "--agent", "claude-code"]);
    run_with_home(home, &["install", "--agent", "claude-code"]);

    let s = read_settings(&home.join(".claude").join("settings.json"));
    let inner = s["hooks"]["PreToolUse"][0]["hooks"].as_array().unwrap();
    assert_eq!(inner.len(), 1, "install duas vezes não duplica");
}

#[test]
fn uninstall_remove_arquivo_quando_vira_vazio() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    std::fs::create_dir_all(home.join(".claude")).unwrap();

    run_with_home(home, &["install", "--agent", "claude-code"]);
    let path = home.join(".claude").join("settings.json");
    assert!(path.exists());

    run_with_home(home, &["uninstall", "--agent", "claude-code"]);
    assert!(
        !path.exists(),
        "arquivo deve ser removido quando vira vazio após uninstall"
    );
}

#[test]
fn uninstall_sem_install_previo_e_no_op() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    std::fs::create_dir_all(home.join(".claude")).unwrap();

    let out = run_with_home(home, &["uninstall", "--agent", "claude-code"]);
    assert!(
        out.status.success(),
        "uninstall sem install não deve falhar"
    );
}

#[test]
fn uninstall_preserva_hooks_alheios() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let claude = home.join(".claude");
    std::fs::create_dir_all(&claude).unwrap();
    let settings_path = claude.join("settings.json");

    // Estado pré-existente: usuário já tem hook próprio em Bash.
    let pre = json!({
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        { "type": "command", "command": "/users/own.sh" }
                    ]
                }
            ]
        }
    });
    std::fs::write(&settings_path, serde_json::to_string_pretty(&pre).unwrap()).unwrap();

    run_with_home(home, &["install", "--agent", "claude-code"]);
    run_with_home(home, &["uninstall", "--agent", "claude-code"]);

    let s = read_settings(&settings_path);
    let inner = s["hooks"]["PreToolUse"][0]["hooks"].as_array().unwrap();
    assert_eq!(inner.len(), 1);
    assert_eq!(inner[0]["command"], "/users/own.sh");
}

// =========================================================================
// Handler `__hook` — testes de comportamento via stdin/stdout
// =========================================================================

fn pipe_hook(stdin_payload: &str) -> Value {
    let mut child = Command::new(ctx_bin())
        .args(["__hook", "claude-code-pre-tool-use"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ctx __hook");

    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin_payload.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait_with_output");
    assert!(out.status.success(), "__hook deve sempre exitar 0");
    serde_json::from_slice(&out.stdout).expect("output deve ser JSON")
}

#[test]
fn hook_redireciona_git_status_para_ctx_exec() {
    let out = pipe_hook(r#"{"tool_name":"Bash","tool_input":{"command":"git status"}}"#);
    assert_eq!(
        out["hookSpecificOutput"]["modifiedToolInput"]["command"],
        "ctx exec git status"
    );
    assert_eq!(out["hookSpecificOutput"]["hookEventName"], "PreToolUse");
}

#[test]
fn hook_ignora_comando_sem_filtro() {
    let out = pipe_hook(r#"{"tool_name":"Bash","tool_input":{"command":"echo oi"}}"#);
    assert_eq!(out, json!({}), "echo não tem filtro registrado");
}

#[test]
fn hook_nao_reescreve_ctx_exec_para_evitar_loop() {
    let out = pipe_hook(r#"{"tool_name":"Bash","tool_input":{"command":"ctx exec git status"}}"#);
    assert_eq!(out, json!({}));
}

#[test]
fn hook_aceita_input_malformado_sem_falhar() {
    let out = pipe_hook("não é json válido");
    assert_eq!(out, json!({}));
}

#[test]
fn hook_ignora_tool_diferente_de_bash() {
    let out = pipe_hook(r#"{"tool_name":"Edit","tool_input":{"command":"git status"}}"#);
    assert_eq!(out, json!({}));
}

// =========================================================================
// Detecção de Claude Code ausente
// =========================================================================

#[test]
fn install_sem_diretorio_claude_em_user_scope_falha_com_mensagem() {
    let tmp = tempfile::tempdir().unwrap();
    // HOME existe mas ~/.claude/ NÃO existe
    let home = tmp.path();

    let out = run_with_home(home, &["install", "--agent", "claude-code"]);
    assert!(
        !out.status.success(),
        "deve falhar quando Claude Code não está instalado"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Claude Code") || stderr.contains(".claude"),
        "mensagem deve apontar o problema: {}",
        stderr
    );
}
