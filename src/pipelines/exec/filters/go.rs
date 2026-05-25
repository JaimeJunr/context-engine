// Filtros para o ecossistema Go: go test, go build, golangci-lint.

use crate::pipelines::exec::types::FilterConfig;

/// `go test ./...` — preserva linhas FAIL/PASS e diagnósticos.
pub fn test() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"^---\s+(FAIL|PASS):".to_string(),
            r"^(FAIL|ok|PASS)\s+".to_string(),
            r"^\s+.+_test\.go:\d+:".to_string(),
            r"^FAIL\b".to_string(),
            r"panic:".to_string(),
        ],
        max_lines: Some(150),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `go build` — silencioso em sucesso, mantém erros.
pub fn build() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"^\S+\.go:\d+:\d+:".to_string(),
            r"^# ".to_string(),
            r"error".to_string(),
            r"undefined:".to_string(),
        ],
        max_lines: Some(80),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `golangci-lint run` — preserva linhas `file:line:col: msg (linter)`.
pub fn golangci() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            // "foo.go:12:5: msg (errcheck)"
            r"\.go:\d+:\d+:".to_string(),
            r"^Error:".to_string(),
            r"^level=error".to_string(),
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
    fn go_test_preserva_falhas() {
        let input = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSub
    math_test.go:42: expected 5, got 3
--- FAIL: TestSub (0.00s)
FAIL
exit status 1
FAIL	myapp/math	0.012s";
        let out = apply_pipeline(input, &test());
        assert!(out.contains("PASS: TestAdd"));
        assert!(out.contains("FAIL: TestSub"));
        assert!(out.contains("expected 5"));
    }

    #[test]
    fn golangci_preserva_diagnosticos() {
        let input = "\
foo.go:12:5: Error: undefined name 'x' (typecheck)
foo.go:20:1: declared but not used: tmp (govet)";
        let out = apply_pipeline(input, &golangci());
        assert!(out.contains("undefined name"));
        assert!(out.contains("declared but not used"));
    }

    #[test]
    fn go_build_clean_reporta_ok() {
        let out = apply_pipeline("", &build());
        assert_eq!(out, "ok");
    }
}
