use anyhow::Result;
use dirs::home_dir;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

static DB: Mutex<Option<Connection>> = Mutex::new(None);
pub static NO_CACHE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

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
             CREATE TABLE IF NOT EXISTS cache_refs (key TEXT PRIMARY KEY, refs TEXT NOT NULL);",
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
