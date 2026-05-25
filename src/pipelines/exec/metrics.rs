use anyhow::Result;
use rusqlite::{params, Connection};

use super::types::{CommandBreakdown, DayBreakdown, ExecutionRecord, SavingsSummary};

/// Migração da tabela execution_metrics
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS execution_metrics (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp            TEXT NOT NULL,
            project_path         TEXT,
            command_name         TEXT NOT NULL,
            raw_command          TEXT NOT NULL,
            proxy_command        TEXT,
            raw_output_size      INTEGER NOT NULL,
            filtered_output_size INTEGER NOT NULL,
            input_tokens         INTEGER NOT NULL,
            output_tokens        INTEGER NOT NULL,
            execution_time_ms    INTEGER NOT NULL,
            exit_code            INTEGER NOT NULL,
            filter_applied       TEXT,
            fallback_path        TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_exec_project_timestamp
            ON execution_metrics(project_path, timestamp);
        CREATE INDEX IF NOT EXISTS idx_exec_command_name
            ON execution_metrics(command_name);
        CREATE INDEX IF NOT EXISTS idx_exec_timestamp
            ON execution_metrics(timestamp);
        ",
    )?;
    Ok(())
}

/// Persiste um registro de execução no banco
pub fn persist(conn: &Connection, record: &ExecutionRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO execution_metrics (
            timestamp, project_path, command_name, raw_command, proxy_command,
            raw_output_size, filtered_output_size, input_tokens, output_tokens,
            execution_time_ms, exit_code, filter_applied, fallback_path
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            record.timestamp,
            record.project_path,
            record.command_name,
            record.raw_command,
            record.proxy_command,
            record.raw_output_size as i64,
            record.filtered_output_size as i64,
            record.input_tokens as i64,
            record.output_tokens as i64,
            record.execution_time_ms as i64,
            record.exit_code,
            record.filter_applied,
            record.fallback_path,
        ],
    )?;
    Ok(())
}

/// Remove registros mais antigos que `retention_days` dias
pub fn prune_old(conn: &Connection, retention_days: u32) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM execution_metrics WHERE timestamp < datetime('now', ?1)",
        params![format!("-{} days", retention_days)],
    )?;
    Ok(count)
}

