use crate::pipelines::exec::dedup::group_repeated;
use crate::pipelines::exec::types::FilterConfig;

pub fn get() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        truncate_lines_at: Some(120),
        max_lines: Some(50),
        ..Default::default()
    }
}

/// `kubectl logs` comprime via dedup (normaliza timestamps/UUIDs e agrupa
/// linhas idênticas com `(×N)`). Mantém tail dos eventos recentes.
pub fn logs() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        preprocess: Some(group_repeated),
        tail_lines: Some(100),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::pipeline::apply_pipeline;

    #[test]
    fn kubectl_logs_agrupa_linhas_repetidas() {
        // 10 linhas idênticas exceto timestamp viram 1 com (×10).
        let mut input = String::new();
        for i in 0..10 {
            input.push_str(&format!("2024-01-15T12:00:{:02}Z INFO heartbeat ok\n", i));
        }
        let out = apply_pipeline(&input, &logs());
        assert!(out.contains("(×10)"), "esperava agrupamento: {}", out);
        assert!(
            out.lines().count() <= 2,
            "deveria reduzir drasticamente: {} linhas",
            out.lines().count()
        );
    }

    #[test]
    fn kubectl_logs_preserva_linhas_distintas() {
        let input = "\
2024-01-15T12:00:00Z INFO server starting
2024-01-15T12:00:01Z INFO listening on port 8080
2024-01-15T12:00:02Z ERROR connection refused
2024-01-15T12:00:03Z INFO retrying";
        let out = apply_pipeline(input, &logs());
        // Mensagens distintas devem aparecer (todas 4)
        assert!(out.contains("server starting"));
        assert!(out.contains("connection refused"));
    }
}
