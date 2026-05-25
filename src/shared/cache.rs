use anyhow::Result;
use dirs::home_dir;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(&hasher.finalize()[..16])
}

static DB: Mutex<Option<Connection>> = Mutex::new(None);
pub static NO_CACHE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn db_path() -> std::path::PathBuf {
    home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("context_engine")
        .join("cache.db")
}

fn with_conn<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&Connection) -> Result<T>,
{
    let mut guard = DB.lock().unwrap();
    if guard.is_none() {
        let path = db_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cache (key TEXT PRIMARY KEY, sigs TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS cache_refs (key TEXT PRIMARY KEY, refs TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS map_cache (key TEXT PRIMARY KEY, result TEXT NOT NULL, created_at INTEGER NOT NULL);",
        )?;
        *guard = Some(conn);
    }
    f(guard.as_ref().unwrap())
}

pub fn cache_key(path: &Path) -> String {
    let mtime = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let raw = format!("{}:{}", path.display(), mtime);
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..10])
}

pub fn cache_get(key: &str) -> Option<Vec<String>> {
    if NO_CACHE.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    with_conn(|conn| {
        let row: Option<String> = conn
            .query_row("SELECT sigs FROM cache WHERE key=?1", params![key], |r| {
                r.get(0)
            })
            .ok();
        Ok(row.and_then(|s| serde_json::from_str(&s).ok()))
    })
    .ok()
    .flatten()
}

pub fn cache_set(key: &str, sigs: &[String]) {
    let json = serde_json::to_string(sigs).unwrap_or_default();
    let _ = with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO cache VALUES (?1, ?2)",
            params![key, json],
        )?;
        Ok(())
    });
}

pub fn cache_get_refs(key: &str) -> Option<Vec<String>> {
    if NO_CACHE.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    with_conn(|conn| {
        let row: Option<String> = conn
            .query_row(
                "SELECT refs FROM cache_refs WHERE key=?1",
                params![key],
                |r| r.get(0),
            )
            .ok();
        Ok(row.and_then(|s| serde_json::from_str(&s).ok()))
    })
    .ok()
    .flatten()
}

pub fn cache_set_refs(key: &str, refs: &[String]) {
    let json = serde_json::to_string(refs).unwrap_or_default();
    let _ = with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO cache_refs VALUES (?1, ?2)",
            params![key, json],
        )?;
        Ok(())
    });
}

/// Retorna chave representando o estado git do repo nos dirs fornecidos.
/// Se repo limpo: SHA do commit HEAD. Se dirty ou não-git: "dirty-<hash>".
pub fn git_commit_key(dirs: &[String]) -> String {
    if dirs.is_empty() {
        return "no-dirs".to_string();
    }
    let dir = &dirs[0];

    // Tentar obter SHA do HEAD
    let head_output = std::process::Command::new("git")
        .args(["-C", dir, "rev-parse", "HEAD"])
        .output();

    if let Ok(out) = head_output {
        if out.status.success() {
            let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();

            // Verificar se repo está limpo
            let status_output = std::process::Command::new("git")
                .args(["-C", dir, "status", "--porcelain"])
                .output();

            if let Ok(status) = status_output {
                if status.stdout.is_empty() {
                    return sha;
                }
            }
        }
    }

    // Dirty ou não-git: hash dos primeiros 32 bytes de dirs.join(",")
    let raw = dirs.join(",");
    let bytes = &raw.as_bytes()[..raw.len().min(32)];
    format!("dirty-{}", sha256_hex(bytes))
}

/// Busca resultado cacheado do mapa. Retorna None se NO_CACHE, se não existir, ou se erro.
pub fn map_cache_get(key: &str) -> Option<String> {
    if NO_CACHE.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    with_conn(|conn| {
        let row: Option<String> = conn
            .query_row(
                "SELECT result FROM map_cache WHERE key=?1",
                params![key],
                |r| r.get(0),
            )
            .ok();
        Ok(row)
    })
    .ok()
    .flatten()
}

/// Salva resultado do mapa no cache. Silenciosamente ignora erros e NO_CACHE.
pub fn map_cache_set(key: &str, result: &str) {
    if NO_CACHE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let _ = with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO map_cache VALUES (?1, ?2, ?3)",
            params![key, result, now],
        )?;
        Ok(())
    });
}

pub fn batch_write(items: Vec<(String, Vec<String>)>, refs_items: Vec<(String, Vec<String>)>) {
    let _ = with_conn(|conn| {
        let tx = conn.unchecked_transaction()?;
        for (key, sigs) in &items {
            let json = serde_json::to_string(sigs).unwrap_or_default();
            tx.execute(
                "INSERT OR REPLACE INTO cache VALUES (?1, ?2)",
                params![key, json],
            )?;
        }
        for (key, refs) in &refs_items {
            let json = serde_json::to_string(refs).unwrap_or_default();
            tx.execute(
                "INSERT OR REPLACE INTO cache_refs VALUES (?1, ?2)",
                params![key, json],
            )?;
        }
        tx.commit()?;
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_cache_roundtrip() {
        NO_CACHE.store(false, std::sync::atomic::Ordering::Relaxed);
        let key = "test_roundtrip_key_unique_abc123";
        map_cache_set(key, "output_value");
        assert_eq!(map_cache_get(key), Some("output_value".to_string()));
    }

    #[test]
    fn test_map_cache_miss_returns_none() {
        NO_CACHE.store(false, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(map_cache_get("nonexistent_key_xyz_987654"), None);
    }

    #[test]
    fn test_git_commit_key_non_git_dir() {
        let result = git_commit_key(&["/tmp".to_string()]);
        // /tmp não é um repo git, deve retornar "dirty-..."
        assert!(
            result.starts_with("dirty-") || result.len() == 40,
            "esperado 'dirty-...' ou SHA, obtido: {}",
            result
        );
    }

    #[test]
    fn test_git_commit_key_deterministic() {
        let dirs = vec!["/tmp".to_string()];
        let r1 = git_commit_key(&dirs);
        let r2 = git_commit_key(&dirs);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_git_commit_key_empty_dirs() {
        assert_eq!(git_commit_key(&[]), "no-dirs");
    }
}
