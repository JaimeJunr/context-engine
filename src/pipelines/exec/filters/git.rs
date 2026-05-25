use crate::pipelines::exec::types::FilterConfig;

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

/// `git push` → "ok <branch> -> <remote-ref> (<short-sha>)" ou "(up to date)".
/// Reduz output do push (que vem em stderr verboso) a 1-2 linhas essenciais.
pub fn push() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        match_output: vec![crate::pipelines::exec::types::GlobalMatchRule {
            pattern: r"Everything up-to-date".to_string(),
            message: "ok (up to date)".to_string(),
            exception: None,
        }],
        // Mantém apenas a linha de destino e linhas com refspec abc1234..def5678
        keep_lines_matching: vec![
            r"->".to_string(),
            r"\* \[new (branch|tag)\]".to_string(),
            r"\[rejected\]".to_string(),
            r"^\s*[0-9a-f]{7,40}\.\.[0-9a-f]{7,40}".to_string(),
        ],
        preprocess: Some(reduce_push_to_ok),
        max_lines: Some(3),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `git pull` → "ok <files-changed>" ou "(already up to date)".
pub fn pull() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        match_output: vec![crate::pipelines::exec::types::GlobalMatchRule {
            pattern: r"Already up.to.date".to_string(),
            message: "ok (already up to date)".to_string(),
            exception: None,
        }],
        keep_lines_matching: vec![
            r"Fast-forward".to_string(),
            r"file[s]? changed".to_string(),
            r"CONFLICT".to_string(),
            r"Merge".to_string(),
            r"->".to_string(),
        ],
        max_lines: Some(5),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `git add` → silencioso (output vazio = sucesso) ou erro se houver.
pub fn add() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(10),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `git commit` → "ok <short-sha> <subject>" ou erro.
pub fn commit() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        preprocess: Some(reduce_commit_to_ok),
        // Após o preprocess, mantém só a linha reescrita "ok ..." ou mensagens de erro.
        keep_lines_matching: vec![
            r"^ok\s".to_string(),
            r"nothing to commit".to_string(),
            r"error:".to_string(),
            r"fatal:".to_string(),
        ],
        max_lines: Some(3),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

fn reduce_push_to_ok(input: &str) -> String {
    // Procura linha com refspec "abc1234..def5678 branch -> remote/branch"
    for line in input.lines() {
        let l = line.trim();
        if l.contains("->") && !l.starts_with('*') {
            return format!("ok {}", l);
        }
    }
    input.to_string()
}

fn reduce_commit_to_ok(input: &str) -> String {
    // Linha característica: "[branch abc1234] mensagem"
    let re = regex::Regex::new(r"^\[([^\s\]]+)\s+([0-9a-f]{7,})\]\s+(.+)$").unwrap();
    for line in input.lines() {
        if let Some(caps) = re.captures(line.trim()) {
            return format!("ok {} {}: {}", &caps[2], &caps[1], &caps[3]);
        }
    }
    input.to_string()
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
    use crate::pipelines::exec::pipeline::apply_pipeline;

    #[test]
    fn git_push_reduzido_a_ok_com_refspec() {
        let input = "\
Enumerating objects: 5, done.
Counting objects: 100% (5/5), done.
Delta compression using up to 8 threads
Compressing objects: 100% (3/3), done.
Writing objects: 100% (3/3), 280 bytes | 280.00 KiB/s, done.
Total 3 (delta 2), reused 0 (delta 0), pack-reused 0
To github.com:user/repo.git
   abc1234..def5678  main -> main";
        let out = apply_pipeline(input, &push());
        assert!(
            out.contains("abc1234..def5678") && out.contains("->"),
            "deve mostrar refspec essencial: {}",
            out
        );
        assert!(
            out.lines().count() <= 3,
            "deve reduzir drasticamente: {} linhas",
            out.lines().count()
        );
    }

    #[test]
    fn git_push_up_to_date_curto_circuito() {
        let input = "Everything up-to-date";
        let out = apply_pipeline(input, &push());
        assert_eq!(out, "ok (up to date)");
    }

    #[test]
    fn git_pull_already_up_to_date_curto_circuito() {
        let input = "Already up to date.";
        let out = apply_pipeline(input, &pull());
        assert_eq!(out, "ok (already up to date)");
    }

    #[test]
    fn git_pull_fast_forward_mantem_resumo() {
        let input = "\
remote: Enumerating objects: 12, done.
remote: Counting objects: 100% (12/12), done.
Unpacking objects: 100% (10/10), done.
From github.com:user/repo
   abc1234..def5678  main       -> origin/main
Updating abc1234..def5678
Fast-forward
 README.md | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)";
        let out = apply_pipeline(input, &pull());
        assert!(out.contains("Fast-forward") || out.contains("file changed"));
        assert!(out.lines().count() <= 5);
    }

    #[test]
    fn git_commit_reduzido_a_ok_sha_subject() {
        let input = "\
[main abc1234d] feat: adiciona suporte a OAuth
 3 files changed, 42 insertions(+), 8 deletions(-)
 create mode 100644 src/auth/oauth.rs";
        let out = apply_pipeline(input, &commit());
        assert!(
            out.starts_with("ok abc1234d main: feat: adiciona suporte a OAuth"),
            "esperava reescrita ok-style: {}",
            out
        );
    }

    #[test]
    fn git_commit_nothing_to_commit_preserva_mensagem() {
        let input = "On branch main\nnothing to commit, working tree clean";
        let out = apply_pipeline(input, &commit());
        assert!(out.contains("nothing to commit"));
    }

    #[test]
    fn git_add_vazio_reporta_ok() {
        let out = apply_pipeline("", &add());
        assert_eq!(out, "ok");
    }
    use crate::pipelines::exec::registry::lookup;

    #[test]
    fn git_log_remove_commits_de_merge() {
        let input = include_str!("../../../../tests/fixtures/git_log_with_merges.txt");
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
        let input = include_str!("../../../../tests/fixtures/git_diff_whitespace.txt");
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
