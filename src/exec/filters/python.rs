use crate::exec::types::FilterConfig;

pub fn pytest() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"FAILED|PASSED|ERROR|WARNING".to_string(),
            r"^={3,}".to_string(),
            r"^ERRORS|^FAILURES|short test summary".to_string(),
            r"^\s+File ".to_string(),
            r"AssertionError|Exception".to_string(),
        ],
        tail_lines: Some(40),
        on_empty: Some("(sem resultados de testes)".to_string()),
        ..Default::default()
    }
}

pub fn generic() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        ..Default::default()
    }
}
