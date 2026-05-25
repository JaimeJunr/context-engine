// Integração com agentes de codificação: instala hooks/configurações que
// fazem o agente passar comandos cobertos por `ctx exec` automaticamente.

use anyhow::Result;
use clap::ValueEnum;

pub mod claude_code;
pub mod claude_desktop;
pub mod hook_handlers;
pub mod settings_merge;

/// Onde a configuração do agente é gravada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// `~/.<agent>/settings.json` — vale para qualquer projeto.
    User,
    /// `.<agent>/settings.json` dentro do projeto atual.
    Project,
}

/// Agentes suportados pelo `ctx install`/`uninstall`.
///
/// Mantemos como enum para validação de CLI; novos agentes
/// (Cursor, Codex, opencode) entram aqui e ganham uma impl de `AgentInstaller`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AgentName {
    #[value(name = "claude-code")]
    ClaudeCode,
    #[value(name = "claude-desktop")]
    ClaudeDesktop,
}

/// Contrato que cada agente deve cumprir para ser instalável via `ctx`.
pub trait AgentInstaller {
    /// Nome humano do agente (para mensagens).
    fn name(&self) -> &'static str;

    /// Instala hooks/configurações no escopo escolhido.
    ///
    /// Idempotente: chamar duas vezes não duplica nada.
    /// `force` permite sobrescrever em casos de conflito conhecido.
    fn install(&self, scope: Scope, force: bool) -> Result<InstallReport>;

    /// Reverte exatamente o que `install` adicionou.
    fn uninstall(&self, scope: Scope) -> Result<UninstallReport>;
}

#[derive(Debug)]
pub struct InstallReport {
    pub settings_path: std::path::PathBuf,
    pub already_installed: bool,
}

#[derive(Debug)]
pub struct UninstallReport {
    pub settings_path: std::path::PathBuf,
    pub removed: bool,
}

/// Resolve a impl correspondente ao enum.
pub fn installer_for(agent: AgentName) -> Box<dyn AgentInstaller> {
    match agent {
        AgentName::ClaudeCode => Box::new(claude_code::ClaudeCodeInstaller),
        AgentName::ClaudeDesktop => Box::new(claude_desktop::ClaudeDesktopInstaller),
    }
}
