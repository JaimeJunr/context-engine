use crate::exec::types::FilterConfig;

pub fn ls() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        truncate_lines_at: Some(120),
        max_lines: Some(80),
        ..Default::default()
    }
}

pub fn find() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(200),
        strip_lines_matching: vec![
            r"^find: ".to_string(), // erros de permissão
        ],
        ..Default::default()
    }
}

pub fn tree() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        ..Default::default()
    }
}

pub fn grep() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        truncate_lines_at: Some(150),
        max_lines: Some(150),
        ..Default::default()
    }
}
