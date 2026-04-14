use serde::{Deserialize, Serialize};

/// Registro de uma execução rastreada pelo proxy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionRecord {
    pub timestamp: String,
    pub raw_command: String,
    pub proxy_command: Option<String>,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub saved_tokens: usize,
    pub savings_percent: f64,
    pub execution_time_ms: u64,
    pub project_path: Option<String>,
    pub command_name: String,
    pub raw_output_size: usize,
    pub filtered_output_size: usize,
    pub exit_code: i32,
    pub filter_applied: Option<String>,
    pub fallback_path: Option<String>,
}

impl ExecutionRecord {
    /// Cria um novo registro validando invariantes
    pub fn new(
        raw_command: String,
        command_name: String,
        raw_output: &str,
        filtered_output: &str,
        execution_time_ms: u64,
        exit_code: i32,
    ) -> Self {
        let raw_output_size = raw_output.len();
        let filtered_output_size = filtered_output.len();
        let input_tokens = raw_output_size / 4;
        let output_tokens = filtered_output_size / 4;
        let saved_tokens = input_tokens.saturating_sub(output_tokens);
        let savings_percent = if input_tokens > 0 {
            (saved_tokens as f64 / input_tokens as f64) * 100.0
        } else {
            0.0
        };
        let timestamp = chrono::Utc::now().to_rfc3339();

        Self {
            timestamp,
            raw_command,
            proxy_command: None,
            input_tokens,
            output_tokens,
            saved_tokens,
            savings_percent,
            execution_time_ms,
            project_path: None,
            command_name,
            raw_output_size,
            filtered_output_size,
            exit_code,
            filter_applied: None,
            fallback_path: None,
        }
    }
}

/// Resumo agregado de economia de tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsSummary {
    pub total_commands: usize,
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub total_saved_tokens: usize,
    pub avg_savings_percent: f64,
    pub total_time_ms: u64,
    pub avg_time_ms: u64,
    pub breakdown_by_command: Vec<CommandBreakdown>,
    pub breakdown_by_day: Vec<DayBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBreakdown {
    pub command_name: String,
    pub count: usize,
    pub saved_tokens: usize,
    pub avg_savings_percent: f64,
    pub total_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayBreakdown {
    pub date: String,
    pub saved_tokens: usize,
}

/// Configuração do pipeline de filtragem
#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    pub strip_ansi: bool,
    pub replacements: Vec<(String, String)>,
    pub match_output: Vec<GlobalMatchRule>,
    pub strip_lines_matching: Vec<String>,
    pub keep_lines_matching: Vec<String>,
    pub truncate_lines_at: Option<usize>,
    pub head_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub on_empty: Option<String>,
    pub filter_stderr: bool,
}

/// Regra de curto-circuito global
#[derive(Debug, Clone)]
pub struct GlobalMatchRule {
    pub pattern: String,
    pub message: String,
    pub exception: Option<String>,
}

/// Nível de filtragem para leitura de código
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FilterLevel {
    #[default]
    None,
    Minimal,
    Aggressive,
}

/// Resultado de parsing (com degradação graceful)
#[derive(Debug, Clone, PartialEq)]
pub enum ParseResult {
    Full(String),
    Degraded {
        content: String,
        warnings: Vec<String>,
    },
    Passthrough {
        truncated: String,
    },
}
