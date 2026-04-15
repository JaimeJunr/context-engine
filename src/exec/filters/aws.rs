use crate::exec::types::FilterConfig;

pub fn generic() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(80),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

pub fn logs() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        tail_lines: Some(50),
        keep_lines_matching: vec![
            r"\d{4}-\d{2}-\d{2}".to_string(), // linhas com timestamp
        ],
        ..Default::default()
    }
}
