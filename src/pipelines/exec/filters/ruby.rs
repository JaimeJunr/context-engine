// Filtros para o ecossistema Ruby: rubocop, rspec, rake.

use crate::pipelines::exec::types::FilterConfig;

/// `rubocop` — preserva `file:line:col: Letter: msg` formato compacto.
pub fn rubocop() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            // rubocop: "foo.rb:12:5: C: Style/Foo: msg"
            r"\.rb:\d+:\d+:".to_string(),
            // sumário "N files inspected, M offenses detected"
            r"files? inspected".to_string(),
            r"offenses? detected".to_string(),
            r"no offenses detected".to_string(),
        ],
        max_lines: Some(200),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `rspec` — preserva linhas de falha, sumário, exemplos pendentes.
pub fn rspec() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            // Linha de resultado: "Finished in 0.03 seconds"
            r"^Finished in".to_string(),
            // "5 examples, 1 failure"
            r"\d+ examples?,\s+\d+ failures?".to_string(),
            // "Failed examples:" header
            r"^Failed examples:".to_string(),
            // "rspec ./spec/foo_spec.rb:42 # description"
            r"^rspec\s+\./spec".to_string(),
            // backtraces e mensagens
            r"^\s+Failure/Error:".to_string(),
            r"^\s+expected:".to_string(),
            r"^\s+got:".to_string(),
            // Pending
            r"^Pending:".to_string(),
        ],
        max_lines: Some(200),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `rake` genérico — strip ANSI e limita.
pub fn rake() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(120),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::pipeline::apply_pipeline;

    #[test]
    fn rubocop_preserva_offenses() {
        let input = "\
Inspecting 12 files
.C..W.......

Offenses:

app/models/user.rb:12:5: C: Style/StringLiterals: Prefer single-quoted strings.
app/models/user.rb:34:1: W: Lint/UselessAssignment: Useless assignment.

12 files inspected, 2 offenses detected";
        let out = apply_pipeline(input, &rubocop());
        assert!(out.contains("Style/StringLiterals"));
        assert!(out.contains("Lint/UselessAssignment"));
        assert!(out.contains("2 offenses detected"));
    }

    #[test]
    fn rubocop_ok_quando_sem_offenses() {
        let input = "12 files inspected, no offenses detected";
        let out = apply_pipeline(input, &rubocop());
        assert!(out.contains("no offenses detected"));
    }

    #[test]
    fn rspec_preserva_falhas() {
        let input = "\
Run options: include {:focus=>true}

Randomized with seed 12345
.F..

Failures:

  1) User#name returns the name
     Failure/Error: expect(user.name).to eq('Alice')

       expected: 'Alice'
            got: 'Bob'

     # ./spec/user_spec.rb:5:in 'block (2 levels)'

Finished in 0.03 seconds (files took 0.5 seconds to load)
4 examples, 1 failure

Failed examples:

rspec ./spec/user_spec.rb:4 # User#name returns the name";
        let out = apply_pipeline(input, &rspec());
        assert!(out.contains("4 examples, 1 failure"));
        assert!(out.contains("rspec ./spec/user_spec.rb:4"));
        assert!(out.contains("expected"));
    }
}
