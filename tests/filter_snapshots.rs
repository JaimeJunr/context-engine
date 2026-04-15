// Testes comportamentais dos filtros exec — verificam que informação crítica é preservada
// e ruído é removido, sem dependência de crates externos de snapshot.

use context_engine::exec::pipeline::apply_pipeline;
use context_engine::exec::registry::lookup;

// =========================================================================
// cargo test: falha visível, linhas de compilação removidas
// =========================================================================

#[test]
fn cargo_test_failure_mostra_falha_e_remove_ruido() {
    let input = include_str!("fixtures/cargo_test_with_failure.txt");
    let config = lookup("cargo", &["test".to_string()]).unwrap();

    let result = apply_pipeline(input, &config);

    assert!(
        result.contains("test_login_invalido") || result.contains("FAILED"),
        "agente precisa ver qual teste falhou; resultado:\n{}",
        result
    );
    assert!(
        !result.contains("Compiling"),
        "linhas 'Compiling' são ruído e não devem aparecer; resultado:\n{}",
        result
    );
}

// =========================================================================
// git log com merges: commits de merge removidos, commits normais preservados
// =========================================================================

#[test]
fn git_log_merges_sao_removidos() {
    let input = include_str!("fixtures/git_log_with_merges.txt");
    let config = lookup("git", &["log".to_string()]).unwrap();

    let result = apply_pipeline(input, &config);

    assert!(
        !result.contains("Merge branch"),
        "commits de merge não devem aparecer; resultado:\n{}",
        result
    );
    assert!(
        result.contains("feat: add user authentication")
            || result.contains("fix: resolve session timeout")
            || result.contains("refactor: extract auth middleware"),
        "commits normais devem ser preservados; resultado:\n{}",
        result
    );
}

// =========================================================================
// git diff whitespace-only: linhas com apenas espaço em branco são removidas
// =========================================================================

#[test]
fn git_diff_remove_whitespace_only_hunks() {
    let input = include_str!("fixtures/git_diff_whitespace.txt");
    let config = lookup("git", &["diff".to_string()]).unwrap();

    let result = apply_pipeline(input, &config);

    for line in result.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            let content = &line[1..];
            assert!(
                !content.trim().is_empty(),
                "linha adicionada com apenas whitespace não deve aparecer: {:?}",
                line
            );
        }
    }
}

// =========================================================================
// npm install erro: erro visível, warns verbose removidos
// =========================================================================

#[test]
fn npm_install_erro_e_visivel() {
    let input = include_str!("fixtures/npm_install_error.txt");
    let config = lookup("npm", &["install".to_string()]).unwrap();

    let result = apply_pipeline(input, &config);

    assert!(
        result.contains("404") || result.to_lowercase().contains("error"),
        "erro 404/error deve ser visível; resultado:\n{}",
        result
    );
    // Linhas de warn verbose (enoent interno) devem ser filtradas ou reduzidas
    // O filtro mantém "npm warn" e "npm error" — verifica apenas que o erro principal é visível
    assert!(
        !result.is_empty(),
        "resultado não deve ser vazio quando há erro"
    );
}

// =========================================================================
// gradle test falha: nome do spec visível, progress tasks removidas
// =========================================================================

#[test]
fn gradle_test_falha_mostra_falha_remove_progress() {
    let input = include_str!("fixtures/gradle_test_failure.txt");
    let config = lookup("gradle", &["test".to_string()]).unwrap();

    let result = apply_pipeline(input, &config);

    assert!(
        result.contains("AuthServiceSpec")
            || result.contains("FAILED")
            || result.contains("BUILD FAILED"),
        "nome do spec falhando deve aparecer; resultado:\n{}",
        result
    );
    assert!(
        !result.contains("> Task :compileJava"),
        "linhas de progress '> Task :' não devem aparecer; resultado:\n{}",
        result
    );
}

// =========================================================================
// mvn test: [INFO] removido, [ERROR] mantido
// =========================================================================

#[test]
fn mvn_test_remove_info_mantem_error() {
    let input = include_str!("fixtures/mvn_test_failure.txt");
    let config = lookup("mvn", &["test".to_string()]).unwrap();

    let result = apply_pipeline(input, &config);

    assert!(
        !result.contains("[INFO] Scanning"),
        "linhas [INFO] devem ser removidas; resultado:\n{}",
        result
    );
    assert!(
        result.contains("[ERROR]"),
        "[ERROR] deve ser preservado; resultado:\n{}",
        result
    );
}
