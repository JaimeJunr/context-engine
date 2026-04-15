use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn ctx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("ctx")
}

fn fixtures_dir() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn test_text_output_contains_repo_map() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "Fix login timeout",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "8192",
        ])
        .output()
        .expect("failed to run ctx");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "ctx exited with error: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("<repo_map>"), "missing <repo_map> tag");
    assert!(stdout.contains("</repo_map>"), "missing </repo_map> tag");
    assert!(
        stdout.contains("sample.rb") || stdout.contains("sample.py"),
        "no fixture files in output"
    );
}

#[test]
fn test_ruby_signatures_extracted() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "user session login",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "8192",
        ])
        .output()
        .expect("failed to run ctx");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("class UserSession"), "missing Ruby class");
    assert!(stdout.contains("belongs_to"), "missing Rails macro");
}

#[test]
fn test_python_signatures_extracted() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login authenticate python",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "8192",
        ])
        .output()
        .expect("failed to run ctx");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("class LoginService"),
        "missing Python class"
    );
}

#[test]
fn test_typescript_signatures_extracted() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "typescript session controller auth",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "8192",
        ])
        .output()
        .expect("failed to run ctx");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("interface AuthSession") || stdout.contains("class SessionController"),
        "missing TS types"
    );
}

#[test]
fn test_json_output_format() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "4096",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run ctx");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("JSON output is not valid JSON");
    assert!(parsed.is_array(), "JSON output should be an array");
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty(), "JSON array should not be empty");
    assert!(arr[0]["path"].is_string(), "each entry should have a path");
    assert!(
        arr[0]["score"].is_number(),
        "each entry should have a score"
    );
    assert!(
        arr[0]["signatures"].is_array(),
        "each entry should have signatures"
    );
}

#[test]
fn test_top_n_limits_results() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login session",
            "--dirs",
            &fixtures_dir(),
            "--top",
            "1",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run ctx");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1, "top=1 should return exactly 1 file");
}

#[test]
fn test_no_cache_flag() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "2048",
            "--no-cache",
        ])
        .output()
        .expect("failed to run ctx");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("<repo_map>"),
        "should still produce output with --no-cache"
    );
}

// =========================================================================
// Testes de relevância: o arquivo certo aparece no topo?
// =========================================================================

#[test]
fn arquivo_mais_relevante_para_query_aparece_primeiro_no_json() {
    // Query "login authenticate" → Python LoginService deve vencer TokenManager
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login authenticate user",
            "--dirs",
            &fixtures_dir(),
            "--format",
            "json",
            "--top",
            "0",
        ])
        .output()
        .expect("falha ao executar ctx");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("JSON inválido");
    let arr = parsed.as_array().unwrap();

    // O primeiro resultado deve ser o arquivo mais relevante para a query
    let first_path = arr[0]["path"].as_str().unwrap_or("");
    assert!(
        first_path.contains("sample.py") || first_path.contains("sample.rb"),
        "arquivo com LoginService/UserSession deve liderar para query de login, obteve: {}",
        first_path
    );
}

#[test]
fn budget_de_tokens_e_respeitado() {
    let max_tokens: usize = 500;
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            &max_tokens.to_string(),
        ])
        .output()
        .expect("falha ao executar ctx");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Aproximação: 1 token ≈ 4 chars
    let approx_tokens = stdout.len() / 4;
    assert!(
        approx_tokens <= max_tokens * 2, // margem de 2x dado que é aproximação
        "output de ~{} tokens deve respeitar budget de {} tokens",
        approx_tokens,
        max_tokens
    );
}

#[test]
fn scores_json_estao_em_ordem_decrescente() {
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "session login authenticate",
            "--dirs",
            &fixtures_dir(),
            "--format",
            "json",
            "--top",
            "0",
        ])
        .output()
        .expect("falha ao executar ctx");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("JSON inválido");
    let arr = parsed.as_array().unwrap();

    if arr.len() > 1 {
        for i in 1..arr.len() {
            let prev = arr[i - 1]["score"].as_f64().unwrap_or(0.0);
            let curr = arr[i]["score"].as_f64().unwrap_or(0.0);
            assert!(
                prev >= curr,
                "scores devem ser decrescentes: pos {} ({}) < pos {} ({})",
                i - 1,
                prev,
                i,
                curr
            );
        }
    }
}

// =========================================================================
// Testes de performance
// =========================================================================

#[test]
fn ctx_map_em_fixtures_termina_em_menos_de_2_segundos() {
    let start = Instant::now();
    let out = Command::new(ctx_bin())
        .args([
            "map",
            "--title",
            "login",
            "--dirs",
            &fixtures_dir(),
            "--max-tokens",
            "4096",
            "--no-cache",
        ])
        .output()
        .expect("falha ao executar ctx");
    let elapsed = start.elapsed();

    assert!(out.status.success());
    assert!(
        elapsed.as_secs() < 2,
        "ctx map deve terminar em < 2s para fixtures pequenas, foi {}ms",
        elapsed.as_millis()
    );
}

// =========================================================================
// Testes de garbage collection (cleanup de documentos deletados)
// =========================================================================

