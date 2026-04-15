use crate::exec::types::FilterConfig;

pub fn status() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(50),
        on_empty: Some("(working tree clean)".to_string()),
        ..Default::default()
    }
}

pub fn log() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        head_lines: Some(30),
        truncate_lines_at: Some(120),
        preprocess: Some(preprocess_git_log),
        ..Default::default()
    }
}

pub fn diff() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(150),
        strip_lines_matching: vec![
            r"^Binary files".to_string(),
            r"^\+\s*$".to_string(),
            r"^-\s*$".to_string(),
        ],
        ..Default::default()
    }
}

pub fn show() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(150),
        strip_lines_matching: vec![
            r"^Binary files".to_string(),
            r"^\+\s*$".to_string(),
            r"^-\s*$".to_string(),
            r"^index [0-9a-f]+\.\.[0-9a-f]+".to_string(),
        ],
        ..Default::default()
    }
}

pub fn blame() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(100),
        truncate_lines_at: Some(100),
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

/// Remove commits de merge e limita a 50 commits do input de `git log`
fn preprocess_git_log(input: &str) -> String {
    // Divide em blocos: cada bloco começa com "commit <hash>"
    let mut blocks: Vec<&str> = Vec::new();
    let mut current_start = 0;
    let chars = input.char_indices().peekable();

    for (i, _) in chars {
        // Verifica se estamos no início de uma linha que começa com "commit "
        let is_line_start = i == 0 || input.as_bytes().get(i.wrapping_sub(1)) == Some(&b'\n');
        if is_line_start && input[i..].starts_with("commit ") {
            if i > current_start {
                blocks.push(&input[current_start..i]);
            }
            current_start = i;
        }
    }
    // Último bloco
    if current_start < input.len() {
        blocks.push(&input[current_start..]);
    }

    // Filtra blocos de merge
    let non_merge: Vec<&str> = blocks
        .into_iter()
        .filter(|block| {
            !block
                .lines()
                .any(|l| l.starts_with("Merge:") || l.starts_with("Merge branch"))
        })
        .collect();

    // Limita a 50 commits
    non_merge.into_iter().take(50).collect::<Vec<_>>().join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::pipeline::apply_pipeline;
    use crate::exec::registry::lookup;

    #[test]
    fn git_log_remove_commits_de_merge() {
        let input = include_str!("../../../tests/fixtures/git_log_with_merges.txt");
        let config = log();
        let result = apply_pipeline(input, &config);

        assert!(
            !result.contains("Merge branch"),
            "commits de merge não devem aparecer no output filtrado; resultado:\n{}",
            result
        );
        // Commits normais devem aparecer
        assert!(
            result.contains("feat: add user authentication")
                || result.contains("fix: resolve session timeout"),
            "commits normais devem ser preservados"
        );
    }

    #[test]
    fn git_log_limita_50_commits() {
        // Gera fixture com 60 commits
        let mut input = String::new();
        for i in 0..60 {
            input.push_str(&format!(
                "commit {:040x}\nAuthor: User <u@e.com>\nDate: Mon Apr 14 10:00:00 2025 +0000\n\n    commit number {}\n\n",
                i, i
            ));
        }
        let _config = log();
        // Aplica apenas o preprocess para contar blocos
        let preprocessed = preprocess_git_log(&input);
        let commit_count = preprocessed
            .lines()
            .filter(|l| l.starts_with("commit "))
            .count();

        assert!(
            commit_count <= 50,
            "deve limitar a 50 commits, obteve {}",
            commit_count
        );
    }

    #[test]
    fn git_diff_remove_whitespace_only() {
        let input = include_str!("../../../tests/fixtures/git_diff_whitespace.txt");
        let config = diff();
        let result = apply_pipeline(input, &config);

        // Linhas adicionadas com apenas whitespace (ou vazias) não devem aparecer
        for line in result.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                let content = &line[1..];
                assert!(
                    !content.trim().is_empty(),
                    "linha com apenas whitespace não deve aparecer: {:?}",
                    line
                );
            }
        }
    }

    #[test]
    fn git_blame_tem_filtro_registrado() {
        let result = lookup("git", &["blame".to_string()]);
        assert!(
            result.is_some(),
            "git blame deve ter filtro registrado no registry"
        );
        // Verifica que é o filtro específico de blame (com truncate_lines_at)
        let config = result.unwrap();
        assert_eq!(
            config.truncate_lines_at,
            Some(100),
            "filtro de blame deve ter truncate_lines_at=100"
        );
    }

    #[test]
    fn preprocess_field_e_chamado_no_pipeline() {
        let config = FilterConfig {
            preprocess: Some(|s| s.to_uppercase()),
            ..Default::default()
        };
        let result = apply_pipeline("hello world", &config);
        assert_eq!(
            result, "HELLO WORLD",
            "preprocess deve ser chamado antes dos estágios de filtragem"
        );
    }
}
