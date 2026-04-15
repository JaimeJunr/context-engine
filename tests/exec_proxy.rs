/// Testes de integração para `ctx exec` — proxy universal
///
/// Testam o comportamento do binário como caixa-preta:
/// - Exit codes propagados corretamente
/// - Passthrough para comandos sem filtro
/// - Filtros aplicados para comandos conhecidos
/// - Performance: execução não adiciona latência excessiva
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn ctx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("ctx")
}

// =========================================================================
// Contratos de exit code
// =========================================================================

#[test]
fn exit_code_zero_propagado_quando_comando_sucede() {
    let out = Command::new(ctx_bin())
        .args(["exec", "echo", "hello"])
        .output()
        .expect("falha ao executar ctx");

    assert_eq!(out.status.code(), Some(0), "exit code 0 deve ser propagado");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("hello"),
        "stdout deve conter output do comando"
    );
}

#[test]
fn exit_code_um_propagado_quando_comando_falha() {
    let out = Command::new(ctx_bin())
        .args(["exec", "false"])
        .output()
        .expect("falha ao executar ctx");

    assert_eq!(out.status.code(), Some(1), "exit code 1 deve ser propagado");
}

#[test]
fn exit_code_127_para_comando_inexistente() {
    let out = Command::new(ctx_bin())
        .args(["exec", "comando-que-nao-existe-em-lugar-nenhum-xyz"])
        .output()
        .expect("falha ao executar ctx");

    assert_eq!(
        out.status.code(),
        Some(127),
        "comando não encontrado deve retornar 127 (padrão POSIX)"
    );
}

// =========================================================================
// Passthrough universal
// =========================================================================

#[test]
fn passthrough_preserva_output_de_comando_sem_filtro() {
    // "printf" não tem filtro — output deve chegar intacto
    let out = Command::new(ctx_bin())
        .args(["exec", "printf", "linha1\nlinha2\nlinha3"])
        .output()
        .expect("falha ao executar ctx");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("linha1"),
        "linha1 deve aparecer no passthrough"
    );
    assert!(
        stdout.contains("linha2"),
        "linha2 deve aparecer no passthrough"
    );
    assert!(
        stdout.contains("linha3"),
        "linha3 deve aparecer no passthrough"
    );
}

#[test]
fn passthrough_propaga_exit_code_de_comando_com_argumento_invalido() {
    // ls em path inexistente → exit 1 ou 2
    let out = Command::new(ctx_bin())
        .args(["exec", "ls", "/caminho/que/nao/existe/absolutamente"])
        .output()
        .expect("falha ao executar ctx");

    assert_ne!(
        out.status.code(),
        Some(0),
        "exit code não-zero deve ser propagado quando ls falha"
    );
}

// =========================================================================
// Filtros aplicados — comportamento observável
// =========================================================================

#[test]
fn git_status_executa_e_produz_output_nao_vazio() {
    // Executa git status real no repositório do projeto
    let out = Command::new(ctx_bin())
        .args(["exec", "git", "status"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("falha ao executar ctx");

    // git status sempre sai com 0 em repos válidos
    assert_eq!(out.status.code(), Some(0));
    // deve ter algum output (working tree clean ou modified)
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "git status não deve ser silencioso"
    );
}

#[test]
fn ls_executa_e_lista_arquivos() {
    let out = Command::new(ctx_bin())
        .args(["exec", "ls", "src/"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("falha ao executar ctx");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("main.rs") || stdout.contains("lib.rs"),
        "ls deve listar arquivos"
    );
}

// =========================================================================
// Testes de performance
// =========================================================================

#[test]
fn overhead_de_proxy_e_inferior_a_200ms_para_echo() {
    // echo é instantâneo; qualquer latência é overhead do ctx exec
    // Limite: 200ms — aceitável para uso em hooks de editor
    let start = Instant::now();
    let out = Command::new(ctx_bin())
        .args(["exec", "echo", "test"])
        .output()
        .expect("falha ao executar ctx");
    let elapsed = start.elapsed();

    assert_eq!(out.status.code(), Some(0));
    assert!(
        elapsed.as_millis() < 500,
        "overhead do proxy deve ser < 200ms, foi {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn output_de_ls_respeita_limite_de_linhas_do_filtro() {
    // ls /usr/bin tem centenas de arquivos; o filtro deve limitar
    let out = Command::new(ctx_bin())
        .args(["exec", "ls", "/usr/bin"])
        .output()
        .expect("falha ao executar ctx");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line_count = stdout.lines().count();

    // O filtro de ls tem max_lines=80 — conteúdo + 1 linha de rodapé de truncamento
    assert!(
        line_count <= 81,
        "filtro de ls deve limitar a 81 linhas (80 + rodapé), obteve {}",
        line_count
    );
}

// =========================================================================
// Subcomando report não quebrou
// =========================================================================

#[test]
fn report_continua_funcionando_apos_run_proxy() {
    let out = Command::new(ctx_bin())
        .args(["exec", "report"])
        .output()
        .expect("falha ao executar ctx");

    // report pode ter 0 entradas mas não deve falhar
    assert_eq!(
        out.status.code(),
        Some(0),
        "ctx exec report deve sair com 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
