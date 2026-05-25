// Instalação de hooks PreToolUse no Claude Code.
//
// Estratégia: escreve um hook que invoca `ctx __hook claude-code-pre-tool-use`
// no settings.json apropriado (~/.claude/ ou ./.claude/). O hook lê stdin,
// e quando o comando Bash está coberto pelo nosso registry, devolve um
// `modifiedToolInput` redirecionando para `ctx exec`.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::path::PathBuf;

use super::{
    settings_merge::{
        insert_mcp_server, insert_pre_tool_use_bash_hook, remove_ctx_hooks, remove_ctx_mcp_servers,
    },
    AgentInstaller, InstallReport, Scope, UninstallReport,
};

/// Comando que o Claude Code invocará ao receber um Bash tool call.
const HOOK_CMD: &str = "ctx __hook claude-code-pre-tool-use";

/// Nome do MCP server registrado em settings.json.
const MCP_SERVER_NAME: &str = "ctx";

pub struct ClaudeCodeInstaller;

impl AgentInstaller for ClaudeCodeInstaller {
    fn name(&self) -> &'static str {
        "Claude Code"
    }

    fn install(&self, scope: Scope, _force: bool) -> Result<InstallReport> {
        let settings_path = resolve_settings_path(scope)?;

        // Para escopo de usuário, exigimos que o diretório base exista —
        // é o sinal de que o Claude Code está instalado.
        if matches!(scope, Scope::User) {
            let dir = settings_path.parent().ok_or_else(|| {
                anyhow!(
                    "caminho de settings sem diretório pai: {}",
                    settings_path.display()
                )
            })?;
            if !dir.exists() {
                return Err(anyhow!(
                    "diretório {} não existe — Claude Code parece não estar instalado.\nUse --project para instalar apenas no projeto atual.",
                    dir.display()
                ));
            }
        }

        let mut settings = read_settings(&settings_path)?;
        let was_present = has_ctx_hook(&settings) || has_ctx_mcp_server(&settings);

        // Registra hook PreToolUse para Bash + MCP server "ctx".
        insert_pre_tool_use_bash_hook(&mut settings, HOOK_CMD);
        insert_mcp_server(&mut settings, MCP_SERVER_NAME, "ctx", &["mcp", "serve"]);
        write_settings(&settings_path, &settings)?;

        Ok(InstallReport {
            settings_path,
            already_installed: was_present,
        })
    }

    fn uninstall(&self, scope: Scope) -> Result<UninstallReport> {
        let settings_path = resolve_settings_path(scope)?;

        if !settings_path.exists() {
            return Ok(UninstallReport {
                settings_path,
                removed: false,
            });
        }

        let mut settings = read_settings(&settings_path)?;
        let was_present = has_ctx_hook(&settings) || has_ctx_mcp_server(&settings);

        remove_ctx_hooks(&mut settings);
        remove_ctx_mcp_servers(&mut settings);

        // Se o arquivo virou `{}`, removemos para não deixar lixo.
        if is_empty_object(&settings) {
            std::fs::remove_file(&settings_path).with_context(|| {
                format!("removendo settings.json vazio: {}", settings_path.display())
            })?;
        } else {
            write_settings(&settings_path, &settings)?;
        }

        Ok(UninstallReport {
            settings_path,
            removed: was_present,
        })
    }
}

/// Resolve o caminho do settings.json para o escopo escolhido.
fn resolve_settings_path(scope: Scope) -> Result<PathBuf> {
    match scope {
        Scope::User => {
            let home =
                dirs::home_dir().ok_or_else(|| anyhow!("não foi possível determinar HOME"))?;
            Ok(home.join(".claude").join("settings.json"))
        }
        Scope::Project => {
            let cwd = std::env::current_dir().context("lendo diretório atual")?;
            Ok(cwd.join(".claude").join("settings.json"))
        }
    }
}

/// Lê settings.json existente ou retorna `{}` se não existir.
fn read_settings(path: &PathBuf) -> Result<Value> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("lendo {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&raw).with_context(|| format!("parseando {} como JSON", path.display()))
}

