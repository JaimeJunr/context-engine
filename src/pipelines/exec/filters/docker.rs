use crate::pipelines::exec::dedup::group_repeated;
use crate::pipelines::exec::types::FilterConfig;

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

/// `docker logs` comprime via dedup com normalização de tokens variáveis.
pub fn logs() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        preprocess: Some(group_repeated),
        tail_lines: Some(100),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

pub fn compose_ps() -> FilterConfig {
    ps()
}
