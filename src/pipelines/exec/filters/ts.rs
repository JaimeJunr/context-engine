// Filtros para linters/typecheckers JS/TS: tsc, eslint, prettier, biome.
//
// Estratégia comum: preservar a linha "file:line:col error/warning" e a mensagem;
// remover boilerplate de progresso, headers de versão, summary verboso.

use crate::pipelines::exec::types::FilterConfig;

/// `tsc` (TypeScript compiler) — preserva erros `file.ts(line,col): error TS####: msg`,
/// remove rastros de progresso.
pub fn tsc() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        // Mantém apenas linhas de erro/warning (formato compacto) e sumário.
        keep_lines_matching: vec![
            r"\.(ts|tsx|js|jsx|d\.ts)\(\d+,\d+\):\s*(error|warning)".to_string(),
            r"^Found \d+ error".to_string(),
        ],
        max_lines: Some(200),
        on_empty: Some("ok (no type errors)".to_string()),
        ..Default::default()
    }
}

/// `eslint` — preserva `file:line:col error/warning rule-name` formato compacto.
pub fn eslint() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        // Remove banner de versão e linhas vazias de separação.
        strip_lines_matching: vec![
            r"^✖ \d+ problems? \(\d+ errors?, \d+ warnings?\)\s*$".to_string()
        ],
        keep_lines_matching: vec![
            r"^\s*\d+:\d+\s+(error|warning)".to_string(), // "12:5  error  Foo  rule"
            r"\.(ts|tsx|js|jsx)$".to_string(),            // headers de arquivo
            r"^\s*\d+\s+problems?\s+\(\d+\s+errors?".to_string(), // sumário final
        ],
        max_lines: Some(200),
        on_empty: Some("ok (no lint errors)".to_string()),
        ..Default::default()
    }
}

/// `prettier --check` — preserva apenas arquivos com diff.
pub fn prettier() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"^\[warn\]\s+".to_string(),
            r"^Code style issues found".to_string(),
            r"^All matched files use Prettier code style".to_string(),
        ],
        max_lines: Some(100),
        on_empty: Some("ok (no formatting issues)".to_string()),
        ..Default::default()
    }
}

/// `biome check/format/lint` — formato similar ao eslint.
pub fn biome() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"×".to_string(),               // erros do biome
            r"^\s*\w+/\w+/\w+".to_string(), // rule path: "lint/correctness/noUnused"
            r"^Found \d+ ".to_string(),     // sumário
            r"^Checked \d+ file".to_string(),
            r"errors? found".to_string(),
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
    fn tsc_preserva_erros_e_remove_ruido() {
        let input = "\
src/app.ts(12,5): error TS2322: Type 'string' is not assignable to type 'number'.
src/util.ts(34,10): error TS2304: Cannot find name 'foo'.
Found 2 errors in 2 files.

Errors  Files
     1  src/app.ts:12
     1  src/util.ts:34";
        let out = apply_pipeline(input, &tsc());
        assert!(out.contains("TS2322"));
        assert!(out.contains("TS2304"));
        assert!(out.contains("Found 2 errors"));
        // Tabela visual deve sumir
        assert!(!out.contains("Errors  Files"));
    }

    #[test]
    fn tsc_sem_erros_reporta_ok() {
        let out = apply_pipeline("", &tsc());
        assert_eq!(out, "ok (no type errors)");
    }

    #[test]
    fn eslint_preserva_violacoes() {
        let input = "
/repo/src/app.ts
  12:5  error    'foo' is defined but never used  no-unused-vars
  34:10 warning  Missing return type              @typescript-eslint/explicit-function-return-type

✖ 2 problems (1 error, 1 warning)
";
        let out = apply_pipeline(input, &eslint());
        assert!(out.contains("no-unused-vars"));
        assert!(out.contains("warning"));
    }

    #[test]
    fn prettier_sem_issues_reporta_ok() {
        let input = "All matched files use Prettier code style!";
        let out = apply_pipeline(input, &prettier());
        assert!(out.contains("All matched files"));
    }
}
