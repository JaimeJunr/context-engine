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
fn limit_lines(input: &str, max: usize) -> String {
    input.lines().take(max).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::types::{FilterConfig, GlobalMatchRule};

    fn default_config() -> FilterConfig {
        FilterConfig::default()
    }

    // --- RED phase tests ---

    #[test]
    fn test_remocao_de_escapes_visuais() {
        // Given: texto contendo códigos de escape ANSI
        let input = "\x1b[31mErro\x1b[0m: arquivo não encontrado";
        // When: estágio de remoção de escapes aplicado
        let result = strip_ansi_escapes(input);
        // Then: códigos removidos; texto visível preservado
        assert_eq!(result, "Erro: arquivo não encontrado");
    }

    #[test]
    fn test_remocao_preserva_texto_sem_escapes() {
        let input = "linha normal sem cores";
        let result = strip_ansi_escapes(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_substituicoes_encadeaveis() {
        // Given: filtro com múltiplas regras de substituição
        let mut config = default_config();
        config.replacements = vec![
            (r"foo".to_string(), "bar".to_string()),
            (r"bar".to_string(), "baz".to_string()),
        ];
        let input = "foo";
        // When: aplicadas em ordem
        let result = apply_pipeline(input, &config);
        // Then: resultado determinístico (foo -> bar -> baz)
        assert_eq!(result, "baz");
    }

    #[test]
    fn test_curto_circuito_global() {
        // Given: filtro com regra global que bate
        let mut config = default_config();
        config.match_output = vec![GlobalMatchRule {
            pattern: r"ERRO_FATAL".to_string(),
            message: "[saída suprimida: erro fatal detectado]".to_string(),
            exception: None,
        }];
        let input = "linha 1\nERRO_FATAL: sistema parou\nlinha 3";
        // When: pipeline aplicado
        let result = apply_pipeline(input, &config);
        // Then: pipeline interrompeu e emitiu mensagem configurada
        assert_eq!(result, "[saída suprimida: erro fatal detectado]");
    }

    #[test]
    fn test_excecao_de_curto_circuito() {
        // Given: regra global com exceção que também bate
        let mut config = default_config();
        config.match_output = vec![GlobalMatchRule {
            pattern: r"WARNING".to_string(),
            message: "[suprimido]".to_string(),
            exception: Some(r"WARNING: ignorável".to_string()),
        }];
        let input = "WARNING: ignorável aqui";
        // When: pipeline aplicado
        let result = apply_pipeline(input, &config);
        // Then: regra pulada, pipeline continua normalmente
        assert_eq!(result, "WARNING: ignorável aqui");
    }

    #[test]
    fn test_inclusao_e_exclusao_de_linhas() {
        // Given: filtro com padrão de exclusão
        let mut config = default_config();
        config.strip_lines_matching = vec![r"^#".to_string()];
        let input = "# comentário\ncódigo aqui\n# outro comentário\nmais código";
        // When: aplicado
        let result = apply_pipeline(input, &config);
        // Then: linhas com # removidas
        assert_eq!(result, "código aqui\nmais código");
    }

    #[test]
    fn test_keep_lines_matching() {
        let mut config = default_config();
        config.keep_lines_matching = vec![r"^fn ".to_string()];
        let input = "use std;\nfn main() {}\nlet x = 1;\nfn foo() {}";
        let result = apply_pipeline(input, &config);
        assert_eq!(result, "fn main() {}\nfn foo() {}");
    }

    #[test]
    fn test_truncamento_por_linha() {
        // Given: filtro com largura máxima por linha
        let mut config = default_config();
        config.truncate_lines_at = Some(10);
        let input = "linha curta\numa linha bem mais longa que dez";
        // When: aplicado
        let result = apply_pipeline(input, &config);
        // Then: linha truncada em 10 chars
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].len() <= 10);
        assert!(lines[1].len() <= 10);
    }

    #[test]
    fn test_limite_absoluto_de_linhas() {
        let mut config = default_config();
        config.max_lines = Some(2);
        let input = "linha1\nlinha2\nlinha3\nlinha4";
        let result = apply_pipeline(input, &config);
        assert_eq!(result.lines().count(), 2);
    }

    #[test]
    fn test_mensagem_para_saida_vazia() {
        // Given: filtro com mensagem para resultado vazio
        let mut config = default_config();
        config.strip_lines_matching = vec![r".*".to_string()];
        config.on_empty = Some("[sem resultados]".to_string());
        let input = "linha removida";
        // When: entrada produz resultado vazio
        let result = apply_pipeline(input, &config);
        // Then: mensagem emitida
        assert_eq!(result, "[sem resultados]");
    }

    #[test]
    fn test_nivel_minimo_preserva_estrutura_funcional() {
        // Pipeline sem filtros deve preservar conteúdo integral
        let config = default_config();
        let input = "fn main() {\n    println!(\"hello\");\n}";
        let result = apply_pipeline(input, &config);
        assert_eq!(result, input);
    }

    #[test]
    fn test_pipeline_deterministico() {
        let mut config = default_config();
        config.strip_ansi = true;
        config.replacements = vec![(r"\d+".to_string(), "N".to_string())];
        let input = "\x1b[32m123\x1b[0m tokens";
        let r1 = apply_pipeline(input, &config);
        let r2 = apply_pipeline(input, &config);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_head_lines() {
        let mut config = default_config();
        config.head_lines = Some(2);
        let input = "a\nb\nc\nd";
        let result = apply_pipeline(input, &config);
        assert_eq!(result, "a\nb");
    }
}