/// Agrega resumo de economia de tokens
pub fn aggregate_summary(
    conn: &Connection,
    project_path: Option<&str>,
    days: Option<u32>,
) -> Result<SavingsSummary> {
    let since = days
        .map(|d| format!("datetime('now', '-{} days')", d))
        .unwrap_or_else(|| "'1970-01-01'".to_string());

    let project_filter = project_path.map(|_| "AND project_path = ?2").unwrap_or("");

    let query = format!(
        "SELECT
            COUNT(*) as total_commands,
            COALESCE(SUM(input_tokens), 0) as total_input,
            COALESCE(SUM(output_tokens), 0) as total_output,
            COALESCE(SUM(execution_time_ms), 0) as total_time
         FROM execution_metrics
         WHERE timestamp >= {since}
         {project_filter}"
    );

    let (total_commands, total_input, total_output, total_time): (usize, usize, usize, u64) =
        if let Some(path) = project_path {
            conn.query_row(&query, params![path], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
        } else {
            conn.query_row(&query, [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
        };

    let total_saved = total_input.saturating_sub(total_output);
    let avg_savings = if total_input > 0 {
        (total_saved as f64 / total_input as f64) * 100.0
    } else {
        0.0
    };
    let avg_time = if total_commands > 0 {
        total_time / total_commands as u64
    } else {
        0
    };

    // Breakdown por comando
    let breakdown_query = format!(
        "SELECT command_name,
                COUNT(*) as cnt,
                COALESCE(SUM(input_tokens - output_tokens), 0) as saved,
                COALESCE(AVG(CASE WHEN input_tokens > 0 THEN (input_tokens - output_tokens) * 100.0 / input_tokens ELSE 0 END), 0) as avg_pct,
                COALESCE(SUM(execution_time_ms), 0) as total_ms
         FROM execution_metrics
         WHERE timestamp >= {since}
         {project_filter}
         GROUP BY command_name
         ORDER BY saved DESC"
    );

    let breakdown_by_command: Vec<CommandBreakdown> = {
        let mut stmt = conn.prepare(&breakdown_query)?;
        let rows = if let Some(path) = project_path {
            stmt.query_map(params![path], |row| {
                Ok(CommandBreakdown {
                    command_name: row.get(0)?,
                    count: row.get(1)?,
                    saved_tokens: row.get(2)?,
                    avg_savings_percent: row.get(3)?,
                    total_time_ms: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(CommandBreakdown {
                    command_name: row.get(0)?,
                    count: row.get(1)?,
                    saved_tokens: row.get(2)?,
                    avg_savings_percent: row.get(3)?,
                    total_time_ms: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
        };
        rows
    };

    // Breakdown por dia
    let day_query = format!(
        "SELECT substr(timestamp, 1, 10) as day,
                COALESCE(SUM(input_tokens - output_tokens), 0) as saved
         FROM execution_metrics
         WHERE timestamp >= {since}
         {project_filter}
         GROUP BY day
         ORDER BY day DESC"
    );

    let breakdown_by_day: Vec<DayBreakdown> = {
        let mut stmt = conn.prepare(&day_query)?;
        let rows = if let Some(path) = project_path {
            stmt.query_map(params![path], |row| {
                Ok(DayBreakdown {
                    date: row.get(0)?,
                    saved_tokens: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(DayBreakdown {
                    date: row.get(0)?,
                    saved_tokens: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
        };
        rows
    };

    Ok(SavingsSummary {
        total_commands,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_saved_tokens: total_saved,
        avg_savings_percent: avg_savings,
        total_time_ms: total_time,
        avg_time_ms: avg_time,
        breakdown_by_command,
        breakdown_by_day,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::types::ExecutionRecord;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn make_record(cmd: &str, raw_size: usize, filtered_size: usize) -> ExecutionRecord {
        let raw = "x".repeat(raw_size);
        let filtered = "x".repeat(filtered_size);
        ExecutionRecord::new(
            cmd.to_string(),
            cmd.split_whitespace().next().unwrap_or(cmd).to_string(),
            &raw,
            &filtered,
            100,
            0,
        )
    }

    // RED: test_registro_de_execucao
    #[test]
    fn test_registro_de_execucao() {
        // Given: um registro de execução
        let conn = test_conn();
        let record = make_record("git log --oneline", 4000, 400);

        // When: persistido
        persist(&conn, &record).unwrap();

        // Then: pode ser recuperado
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM execution_metrics", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_registro_preserva_invariante_tokens() {
        // savedTokens = inputTokens - outputTokens
        let conn = test_conn();
        let record = make_record("git status", 4000, 1000);

        assert_eq!(record.input_tokens, 1000); // 4000/4
        assert_eq!(record.output_tokens, 250); // 1000/4
        assert_eq!(record.saved_tokens, 750); // 1000 - 250

        persist(&conn, &record).unwrap();

        let (input, output): (i64, i64) = conn
            .query_row(
                "SELECT input_tokens, output_tokens FROM execution_metrics",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(input - output, record.saved_tokens as i64);
    }

    // RED: test_agregacao_de_resumo
    #[test]
    fn test_agregacao_de_resumo() {
        // Given: múltiplos registros persistidos com command_name distintos
        let conn = test_conn();
        persist(&conn, &make_record("git-log", 8000, 800)).unwrap();
        persist(&conn, &make_record("git-status", 4000, 2000)).unwrap();
        persist(&conn, &make_record("npm-test", 16000, 1600)).unwrap();

        // When: resumo agregado
        let summary = aggregate_summary(&conn, None, None).unwrap();

        // Then: totais corretos
        assert_eq!(summary.total_commands, 3);
        assert!(summary.total_saved_tokens > 0);
        assert!(summary.avg_savings_percent > 0.0);
        assert_eq!(summary.breakdown_by_command.len(), 3);
    }

    #[test]
    fn test_poda_de_registros_antigos() {
        // Given: registros existentes (todos recentes)
        let conn = test_conn();
        persist(&conn, &make_record("cmd", 100, 10)).unwrap();

        // When: poda com 0 dias (remove tudo com mais de 0 dias, nenhum é antigo)
        let pruned = prune_old(&conn, 90).unwrap();

        // Then: nada removido (são recentes)
        assert_eq!(pruned, 0);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM execution_metrics", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_resumo_vazio_sem_registros() {
        let conn = test_conn();
        let summary = aggregate_summary(&conn, None, None).unwrap();
        assert_eq!(summary.total_commands, 0);
        assert_eq!(summary.total_saved_tokens, 0);
        assert_eq!(summary.avg_savings_percent, 0.0);
    }
}
