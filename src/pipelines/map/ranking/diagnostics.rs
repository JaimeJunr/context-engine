use std::path::PathBuf;

/// Padrões de nomes de arquivo que indicam configuração/boilerplate
const CONFIG_PATTERNS: &[&str] = &[
    "Config",
    "Bootstrap",
    "Application.",
    ".config.ts",
    ".config.js",
    ".config.mjs",
    ".module.ts",
    "build.gradle",
    "build.gradle.kts",
    "pom.xml",
    "webpack.config.",
    "vite.config.",
    "jest.config.",
    "application.rb",
    "application.properties",
    "application.yml",
];

fn is_config_file(path: &std::path::Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    CONFIG_PATTERNS.iter().any(|pat| name.contains(pat))
}

/// Detecta quando o resultado é dominado por arquivos de config/boilerplate
/// e retorna uma sugestão para usar --seeds.
///
/// Threshold: se `count(config) / min(top_n, ranked.len()) > 0.60`
pub fn boilerplate_hint(ranked: &[(PathBuf, f64)], top_n: usize) -> Option<String> {
    let window = top_n.min(ranked.len());
    if window == 0 {
        return None;
    }

    let config_count = ranked[..window]
        .iter()
        .filter(|(path, _)| is_config_file(path))
        .count();

    // Threshold estritamente maior que 60%
    if config_count as f64 / window as f64 > 0.60 {
        Some(
            "hint: resultado dominado por arquivos de configuração. Tente --seeds <diretório de domínio>"
                .to_string(),
        )
    } else {
        None
    }
}

/// Detecta quando os top-N scores estão muito próximos (baixa variância),
/// sugerindo que a query pode estar muito genérica.
pub fn low_variance_hint(ranked: &[(PathBuf, f64)], top_n: usize) -> Option<String> {
    let window = &ranked[..top_n.min(ranked.len())];

    if window.len() < 3 {
        return None;
    }

    let max_score = window.iter().map(|(_, s)| *s).fold(f64::NAN, f64::max);
    if max_score <= 0.05 {
        return None; // scores todos muito baixos — corpus pequeno ou query sem match
    }

    let mean = window.iter().map(|(_, s)| s).sum::<f64>() / window.len() as f64;
    let variance =
        window.iter().map(|(_, s)| (s - mean).powi(2)).sum::<f64>() / (window.len() - 1) as f64;

    if variance < 0.001 {
        Some(
            "hint: scores muito similares — tente uma query mais específica ou use --seeds"
                .to_string(),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ranked(names: &[&str]) -> Vec<(PathBuf, f64)> {
        names.iter().map(|n| (PathBuf::from(n), 1.0)).collect()
    }

    #[test]
    fn hint_quando_maioria_e_config() {
        // 7 de 10 são config → 70% > 60% → Some(hint)
        let ranked = make_ranked(&[
            "AppConfig.ts",
            "vite.config.js",
            "jest.config.ts",
            "webpack.config.js",
            "application.rb",
            "application.properties",
            "Bootstrap.java",
            "main.rs",
            "lib.rs",
            "service.rs",
        ]);
        let result = boilerplate_hint(&ranked, 10);
        assert!(
            result.is_some(),
            "deve retornar hint quando maioria é config (7/10)"
        );
        assert!(result.unwrap().contains("--seeds"));
    }

    #[test]
    fn sem_hint_quando_minoria_e_config() {
        // 2 de 10 são config → 20% → None
        let ranked = make_ranked(&[
            "main.rs",
            "lib.rs",
            "service.rs",
            "controller.rs",
            "model.rs",
            "handler.rs",
            "router.rs",
            "db.rs",
            "vite.config.js",
            "application.rb",
        ]);
        let result = boilerplate_hint(&ranked, 10);
        assert!(
            result.is_none(),
            "não deve retornar hint quando minoria é config (2/10)"
        );
    }

    #[test]
    fn threshold_exato_60_pct_nao_e_hint() {
        // 6 de 10 = 60% — threshold é > 60%, portanto 60% não dispara
        let ranked = make_ranked(&[
            "AppConfig.ts",
            "vite.config.js",
            "jest.config.ts",
            "webpack.config.js",
            "application.rb",
            "application.properties",
            "main.rs",
            "lib.rs",
            "service.rs",
            "model.rs",
        ]);
        let result = boilerplate_hint(&ranked, 10);
        assert!(
            result.is_none(),
            "60% exato não deve disparar hint (threshold é >60%)"
        );
    }

    // =========================================================================
    // Testes para low_variance_hint
    // =========================================================================

    #[test]
    fn hint_quando_scores_muito_similares() {
        // variância ≈ 0 (todos scores iguais), max > 0.05 → Some
        let ranked: Vec<(PathBuf, f64)> = (0..5)
            .map(|i| (PathBuf::from(format!("file{}.rs", i)), 0.5))
            .collect();
        let result = low_variance_hint(&ranked, 5);
        assert!(
            result.is_some(),
            "deve retornar hint quando scores são idênticos"
        );
        assert!(result.unwrap().contains("--seeds"));
    }

    #[test]
    fn sem_hint_quando_scores_variados() {
        // variância alta — scores bem distintos
        let ranked = vec![
            (PathBuf::from("a.rs"), 0.9),
            (PathBuf::from("b.rs"), 0.5),
            (PathBuf::from("c.rs"), 0.1),
            (PathBuf::from("d.rs"), 0.05),
            (PathBuf::from("e.rs"), 0.01),
        ];
        let result = low_variance_hint(&ranked, 5);
        assert!(
            result.is_none(),
            "não deve retornar hint quando scores têm alta variância"
        );
    }

    #[test]
    fn sem_hint_quando_janela_pequena() {
        // 2 elementos → None (limiar é < 3)
        let ranked = vec![(PathBuf::from("a.rs"), 0.5), (PathBuf::from("b.rs"), 0.5)];
        let result = low_variance_hint(&ranked, 2);
        assert!(
            result.is_none(),
            "janela com menos de 3 elementos não deve retornar hint"
        );
    }

    #[test]
    fn sem_hint_quando_todos_scores_zero() {
        // max_score = 0.0 ≤ 0.05 → None (corpus pequeno ou sem match)
        let ranked: Vec<(PathBuf, f64)> = (0..5)
            .map(|i| (PathBuf::from(format!("file{}.rs", i)), 0.0))
            .collect();
        let result = low_variance_hint(&ranked, 5);
        assert!(
            result.is_none(),
            "não deve retornar hint quando todos os scores são zero"
        );
    }

    #[test]
    fn threshold_61_pct_e_hint() {
        // 7 de 10 ≈ 70% > 60% → Some
        // Já coberto pelo primeiro teste, mas validamos explicitamente a fronteira
        let ranked = make_ranked(&[
            "AppConfig.ts",
            "vite.config.js",
            "jest.config.ts",
            "webpack.config.js",
            "application.rb",
            "application.properties",
            "Bootstrap.java",
            "main.rs",
            "lib.rs",
            "service.rs",
        ]);
        let result = boilerplate_hint(&ranked, 10);
        assert!(result.is_some(), "70% deve disparar hint (>60% threshold)");
    }
}
