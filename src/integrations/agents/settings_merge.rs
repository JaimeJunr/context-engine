// Helpers puros para mesclar e remover hooks em settings.json de agentes.
// Não realiza I/O — operam em serde_json::Value para facilitar testes.

use serde_json::{json, Value};

/// Marcador que identifica entradas instaladas pelo ctx.
pub const INSTALLER_MARK: &str = "_installer";
pub const INSTALLER_VALUE: &str = "ctx";

/// Adiciona um hook PreToolUse para o matcher "Bash" se ainda não estiver presente.
/// Operação idempotente: chamar várias vezes não duplica.
///
/// `hook_cmd` é o comando que o Claude Code irá invocar (ex: "ctx __hook claude-code-pre-tool-use").
pub fn insert_pre_tool_use_bash_hook(settings: &mut Value, hook_cmd: &str) {
    if !settings.is_object() {
        *settings = json!({});
    }
    let obj = settings.as_object_mut().expect("settings é object");

    let hooks = obj.entry("hooks").or_insert_with(|| json!({}));
    if !hooks.is_object() {
        *hooks = json!({});
    }
    let hooks_obj = hooks.as_object_mut().expect("hooks é object");

    let pre_arr = hooks_obj.entry("PreToolUse").or_insert_with(|| json!([]));
    if !pre_arr.is_array() {
        *pre_arr = json!([]);
    }
    let pre = pre_arr.as_array_mut().expect("PreToolUse é array");

    // Procura entrada com matcher == "Bash"
    let bash_entry_idx = pre
        .iter()
        .position(|entry| entry.get("matcher").and_then(|m| m.as_str()) == Some("Bash"));

    let entry = match bash_entry_idx {
        Some(idx) => &mut pre[idx],
        None => {
            pre.push(json!({ "matcher": "Bash", "hooks": [] }));
            pre.last_mut().expect("recém-inserido")
        }
    };

    let entry_obj = entry.as_object_mut().expect("entrada é object");
    let inner_arr = entry_obj.entry("hooks").or_insert_with(|| json!([]));
    if !inner_arr.is_array() {
        *inner_arr = json!([]);
    }
    let inner = inner_arr.as_array_mut().expect("hooks interno é array");

    // Já existe hook nosso? Se sim, no-op.
    let already_present = inner.iter().any(is_ctx_hook);
    if already_present {
        return;
    }

    inner.push(json!({
        "type": "command",
        "command": hook_cmd,
        INSTALLER_MARK: INSTALLER_VALUE,
    }));
}

/// Remove apenas os hooks marcados com `_installer == "ctx"`.
/// Preserva hooks de outras ferramentas e configurações do usuário.
/// Limpa chaves vazias resultantes (entrada Bash sem hooks, PreToolUse vazio, etc).
pub fn remove_ctx_hooks(settings: &mut Value) {
    let Some(obj) = settings.as_object_mut() else {
        return;
    };
    let Some(hooks) = obj.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return;
    };
    let Some(pre) = hooks.get_mut("PreToolUse").and_then(|v| v.as_array_mut()) else {
        return;
    };

    // Remove hooks marcados em cada entrada, depois remove entradas que ficaram sem hooks.
    pre.iter_mut().for_each(|entry| {
        if let Some(inner) = entry.get_mut("hooks").and_then(|v| v.as_array_mut()) {
            inner.retain(|h| !is_ctx_hook(h));
        }
    });
    pre.retain(|entry| {
        entry
            .get("hooks")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    });

    // Limpa chaves vazias para deixar o arquivo mínimo.
    if pre.is_empty() {
        hooks.remove("PreToolUse");
    }
    if hooks.is_empty() {
        obj.remove("hooks");
    }
}

/// Indica se o hook foi instalado pelo ctx.
fn is_ctx_hook(h: &Value) -> bool {
    h.get(INSTALLER_MARK).and_then(|v| v.as_str()) == Some(INSTALLER_VALUE)
}

/// Registra um MCP server em `mcpServers.<name>`. Operação idempotente.
///
/// Marca a entrada com `_installer: "ctx"` para uninstall seletivo.
pub fn insert_mcp_server(settings: &mut Value, name: &str, command: &str, args: &[&str]) {
    if !settings.is_object() {
        *settings = json!({});
    }
    let obj = settings.as_object_mut().expect("settings é object");
    let servers = obj.entry("mcpServers").or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    let servers_obj = servers.as_object_mut().expect("mcpServers é object");

    // Se já existe entrada com o mesmo nome marcada como ctx, no-op.
    if let Some(existing) = servers_obj.get(name) {
        if is_ctx_hook(existing) {
            return;
        }
    }

    servers_obj.insert(
        name.to_string(),
        json!({
            "command": command,
            "args": args,
            INSTALLER_MARK: INSTALLER_VALUE,
        }),
    );
}

