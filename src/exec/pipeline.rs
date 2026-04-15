use regex::Regex;

use super::types::FilterConfig;

/// Aplica o pipeline de 8 estágios de filtragem declarativa
///
/// Estágios (em ordem):
/// 1. Remoção de códigos de escape ANSI
/// 2. Substituições textuais
/// 3. Curto-circuito global
/// 4. Seleção de linhas (inclusão ou exclusão)
/// 5. Truncamento por linha
/// 6. Retenção de head/tail
/// 7. Limite absoluto de linhas
/// 8. Mensagem para saída vazia
pub fn apply_pipeline(input: &str, config: &FilterConfig) -> String {
    // Estágio 0: pré-processamento opcional
    let preprocessed;
    let input = if let Some(f) = config.preprocess {
        preprocessed = f(input);
        preprocessed.as_str()
    } else {
        input
    };

    // Estágio 1: remoção de escapes ANSI
    let s1 = if config.strip_ansi {
        strip_ansi_escapes(input)
    } else {
        input.to_string()
    };

    // Estágio 2: substituições textuais (encadeáveis, linha a linha)
    let s2 = apply_replacements(&s1, &config.replacements);

    // Estágio 3: curto-circuito global
    if let Some(msg) = check_global_match(&s2, &config.match_output) {
        return msg;
    }

    // Estágio 4: seleção de linhas
    let s4 = filter_lines(
        &s2,
        &config.strip_lines_matching,
        &config.keep_lines_matching,
    );

    // Estágio 5: truncamento por linha
    let s5 = if let Some(width) = config.truncate_lines_at {
        truncate_lines(&s4, width)
    } else {
        s4
    };

    // Estágio 6: head/tail
    let s6 = apply_head_tail(&s5, config.head_lines, config.tail_lines);

    // Estágio 7: limite absoluto
    let s7 = if let Some(max) = config.max_lines {
        limit_lines(&s6, max)
    } else {
        s6
    };

    // Estágio 8: mensagem para vazio
    if s7.trim().is_empty() {
        if let Some(msg) = &config.on_empty {
            return msg.clone();
        }
    }

    s7
}

/// Estágio 1: remove códigos de escape ANSI/VT100
pub fn strip_ansi_escapes(input: &str) -> String {
    // Padrão cobre ESC[ sequences e outras sequências de escape comuns
    static ANSI_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = ANSI_RE.get_or_init(|| {
        Regex::new(r"\x1b\[[0-9;]*[A-Za-z]|\x1b[()][AB012]|\x1b[=>]|\x07|\x1b\].*?\x07").unwrap()
    });
    re.replace_all(input, "").to_string()
}

/// Estágio 2: substituições textuais encadeáveis
fn apply_replacements(input: &str, replacements: &[(String, String)]) -> String {
    let mut result = input.to_string();
    for (pattern, replacement) in replacements {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, replacement.as_str()).to_string();
        }
    }
    result
}

/// Estágio 3: verifica regras de curto-circuito global
fn check_global_match(input: &str, rules: &[super::types::GlobalMatchRule]) -> Option<String> {
    for rule in rules {
        if let Ok(pattern_re) = Regex::new(&rule.pattern) {
            if pattern_re.is_match(input) {
                // Verifica se exceção também bate (skip se bater)
                let exception_matches = rule
                    .exception
                    .as_ref()
                    .and_then(|exc| Regex::new(exc).ok())
                    .map(|re| re.is_match(input))
                    .unwrap_or(false);
                if !exception_matches {
                    return Some(rule.message.clone());
                }
            }
        }
    }
    None
}

/// Estágio 4: seleção de linhas por padrão
fn filter_lines(input: &str, strip: &[String], keep: &[String]) -> String {
    let lines: Vec<&str> = input.lines().collect();

    let filtered: Vec<&str> = lines
        .into_iter()
        .filter(|line| {
            // Exclui linhas que batem padrão de exclusão
            let should_strip = strip
                .iter()
                .any(|pat| Regex::new(pat).map(|re| re.is_match(line)).unwrap_or(false));
            if should_strip {
                return false;
            }
            // Inclui apenas linhas que batem padrão de inclusão (se definido)
            if !keep.is_empty() {
                return keep
                    .iter()
                    .any(|pat| Regex::new(pat).map(|re| re.is_match(line)).unwrap_or(false));
            }
            true
        })
        .collect();

    filtered.join("\n")
}

