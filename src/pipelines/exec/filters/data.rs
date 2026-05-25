use crate::pipelines::exec::types::FilterConfig;

pub fn curl() -> FilterConfig {
    // curl pode retornar JSON ou binário — apenas strip ANSI e limita linhas
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

pub fn jq() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        ..Default::default()
    }
}

pub fn sqlite3() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        truncate_lines_at: Some(150),
        ..Default::default()
    }
}
