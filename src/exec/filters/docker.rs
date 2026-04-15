use crate::exec::types::FilterConfig;

pub fn ps() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        truncate_lines_at: Some(120),
        max_lines: Some(50),
        ..Default::default()
    }
}

pub fn images() -> FilterConfig {
    ps()
}

pub fn logs() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        tail_lines: Some(50),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

pub fn compose_ps() -> FilterConfig {
    ps()
}
