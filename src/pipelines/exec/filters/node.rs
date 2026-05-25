use crate::pipelines::exec::types::FilterConfig;

pub fn install() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"^(npm warn|npm error|error|ERR!|added \d+|removed \d+|changed \d+)".to_string(),
        ],
        max_lines: Some(50),
        on_empty: Some("(instalação concluída)".to_string()),
        ..Default::default()
    }
}

pub fn test() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"FAIL|PASS|✓|✗|×|●|FAILED|passed|failed|Tests:".to_string(),
            r"^  ●".to_string(),
            r"Test Suites:".to_string(),
        ],
        tail_lines: Some(50),
        on_empty: Some("(sem resultados de testes)".to_string()),
        ..Default::default()
    }
}

pub fn build() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![r"error|Error|warning|warn|✓|✗|built in".to_string()],
        max_lines: Some(80),
        on_empty: Some("(build concluído)".to_string()),
        ..Default::default()
    }
}

pub fn run_script() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        ..Default::default()
    }
}