/// Estágio 5: truncamento por largura
fn truncate_lines(input: &str, width: usize) -> String {
    input
        .lines()
        .map(|line| {
            if line.len() > width {
                &line[..width]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Estágio 6: retenção de head/tail
fn apply_head_tail(input: &str, head: Option<usize>, tail: Option<usize>) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let n = lines.len();

    match (head, tail) {
        (Some(h), Some(t)) => {
            let head_lines: Vec<&str> = lines.iter().take(h).copied().collect();
            let tail_start = n.saturating_sub(t);
            let tail_lines: Vec<&str> = lines.iter().skip(tail_start).copied().collect();
            [head_lines, tail_lines].concat().join("\n")
        }
        (Some(h), None) => lines.iter().take(h).copied().collect::<Vec<_>>().join("\n"),
        (None, Some(t)) => {
            let start = n.saturating_sub(t);
            lines
                .iter()
                .skip(start)
                .copied()
                .collect::<Vec<_>>()
                .join("\n")
        }
        (None, None) => input.to_string(),
    }
}

/// Estágio 7: limite absoluto de linhas
///
/// Quando truncar, adiciona rodapé informativo com contagem.
fn limit_lines(input: &str, max: usize) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let total = lines.len();
    if total <= max {
        return lines.join("\n");
    }
    let mut result = lines[..max].join("\n");
    result.push_str(&format!(
        "\n[... truncado: mostrando {} de {} linhas]",
        max, total
    ));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::{
        filters,
        types::{FilterConfig, GlobalMatchRule},
    };

    fn default_config() -> FilterConfig {
        FilterConfig::default()
    }

    // =========================================================================
    // Testes de comportamento com fixtures reais
    // O que o código ENTREGA, não como internamente processa.
    // =========================================================================

    #[test]
    fn cargo_test_com_falhas_mostra_falhas_e_resume() {
        // Dado: output real de `cargo test` com 2 falhas
        let input = include_str!("../../tests/fixtures/cargo_test_with_failure.txt");
        let config = filters::cargo::test();

        let result = apply_pipeline(input, &config);

        // O agente precisa ver QUAIS testes falharam
        assert!(
            result.contains("test_login_invalido"),
            "falha específica deve aparecer"
        );
        assert!(
            result.contains("test_session_expiry"),
            "segunda falha deve aparecer"
        );
        // O agente precisa ver o resultado final
        assert!(
            result.contains("2 failed"),
            "resumo de falhas deve aparecer"
        );
        // Ruído de compilação não deve dominar
        let lines: Vec<&str> = result.lines().collect();
        assert!(
            lines.len() < 30,
            "output filtrado deve ser conciso (era {} linhas)",
            input.lines().count()
        );
    }

    #[test]
    fn cargo_test_tudo_ok_mostra_resultado_final() {
        // Dado: output real de `cargo test` 100% ok
        let input = include_str!("../../tests/fixtures/cargo_test_all_pass.txt");
        let config = filters::cargo::test();

        let result = apply_pipeline(input, &config);

        assert!(
            result.contains("13 passed") || result.contains("ok"),
            "deve mostrar que testes passaram"
        );
        // Nenhuma linha de falha deve aparecer
        assert!(
            !result.contains("FAILED"),
            "não deve conter FAILED quando tudo passou"
        );
    }

    #[test]
    fn git_status_mostra_arquivos_modificados() {
        // Dado: output real de `git status` com mudanças
        let input = include_str!("../../tests/fixtures/git_status_dirty.txt");
        let config = filters::git::status();

        let result = apply_pipeline(input, &config);

        // O agente precisa saber o que foi modificado
        assert!(
            result.contains("src/auth.rs"),
            "arquivo modificado deve aparecer"
        );
        assert!(
            result.contains("src/config.rs"),
            "segundo arquivo modificado deve aparecer"
        );
        // Instruções redundantes do git podem ser removidas mas o estado deve estar claro
        assert!(!result.is_empty(), "resultado não pode ser vazio");
    }

    #[test]
    fn git_log_responde_limite_de_linhas() {
        // Dado: output de `git log` com 5 commits
        let input = include_str!("../../tests/fixtures/git_log.txt");
        let config = filters::git::log();

        let result = apply_pipeline(input, &config);

        // Resultado deve ser menor que a entrada
        assert!(
            result.lines().count() <= input.lines().count(),
            "filtro não pode aumentar o output"
        );
        // Deve conter os hashes dos commits (informação essencial)
        assert!(
            result.contains("feat(exec)") || result.contains("a1b2c3"),
            "commits devem aparecer"
        );
    }

    #[test]
    fn output_sem_filtro_passa_integro() {
        // Dado: qualquer input, sem config de filtro
        let config = default_config();
        let input = "fn main() {\n    println!(\"hello\");\n}";

        let result = apply_pipeline(input, &config);

        assert_eq!(
            result, input,
            "sem filtro o conteúdo deve ser preservado integralmente"
        );
    }

    #[test]
    fn ansi_removido_do_output_de_terminal() {
        // Dado: output colorido típico de ferramentas CLI
        let input = "\x1b[32m✓ ok\x1b[0m\n\x1b[31m✗ FAILED: auth_test\x1b[0m\n\x1b[33mwarning: unused\x1b[0m";
        let mut config = default_config();
        config.strip_ansi = true;

        let result = apply_pipeline(input, &config);

        assert!(
            !result.contains("\x1b["),
            "códigos ANSI não devem aparecer no resultado"
        );
        assert!(
            result.contains("FAILED: auth_test"),
            "conteúdo visível deve ser preservado"
        );
        assert!(
            result.contains("warning: unused"),
            "warnings devem ser preservados"
        );
    }

    #[test]
    fn resultado_vazio_recebe_mensagem_de_fallback() {
        // Dado: filtro agressivo que remove tudo + mensagem configurada
        let mut config = default_config();
        config.keep_lines_matching = vec![r"IMPOSSÍVEL_DE_ENCONTRAR".to_string()];
        config.on_empty = Some("(sem saída relevante)".to_string());

        let result = apply_pipeline("qualquer coisa aqui", &config);

        assert_eq!(result, "(sem saída relevante)");
    }

    // =========================================================================
    // Testes de propriedade (invariantes que sempre devem ser verdadeiros)
    // =========================================================================

    #[test]
    fn filtro_nunca_aumenta_numero_de_linhas() {
        let inputs = vec![
            "linha1\nlinha2\nlinha3\nlinha4\nlinha5",
            "apenas uma linha",
            "",
        ];
        for input in inputs {
            let mut config = default_config();
            config.max_lines = Some(3);
            let result = apply_pipeline(input, &config);
            assert!(
                result.lines().count() <= input.lines().count(),
                "filtro não pode criar linhas novas"
            );
        }
    }

    #[test]
    fn pipeline_e_deterministico_para_mesma_entrada() {
        let mut config = default_config();
        config.strip_ansi = true;
        config.max_lines = Some(5);
        let input = "\x1b[32m123\x1b[0m tokens\nline2\nline3\nline4\nline5\nline6";

        let r1 = apply_pipeline(input, &config);
        let r2 = apply_pipeline(input, &config);

        assert_eq!(r1, r2, "mesma entrada deve sempre produzir mesmo output");
    }

    #[test]
    fn curto_circuito_interrompe_pipeline_ao_detectar_padrao() {
        let mut config = default_config();
        config.match_output = vec![GlobalMatchRule {
            pattern: r"ERRO_FATAL".to_string(),
            message: "[erro fatal detectado — saída suprimida]".to_string(),
            exception: None,
        }];
        // mesmo com outras regras, o curto-circuito deve vencer
        config.max_lines = Some(1);

        let input = "linha normal\nERRO_FATAL: sistema parou\noutra linha";
        let result = apply_pipeline(input, &config);

        assert_eq!(result, "[erro fatal detectado — saída suprimida]");
    }

    #[test]
    fn limit_lines_adiciona_rodape_quando_trunca() {
        let input = "l1\nl2\nl3\nl4\nl5";
        let result = limit_lines(input, 3);

        assert!(
            result.contains("[... truncado: mostrando 3 de 5 linhas]"),
            "rodapé de truncamento deve aparecer quando há corte: {:?}",
            result
        );
        assert!(
            result.starts_with("l1\nl2\nl3"),
            "linhas iniciais devem ser preservadas"
        );
    }

    #[test]
    fn limit_lines_sem_truncamento_nao_adiciona_rodape() {
        let input = "l1\nl2\nl3";
        let result = limit_lines(input, 5);

        assert!(
            !result.contains("[... truncado"),
            "não deve adicionar rodapé quando não há corte"
        );
        assert_eq!(result, input, "conteúdo deve ser preservado integralmente");
    }

    #[test]
    fn limit_lines_exato_no_limite_nao_adiciona_rodape() {
        let input = "l1\nl2\nl3";
        let result = limit_lines(input, 3);

        assert!(
            !result.contains("[... truncado"),
            "não deve adicionar rodapé quando total == max"
        );
    }

    #[test]
    fn apply_pipeline_com_max_lines_inclui_rodape_de_truncamento() {
        let input = (1..=10)
            .map(|i| format!("linha {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let mut config = default_config();
        config.max_lines = Some(5);

        let result = apply_pipeline(&input, &config);

        assert!(
            result.contains("[... truncado: mostrando 5 de 10 linhas]"),
            "pipeline deve propagar rodapé de truncamento: {:?}",
            result
        );
    }

    #[test]
    fn excecao_de_curto_circuito_deixa_pipeline_continuar() {
        let mut config = default_config();
        config.match_output = vec![GlobalMatchRule {
            pattern: r"WARNING".to_string(),
            message: "[suprimido]".to_string(),
            exception: Some(r"WARNING: ignorável".to_string()),
        }];

        let result = apply_pipeline("WARNING: ignorável aqui", &config);

        assert_ne!(result, "[suprimido]", "exceção deve impedir a supressão");
        assert!(
            result.contains("WARNING: ignorável"),
            "conteúdo original deve estar presente"
        );
    }
}
