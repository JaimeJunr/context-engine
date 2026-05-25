pub mod dedup;
pub mod filters;
pub mod metrics;
pub mod pipeline;
pub mod registry;
pub mod types;

pub use types::{ExecutionRecord, FilterConfig, FilterLevel, ParseResult, SavingsSummary};

use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

/// Resultado capturado de uma execução proxy: output já filtrado + exit code.
pub struct ProxyOutput {
    pub stdout: String,
    pub exit_code: i32,
}

/// Executa `argv` com filtro aplicado e RETORNA o output filtrado (não imprime).
///
/// Útil para callers que precisam do output como string — MCP server, testes,
/// composição programática. Persiste métricas no caminho normal.
pub fn run_proxy_capture(argv: Vec<String>) -> Result<ProxyOutput> {
    if argv.is_empty() {
        anyhow::bail!("ctx exec: nenhum comando fornecido");
    }

    let cmd_name = &argv[0];
    let cmd_args = &argv[1..];

    let filter = registry::lookup(cmd_name, cmd_args);
    let filter_name = filter.as_ref().map(|_| {
        if cmd_args.is_empty() {
            cmd_name.clone()
        } else {
            format!("{} {}", cmd_name, cmd_args[0])
        }
    });

    let start = Instant::now();

    let output = Command::new(cmd_name)
        .args(cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let elapsed_ms = start.elapsed().as_millis() as u64;

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::NotFound {
                format!("ctx exec: comando '{}' não encontrado no PATH", cmd_name)
            } else {
                format!("ctx exec: falha ao executar '{}': {}", cmd_name, e)
            };
            return Ok(ProxyOutput {
                stdout: msg,
                exit_code: 127,
            });
        }
    };

    let exit_code = output.status.code().unwrap_or(1);

    // Combina stdout + stderr
    let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let raw_stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let raw_combined = if raw_stderr.is_empty() {
        raw_stdout.clone()
    } else if raw_stdout.is_empty() {
        raw_stderr.clone()
    } else {
        format!("{}{}", raw_stdout, raw_stderr)
    };

    let filtered = match &filter {
        Some(config) => pipeline::apply_pipeline(&raw_combined, config),
        None => raw_combined.clone(),
    };

    // Persiste métricas (melhor esforço — não falha se DB indisponível)
    let _ = persist_metric(
        cmd_name,
        &argv,
        &raw_combined,
        &filtered,
        elapsed_ms,
        exit_code,
        filter_name.as_deref(),
    );

    Ok(ProxyOutput {
        stdout: filtered,
        exit_code,
    })
}

/// Proxy universal: executa `argv`, aplica filtro se registrado, escreve stdout.
/// Retorna o exit code do subprocesso. Wrapper de `run_proxy_capture`.
pub fn run_proxy(argv: Vec<String>) -> Result<i32> {
    let ProxyOutput { stdout, exit_code } = run_proxy_capture(argv)?;

    let stdout_handle = std::io::stdout();
    let mut handle = stdout_handle.lock();
    handle.write_all(stdout.as_bytes())?;
    if !stdout.ends_with('\n') && !stdout.is_empty() {
        handle.write_all(b"\n")?;
    }

    Ok(exit_code)
}

fn persist_metric(
    cmd_name: &str,
    argv: &[String],
    raw: &str,
    filtered: &str,
    elapsed_ms: u64,
    exit_code: i32,
    filter_name: Option<&str>,
) -> Result<()> {
    use dirs::home_dir;
    use rusqlite::Connection;

    let db_path = home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("context_engine")
        .join("catalog.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let conn = Connection::open(&db_path)?;
    metrics::migrate(&conn)?;

    let project_path = std::env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()));

    let raw_command = argv.join(" ");
    let mut record = ExecutionRecord::new(
        raw_command,
        cmd_name.to_string(),
        raw,
        filtered,
        elapsed_ms,
        exit_code,
    );
    record.filter_applied = filter_name.map(|s| s.to_string());
    record.project_path = project_path;

    metrics::persist(&conn, &record)?;
    Ok(())
}
