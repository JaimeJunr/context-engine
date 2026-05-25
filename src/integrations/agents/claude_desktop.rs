// Instalação de MCP server no Claude Desktop (app de chat da Anthropic).
//
// Diferente do Claude Code:
//   - Desktop NÃO suporta hooks PreToolUse — só MCP servers.
//   - Path muda por SO via XDG/Apple/Windows:
//     · Linux:   ~/.config/Claude/claude_desktop_config.json
//     · macOS:   ~/Library/Application Support/Claude/claude_desktop_config.json
//     · Windows: %APPDATA%/Claude/claude_desktop_config.json
//   - Não tem escopo de projeto — só user-level faz sentido.
//
// Reusa `settings_merge::{insert_mcp_server, remove_ctx_mcp_servers}` (mesmo schema).

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::path::PathBuf;

use super::{
    settings_merge::{insert_mcp_server, remove_ctx_mcp_servers},
    AgentInstaller, InstallReport, Scope, UninstallReport,
};

const MCP_SERVER_NAME: &str = "ctx";

pub struct ClaudeDesktopInstaller;

impl AgentInstaller for ClaudeDesktopInstaller {
    fn name(&self) -> &'static str {
        "Claude Desktop"
    }

    fn install(&self, scope: Scope, _force: bool) -> Result<InstallReport> {
        // Desktop não tem escopo de projeto — Project é tratado como User
        // com aviso silencioso (operação semântica é igual).
        let _ = scope;

        let settings_path = resolve_settings_path()?;

        // Garante o diretório pai (Claude Desktop instalado mas nunca aberto
        // pode ainda não ter criado o config).
        if let Some(dir) = settings_path.parent() {
            if !dir.exists() {
                return Err(anyhow!(
                    "diretório {} não existe — Claude Desktop parece não estar instalado.\nAbra o app pelo menos uma vez para criar a pasta de configuração.",
                    dir.display()
                ));
            }
        }

        let mut settings = read_settings(&settings_path)?;
        let was_present = has_ctx_mcp_server(&settings);

        insert_mcp_server(&mut settings, MCP_SERVER_NAME, "ctx", &["mcp", "serve"]);
        write_settings(&settings_path, &settings)?;

        Ok(InstallReport {
            settings_path,
            already_installed: was_present,
        })
    }

    fn uninstall(&self, scope: Scope) -> Result<UninstallReport> {
        let _ = scope;
        let settings_path = resolve_settings_path()?;

        if !settings_path.exists() {
            return Ok(UninstallReport {
                settings_path,
                removed: false,
            });
        }

        let mut settings = read_settings(&settings_path)?;
        let was_present = has_ctx_mcp_server(&settings);

        remove_ctx_mcp_servers(&mut settings);

        // Diferente do Claude Code: preservamos o arquivo mesmo se ficar com
        // apenas `preferences` (usuário típico do Desktop tem muita config lá).
        write_settings(&settings_path, &settings)?;

        Ok(UninstallReport {
            settings_path,
            removed: was_present,
        })
    }
}

/// Resolve o caminho do `claude_desktop_config.json` por SO.
fn resolve_settings_path() -> Result<PathBuf> {
    // Em macOS e Windows, `dirs::config_dir()` retorna exatamente o caminho
    // que o Claude Desktop usa (Application Support / AppData Roaming).
    // No Linux retorna `~/.config`.
    let base = dirs::config_dir()
        .ok_or_else(|| anyhow!("não foi possível determinar diretório de config do SO"))?;
    Ok(base.join("Claude").join("claude_desktop_config.json"))
}

/// Lê config existente ou retorna `{}` se não existir.
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

/// Escreve config criando diretórios pai conforme necessário.
fn write_settings(path: &PathBuf, settings: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("criando diretório {}", parent.display()))?;
    }
    let pretty = serde_json::to_string_pretty(settings).context("serializando config")?;
    std::fs::write(path, format!("{}\n", pretty))
        .with_context(|| format!("escrevendo {}", path.display()))?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: simula install em diretório isolado (sem mexer em HOME real).
    fn install_into(tmp: &TempDir) -> PathBuf {
        let dir = tmp.path().join("Claude");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("claude_desktop_config.json");

        let mut s = read_settings(&path).unwrap();
        insert_mcp_server(&mut s, MCP_SERVER_NAME, "ctx", &["mcp", "serve"]);
        write_settings(&path, &s).unwrap();
        path
    }

    #[test]
    fn install_cria_arquivo_com_schema_esperado() {
        let tmp = TempDir::new().unwrap();
        let path = install_into(&tmp);

        let s: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let entry = &s["mcpServers"]["ctx"];
        assert_eq!(entry["command"], "ctx");
        assert_eq!(entry["args"], serde_json::json!(["mcp", "serve"]));
        assert_eq!(entry["_installer"], "ctx");
    }

    #[test]
    fn install_preserva_preferences_do_usuario() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("Claude");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("claude_desktop_config.json");

        // Simula config existente do usuário com preferences complexas.
        let pre = serde_json::json!({
            "preferences": {
                "dockBounceEnabled": true,
                "sidebarMode": "epitaxy"
            }
        });
        write_settings(&path, &pre).unwrap();

        let mut s = read_settings(&path).unwrap();
        insert_mcp_server(&mut s, MCP_SERVER_NAME, "ctx", &["mcp", "serve"]);
        write_settings(&path, &s).unwrap();

        let final_state: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(final_state["preferences"]["dockBounceEnabled"], true);
        assert_eq!(final_state["preferences"]["sidebarMode"], "epitaxy");
        assert_eq!(final_state["mcpServers"]["ctx"]["command"], "ctx");
    }

    #[test]
    fn uninstall_preserva_mcp_servers_alheios() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("Claude");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("claude_desktop_config.json");

        let pre = serde_json::json!({
            "preferences": { "foo": "bar" },
            "mcpServers": {
                "other-tool": { "command": "other", "args": [] }
            }
        });
        write_settings(&path, &pre).unwrap();

        // Install + uninstall do ctx
        let mut s = read_settings(&path).unwrap();
        insert_mcp_server(&mut s, MCP_SERVER_NAME, "ctx", &["mcp", "serve"]);
        write_settings(&path, &s).unwrap();

        let mut s = read_settings(&path).unwrap();
        remove_ctx_mcp_servers(&mut s);
        write_settings(&path, &s).unwrap();

        let final_state: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        // other-tool intocado, preferences intacto, ctx removido
        assert_eq!(final_state["mcpServers"]["other-tool"]["command"], "other");
        assert!(final_state["mcpServers"].get("ctx").is_none());
        assert_eq!(final_state["preferences"]["foo"], "bar");
    }

    #[test]
    fn has_ctx_mcp_server_detecta_presenca() {
        let mut s = serde_json::json!({});
        assert!(!has_ctx_mcp_server(&s));
        insert_mcp_server(&mut s, MCP_SERVER_NAME, "ctx", &["mcp", "serve"]);
        assert!(has_ctx_mcp_server(&s));
        remove_ctx_mcp_servers(&mut s);
        assert!(!has_ctx_mcp_server(&s));
    }
}
