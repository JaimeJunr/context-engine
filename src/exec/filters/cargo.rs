use crate::exec::types::FilterConfig;

pub fn test() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"^test ".to_string(),
            r"FAILED".to_string(),
            r"^running \d+".to_string(),
            r"^\d+ passed".to_string(),
            r"test result".to_string(),
            r"^error".to_string(),
            r"^thread '".to_string(),
            r"^note:".to_string(),
        ],
        tail_lines: Some(40),
        on_empty: Some("(sem saída de testes)".to_string()),
        ..Default::default()
    }
}

pub fn build() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"^error".to_string(),
            r"^warning".to_string(),
            r"^  -->".to_string(),
            r"Finished|Compiling|error\[".to_string(),
        ],
        max_lines: Some(100),
        on_empty: Some("(build sem erros ou avisos)".to_string()),
        ..Default::default()
    }
}

pub fn clippy() -> FilterConfig {
    build()
}

pub fn fmt() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(50),
        on_empty: Some("(formatação ok)".to_string()),
        ..Default::default()
    }
}

pub fn run() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(200),
        ..Default::default()
    }
}