/// Escreve settings.json criando diretórios pai conforme necessário.
fn write_settings(path: &PathBuf, settings: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("criando diretório {}", parent.display()))?;
    }
    let pretty = serde_json::to_string_pretty(settings).context("serializando settings.json")?;
    std::fs::write(path, format!("{}\n", pretty))
        .with_context(|| format!("escrevendo {}", path.display()))?;
    Ok(())
}

fn has_ctx_hook(settings: &Value) -> bool {
    settings
        .get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|inner| {
                        inner.iter().any(|h| {
                            h.get(super::settings_merge::INSTALLER_MARK)
                                .and_then(|v| v.as_str())
                                == Some(super::settings_merge::INSTALLER_VALUE)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn has_ctx_mcp_server(settings: &Value) -> bool {
    settings
        .get("mcpServers")
        .and_then(|s| s.as_object())
        .map(|obj| {
            obj.values().any(|v| {
                v.get(super::settings_merge::INSTALLER_MARK)
                    .and_then(|x| x.as_str())
                    == Some(super::settings_merge::INSTALLER_VALUE)
            })
        })
        .unwrap_or(false)
}

fn is_empty_object(v: &Value) -> bool {
    v.as_object().map(|o| o.is_empty()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Instala apontando para um diretório isolado, ignorando o HOME real.
    fn install_into(tmp: &TempDir) -> PathBuf {
        let claude_dir = tmp.path().join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings = claude_dir.join("settings.json");

        // Caminho direto para testar a lógica sem mexer em HOME.
        let mut s = read_settings(&settings).unwrap();
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);
        write_settings(&settings, &s).unwrap();
        settings
    }

    #[test]
    fn install_cria_arquivo_com_schema_esperado() {
        let tmp = TempDir::new().unwrap();
        let path = install_into(&tmp);

        let raw = std::fs::read_to_string(&path).unwrap();
        let s: Value = serde_json::from_str(&raw).unwrap();
        let hook = &s["hooks"]["PreToolUse"][0]["hooks"][0];
        assert_eq!(hook["command"], HOOK_CMD);
        assert_eq!(hook["type"], "command");
    }

    #[test]
    fn uninstall_remove_arquivo_quando_so_havia_nosso_hook() {
        let tmp = TempDir::new().unwrap();
        let path = install_into(&tmp);
        assert!(path.exists());

        let mut s = read_settings(&path).unwrap();
        remove_ctx_hooks(&mut s);
        if is_empty_object(&s) {
            std::fs::remove_file(&path).unwrap();
        }
        assert!(
            !path.exists(),
            "arquivo deve ser removido quando fica vazio"
        );
    }

    #[test]
    fn uninstall_preserva_hooks_alheios() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        // Estado inicial: usuário tem um hook próprio.
        let pre_existing = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [{ "type": "command", "command": "/users/own-hook.sh" }]
                    }
                ]
            }
        });
        write_settings(&path, &pre_existing).unwrap();

        // Install + uninstall
        let mut s = read_settings(&path).unwrap();
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);
        write_settings(&path, &s).unwrap();

        let mut s = read_settings(&path).unwrap();
        remove_ctx_hooks(&mut s);
        write_settings(&path, &s).unwrap();

        // Verifica que o hook do usuário permanece intacto.
        let final_state: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let inner = final_state["hooks"]["PreToolUse"][0]["hooks"]
            .as_array()
            .unwrap();
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0]["command"], "/users/own-hook.sh");
    }

    #[test]
    fn read_settings_inexistente_retorna_objeto_vazio() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("naoexiste.json");
        let s = read_settings(&path).unwrap();
        assert_eq!(s, serde_json::json!({}));
    }

    #[test]
    fn has_ctx_hook_detecta_presenca_correta() {
        let mut s = serde_json::json!({});
        assert!(!has_ctx_hook(&s));
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);
        assert!(has_ctx_hook(&s));
        remove_ctx_hooks(&mut s);
        assert!(!has_ctx_hook(&s));
    }
}
