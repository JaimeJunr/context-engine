use std::path::PathBuf;
use std::process::Command;

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
