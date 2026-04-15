use crate::exec::types::FilterConfig;

pub fn pr_list() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(30),
        ..Default::default()
    }
}

pub fn pr_view() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(60),
        ..Default::default()
    }
}

pub fn issue_list() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(30),
        ..Default::default()
    }
}

pub fn run_list() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(30),
        ..Default::default()
    }
}

pub fn generic() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(80),
        ..Default::default()
    }
}