/// Remove apenas entradas de `mcpServers` marcadas com `_installer == "ctx"`.
/// Limpa a chave `mcpServers` se ficar vazia.
pub fn remove_ctx_mcp_servers(settings: &mut Value) {
    let Some(obj) = settings.as_object_mut() else {
        return;
    };
    let Some(servers) = obj.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
        return;
    };

    let keys_to_remove: Vec<String> = servers
        .iter()
        .filter(|(_, v)| is_ctx_hook(v))
        .map(|(k, _)| k.clone())
        .collect();
    for k in keys_to_remove {
        servers.remove(&k);
    }

    if servers.is_empty() {
        obj.remove("mcpServers");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const HOOK_CMD: &str = "ctx __hook claude-code-pre-tool-use";

    #[test]
    fn insercao_em_settings_vazio_cria_estrutura_completa() {
        let mut s = json!({});
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);

        let hook = &s["hooks"]["PreToolUse"][0]["hooks"][0];
        assert_eq!(hook["command"], HOOK_CMD);
        assert_eq!(hook[INSTALLER_MARK], INSTALLER_VALUE);
        assert_eq!(s["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    }

    #[test]
    fn insercao_duplicada_e_no_op() {
        let mut s = json!({});
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);

        let inner = s["hooks"]["PreToolUse"][0]["hooks"].as_array().unwrap();
        assert_eq!(inner.len(), 1, "não deve duplicar hooks do ctx");
    }

    #[test]
    fn insercao_preserva_hooks_alheios_no_mesmo_matcher() {
        let mut s = json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            { "type": "command", "command": "/path/to/other-hook.sh" }
                        ]
                    }
                ]
            }
        });
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);

        let inner = s["hooks"]["PreToolUse"][0]["hooks"].as_array().unwrap();
        assert_eq!(
            inner.len(),
            2,
            "deve manter o hook do usuário e adicionar o nosso"
        );
        assert!(inner
            .iter()
            .any(|h| h["command"] == "/path/to/other-hook.sh"));
        assert!(inner.iter().any(|h| h["command"] == HOOK_CMD));
    }

    #[test]
    fn insercao_preserva_outros_matchers() {
        let mut s = json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Edit",
                        "hooks": [{ "type": "command", "command": "format.sh" }]
                    }
                ]
            }
        });
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);

        let arr = s["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 2, "Edit e Bash devem coexistir");
    }

    #[test]
    fn remocao_em_settings_vazio_e_no_op() {
        let mut s = json!({});
        remove_ctx_hooks(&mut s);
        assert_eq!(s, json!({}));
    }

    #[test]
    fn remocao_tira_hook_do_ctx_e_preserva_outros() {
        let mut s = json!({});
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);
        // Simula hook alheio adicionado depois
        s["hooks"]["PreToolUse"][0]["hooks"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "type": "command", "command": "other.sh" }));

        remove_ctx_hooks(&mut s);

        let inner = s["hooks"]["PreToolUse"][0]["hooks"].as_array().unwrap();
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0]["command"], "other.sh");
    }

    #[test]
    fn remocao_limpa_chaves_vazias_quando_unico_hook_era_nosso() {
        let mut s = json!({});
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);

        remove_ctx_hooks(&mut s);

        assert_eq!(s, json!({}), "arquivo deve ficar mínimo após uninstall");
    }

    #[test]
    fn remocao_nao_toca_em_hooks_de_outros_matchers() {
        let mut s = json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Edit",
                        "hooks": [{ "type": "command", "command": "format.sh" }]
                    }
                ]
            }
        });
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);

        remove_ctx_hooks(&mut s);

        let arr = s["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["matcher"], "Edit");
    }

    #[test]
    fn remocao_idempotente() {
        let mut s = json!({});
        insert_pre_tool_use_bash_hook(&mut s, HOOK_CMD);
        remove_ctx_hooks(&mut s);
        let after_first = s.clone();
        remove_ctx_hooks(&mut s);
        assert_eq!(s, after_first, "segunda remoção é no-op");
    }

    // =====================================================================
    // MCP servers
    // =====================================================================

    #[test]
    fn mcp_insert_cria_entrada_marcada() {
        let mut s = json!({});
        insert_mcp_server(&mut s, "ctx", "ctx", &["mcp", "serve"]);

        let entry = &s["mcpServers"]["ctx"];
        assert_eq!(entry["command"], "ctx");
        assert_eq!(entry["args"], json!(["mcp", "serve"]));
        assert_eq!(entry[INSTALLER_MARK], INSTALLER_VALUE);
    }

    #[test]
    fn mcp_insert_duplicado_e_no_op() {
        let mut s = json!({});
        insert_mcp_server(&mut s, "ctx", "ctx", &["mcp", "serve"]);
        let snapshot = s.clone();
        insert_mcp_server(&mut s, "ctx", "ctx", &["mcp", "serve"]);
        assert_eq!(s, snapshot, "segunda inserção é no-op");
    }

    #[test]
    fn mcp_insert_preserva_servers_alheios() {
        let mut s = json!({
            "mcpServers": {
                "outro": { "command": "/usr/bin/outro", "args": [] }
            }
        });
        insert_mcp_server(&mut s, "ctx", "ctx", &["mcp", "serve"]);

        let servers = s["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers.contains_key("outro"));
        assert!(servers.contains_key("ctx"));
    }

    #[test]
    fn mcp_remove_tira_apenas_entradas_marcadas() {
        let mut s = json!({
            "mcpServers": {
                "outro": { "command": "/usr/bin/outro", "args": [] }
            }
        });
        insert_mcp_server(&mut s, "ctx", "ctx", &["mcp", "serve"]);

        remove_ctx_mcp_servers(&mut s);

        let servers = s["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers["outro"]["command"], "/usr/bin/outro");
    }

    #[test]
    fn mcp_remove_limpa_chave_quando_unico_era_nosso() {
        let mut s = json!({});
        insert_mcp_server(&mut s, "ctx", "ctx", &["mcp", "serve"]);
        remove_ctx_mcp_servers(&mut s);
        assert_eq!(s, json!({}), "settings deve ficar vazio");
    }
}
