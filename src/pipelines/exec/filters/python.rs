use crate::pipelines::exec::types::FilterConfig;

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

/// `ruff check` — preserva `file:line:col: code message`, remove sumário verboso.
pub fn ruff() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            // Ruff: "src/foo.py:12:5: E501 line too long"
            r"\.py:\d+:\d+:\s+[A-Z]\d+".to_string(),
            // "Found N errors."
            r"^Found \d+ error".to_string(),
            // "All checks passed!"
            r"^All checks passed".to_string(),
        ],
        max_lines: Some(200),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `mypy` — preserva `file:line: error/note: msg`, remove progresso.
pub fn mypy() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            // mypy: "src/foo.py:12: error: Incompatible types"
            r"\.py:\d+:\s+(error|note|warning)".to_string(),
            // "Found N errors in X files"
            r"^Found \d+ error".to_string(),
            // "Success: no issues found"
            r"^Success: no issues".to_string(),
        ],
        max_lines: Some(200),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::pipeline::apply_pipeline;

    #[test]
    fn ruff_preserva_violacoes() {
        let input = "\
src/app.py:12:5: E501 line too long (105 > 100)
src/util.py:34:1: F401 'os' imported but unused
Found 2 errors.";
        let out = apply_pipeline(input, &ruff());
        assert!(out.contains("E501"));
        assert!(out.contains("F401"));
        assert!(out.contains("Found 2 errors"));
    }

    #[test]
    fn ruff_ok_quando_clean() {
        let input = "All checks passed!";
        let out = apply_pipeline(input, &ruff());
        assert!(out.contains("All checks passed"));
    }

    #[test]
    fn mypy_preserva_erros() {
        let input = "\
src/app.py:12: error: Incompatible types in assignment\nsrc/app.py:13: note: Did you mean 'foo'?\nFound 1 error in 1 file";
        let out = apply_pipeline(input, &mypy());
        assert!(out.contains("Incompatible types"));
        assert!(out.contains("Found 1 error"));
    }

    #[test]
    fn mypy_success_passa() {
        let input = "Success: no issues found in 5 source files";
        let out = apply_pipeline(input, &mypy());
        assert!(out.contains("Success"));
    }
}
