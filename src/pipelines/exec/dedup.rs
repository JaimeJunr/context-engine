// Utilitários de deduplicação de linhas em outputs verbosos.
//
// Estratégia inspirada no `log_cmd.rs` do RTK:
// 1. Normaliza tokens variáveis (timestamps, UUIDs, hex, paths, IDs numéricos)
//    para que linhas que diferem só nesses tokens sejam considerados "iguais".
// 2. Agrupa linhas iguais consecutivas (ou em janela curta) num único item
//    anotado com `(×N)`.
//
// Resultado: logs de 5k linhas com a mesma mensagem repetida viram poucas
// linhas legíveis, preservando o sinal.

use regex::Regex;

/// Substitui tokens variáveis por placeholders para fins de comparação.
///
/// Heurísticas (ordem importa — mais específicas primeiro):
/// - Timestamps ISO 8601 / RFC 3339 → `<ts>`
/// - Timestamps em formato comum de log (`2024-01-15 12:34:56`, `15/Jan/2024:...`) → `<ts>`
/// - UUIDs → `<uuid>`
/// - Hashes hex longos (8+ chars) → `<hex>`
/// - Endereços IP → `<ip>`
/// - Paths absolutos → `<path>`
/// - Números longos (5+ dígitos, ex: PIDs, ports altos, contadores) → `<n>`
pub fn normalize_log_tokens(line: &str) -> String {
    use std::sync::OnceLock;
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            // ISO 8601 / RFC 3339 com Z ou offset
            (
                Regex::new(
                    r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?",
                )
                .unwrap(),
                "<ts>",
            ),
            // 15/Jan/2024:12:34:56 +0000
            (
                Regex::new(r"\d{1,2}/[A-Za-z]{3}/\d{4}:\d{2}:\d{2}:\d{2}(?:\s[+-]\d{4})?").unwrap(),
                "<ts>",
            ),
            // Jan 15 12:34:56
            (
                Regex::new(r"[A-Za-z]{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}").unwrap(),
                "<ts>",
            ),
            // UUID
            (
                Regex::new(
                    r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
                )
                .unwrap(),
                "<uuid>",
            ),
            // IPs v4
            (
                Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(?::\d+)?\b").unwrap(),
                "<ip>",
            ),
            // Hex longo (commit SHA, request IDs)
            (Regex::new(r"\b[0-9a-fA-F]{8,40}\b").unwrap(), "<hex>"),
            // Paths absolutos *nix
            (
                Regex::new(r"/[A-Za-z0-9_./-]+/[A-Za-z0-9_.-]+").unwrap(),
                "<path>",
            ),
            // Números longos (>= 5 dígitos)
            (Regex::new(r"\b\d{5,}\b").unwrap(), "<n>"),
        ]
    });

    let mut result = line.to_string();
    for (re, placeholder) in patterns {
        result = re.replace_all(&result, *placeholder).to_string();
    }
    result
}

/// Agrupa linhas consecutivas que normalizam para a mesma chave.
/// A primeira ocorrência é preservada literalmente; ocorrências subsequentes
/// somam num contador anotado como `(×N)` à linha original.
///
/// Em vez de strict consecutivas, usa janela: se a linha normaliza igual a
/// alguma das últimas `WINDOW` linhas distintas vistas, agrupa.
pub fn group_repeated(input: &str) -> String {
    const WINDOW: usize = 4;
    let mut out: Vec<(String, String, usize)> = Vec::new(); // (raw, normalized, count)
    for line in input.lines() {
        let norm = normalize_log_tokens(line);
        // Procura nas últimas WINDOW entradas se há match.
        let start = out.len().saturating_sub(WINDOW);
        if let Some((_, _, count)) = out[start..].iter_mut().rev().find(|(_, n, _)| n == &norm) {
            *count += 1;
        } else {
            out.push((line.to_string(), norm, 1));
        }
    }
    out.into_iter()
        .map(|(raw, _, count)| {
            if count > 1 {
                format!("{} (×{})", raw, count)
            } else {
                raw
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_timestamp_iso() {
        let line = "2024-01-15T12:34:56.789Z INFO starting up";
        assert_eq!(normalize_log_tokens(line), "<ts> INFO starting up");
    }

    #[test]
    fn normaliza_uuid() {
        let line = "request 550e8400-e29b-41d4-a716-446655440000 processed";
        assert_eq!(normalize_log_tokens(line), "request <uuid> processed");
    }

    #[test]
    fn normaliza_hex_longo_mas_preserva_curto() {
        assert_eq!(normalize_log_tokens("commit a1b2c3d4e5f6"), "commit <hex>");
        // 6 hex curtos podem ser valores legítimos — preservamos
        assert_eq!(normalize_log_tokens("color a1b"), "color a1b");
    }

    #[test]
    fn normaliza_ip_e_porta() {
        let line = "connected to 192.168.1.10:8080";
        assert_eq!(normalize_log_tokens(line), "connected to <ip>");
    }

    #[test]
    fn normaliza_path_absoluto() {
        let line = "loading /etc/myapp/config.yaml";
        assert_eq!(normalize_log_tokens(line), "loading <path>");
    }

    #[test]
    fn normaliza_numero_longo() {
        let line = "pid 28475 starting";
        assert_eq!(normalize_log_tokens(line), "pid <n> starting");
    }

    #[test]
    fn agrupa_linhas_repetidas_apenas_no_tempo() {
        // 5 linhas idênticas exceto timestamp e UUID viram 1 com (×5).
        let logs = "\
2024-01-15T12:00:01Z INFO request 11111111-1111-1111-1111-111111111111 ok
2024-01-15T12:00:02Z INFO request 22222222-2222-2222-2222-222222222222 ok
2024-01-15T12:00:03Z INFO request 33333333-3333-3333-3333-333333333333 ok
2024-01-15T12:00:04Z INFO request 44444444-4444-4444-4444-444444444444 ok
2024-01-15T12:00:05Z INFO request 55555555-5555-5555-5555-555555555555 ok";
        let out = group_repeated(logs);
        assert!(out.contains("(×5)"), "esperava (×5), obteve: {}", out);
        assert_eq!(out.lines().count(), 1, "deveria virar 1 linha");
    }

    #[test]
    fn preserva_linhas_realmente_diferentes() {
        let logs = "\
INFO server starting
INFO listening on port 8080
ERROR connection refused
INFO retrying";
        let out = group_repeated(logs);
        assert_eq!(out.lines().count(), 4, "linhas distintas não devem agrupar");
    }

    #[test]
    fn agrupa_intercaladas_dentro_da_janela() {
        // A e B alternadas (com timestamps reais para serem normalizadas) —
        // janela=4 detecta repetição.
        let logs = "\
2024-01-15T12:00:01Z msg A
2024-01-15T12:00:02Z msg B
2024-01-15T12:00:03Z msg A
2024-01-15T12:00:04Z msg B";
        let out = group_repeated(logs);
        // A normaliza diferente de B, então agrupa A com A e B com B.
        assert!(out.contains("(×2)"), "esperava (×2), obteve:\n{}", out);
    }

    #[test]
    fn input_vazio_nao_quebra() {
        assert_eq!(group_repeated(""), "");
    }

    #[test]
    fn linhas_realmente_unicas_sem_anotacao() {
        let logs = "linha A\nlinha B\nlinha C";
        let out = group_repeated(logs);
        assert!(
            !out.contains("(×"),
            "linhas únicas não devem ganhar contagem"
        );
    }
}