#[test]
fn test_catalog_cleanup_removes_deleted_files() {
    use std::io::Write;

    // Setup: usar /tmp para diretório de teste
    let temp_base = PathBuf::from("/tmp/ctx_cleanup_test");
    let _ = fs::remove_dir_all(&temp_base); // Limpar se existir
    fs::create_dir_all(&temp_base).expect("falha ao criar temp_base");

    let file1_path = temp_base.join("file1.md");
    let file2_path = temp_base.join("file2.md");

    // Criar 2 arquivos
    let mut file1 = std::fs::File::create(&file1_path).expect("falha ao criar file1");
    file1
        .write_all(b"# File 1\nConteudo unico 1\n")
        .expect("falha ao escrever file1");

    let mut file2 = std::fs::File::create(&file2_path).expect("falha ao criar file2");
    file2
        .write_all(b"# File 2\nConteudo unico 2\n")
        .expect("falha ao escrever file2");

    let test_collection = format!(
        "test_cleanup_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let test_collection_ref = test_collection.as_str();
    let temp_base_ref = temp_base.to_string_lossy().into_owned();

    // 1. Adicionar coleção com 2 arquivos
    let add_out = Command::new(ctx_bin())
        .args([
            "add",
            test_collection_ref,
            "-s",
            &temp_base_ref,
            "--include",
            "**/*.md",
        ])
        .output()
        .expect("falha ao executar ctx add");

    assert!(
        add_out.status.success(),
        "ctx add falhou: {}",
        String::from_utf8_lossy(&add_out.stderr)
    );

    // 2. Indexar (deve indexar 2 documentos)
    let index_out = Command::new(ctx_bin())
        .args(["index", test_collection_ref])
        .output()
        .expect("falha ao executar ctx index");

    let index_stdout = String::from_utf8_lossy(&index_out.stdout);
    assert!(
        index_stdout.contains("2 varridos") && index_stdout.contains("2 indexados"),
        "esperava 2 documentos indexados, output: {}",
        index_stdout
    );

    // 3. Verificar que temos 2 documentos no status
    let status_out = Command::new(ctx_bin())
        .args(["status", test_collection_ref])
        .output()
        .expect("falha ao executar ctx status");

    let status_stdout = String::from_utf8_lossy(&status_out.stdout);
    assert!(
        status_stdout.contains("Documentos:") && status_stdout.contains("2"),
        "esperava 2 documentos, output: {}",
        status_stdout
    );

    // 4. Deletar file1.md
    fs::remove_file(&file1_path).expect("falha ao deletar file1");

    // 5. Reindexar (deve detectar e remover file1)
    let reindex_out = Command::new(ctx_bin())
        .args(["index", test_collection_ref])
        .output()
        .expect("falha ao executar ctx index novamente");

    let reindex_stdout = String::from_utf8_lossy(&reindex_out.stdout);
    // Esperamos: 1 arquivo varrido (file2)
    assert!(
        reindex_stdout.contains("1 varrido"),
        "esperava 1 arquivo varrido apos deletar, output: {}",
        reindex_stdout
    );

    // 6. Verificar que temos apenas 1 documento no status
    let final_status = Command::new(ctx_bin())
        .args(["status", test_collection_ref])
        .output()
        .expect("falha ao executar ctx status final");

    let final_status_str = String::from_utf8_lossy(&final_status.stdout);
    assert!(
        final_status_str.contains("Documentos:") && final_status_str.contains("1"),
        "esperava 1 documento apos cleanup, output: {}",
        final_status_str
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_base);
}

// =========================================================================
// Testes de bootstrap
// =========================================================================

#[test]
fn bootstrap_descobre_apenas_arquivos_md_e_ignora_rs() {
    use context_engine::catalog;
    use std::io::Write;

    let temp_base = PathBuf::from(format!(
        "/tmp/ctx_bootstrap_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    let _ = fs::remove_dir_all(&temp_base);
    fs::create_dir_all(&temp_base).expect("falha ao criar temp dir");

    // Cria 2 arquivos .md e 1 .rs
    let mut f1 = std::fs::File::create(temp_base.join("README.md")).unwrap();
    f1.write_all(b"# Titulo\nConteudo relevante para busca.\n")
        .unwrap();

    let mut f2 = std::fs::File::create(temp_base.join("guide.md")).unwrap();
    f2.write_all(b"## Guia\nPassos para instalar.\n").unwrap();

    let mut f3 = std::fs::File::create(temp_base.join("main.rs")).unwrap();
    f3.write_all(b"fn main() { println!(\"hello\"); }\n")
        .unwrap();

    let stats = catalog::bootstrap(&temp_base, None).expect("bootstrap não deve falhar");

    assert_eq!(
        stats.files_discovered, 2,
        "bootstrap deve descobrir apenas os 2 arquivos .md, encontrou {}",
        stats.files_discovered
    );

    // Verifica que a collection existe listando
    let collections = catalog::list_collections().unwrap();
    let names: Vec<_> = collections.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        names.contains(&stats.collection_name.as_str()),
        "collection '{}' deve existir após bootstrap",
        stats.collection_name
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_base);
}
