use anyhow::Result;
use dirs::home_dir;
use rusqlite::{params, Connection};
use std::sync::Mutex;

use super::types::{Chunk, ChunkStatus, Collection, CollectionHealth, Document, PathContext};

static DB: Mutex<Option<Connection>> = Mutex::new(None);

fn db_path() -> std::path::PathBuf {
    home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("context_engine")
        .join("catalog.db")
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
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        migrate(&conn)?;
        *guard = Some(conn);
    }
    f(guard.as_ref().unwrap())
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS collections (
            name         TEXT PRIMARY KEY,
            config_json  TEXT NOT NULL,
            last_indexed TEXT
        );

        CREATE TABLE IF NOT EXISTS documents (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            collection   TEXT NOT NULL,
            path         TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            indexed_at   TEXT NOT NULL,
            UNIQUE(collection, path)
        );

        CREATE TABLE IF NOT EXISTS chunks (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            doc_id       INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            content      TEXT NOT NULL,
            start_offset INTEGER NOT NULL,
            embedding    BLOB,
            status       TEXT NOT NULL DEFAULT 'pending'
        );

        CREATE TABLE IF NOT EXISTS path_contexts (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            collection   TEXT NOT NULL,
            path_prefix  TEXT NOT NULL,
            description  TEXT NOT NULL,
            priority     INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS op_cache (
            input_hash   TEXT NOT NULL,
            op_type      TEXT NOT NULL,
            result_json  TEXT NOT NULL,
            created_at   TEXT NOT NULL,
            PRIMARY KEY(input_hash, op_type)
        );
        ",
    )?;
    Ok(())
}

// --- Collections ---

pub fn upsert_collection(col: &Collection) -> Result<()> {
    let json = serde_json::to_string(col)?;
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO collections (name, config_json) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET config_json = excluded.config_json",
            params![col.name, json],
        )?;
        Ok(())
    })
}

pub fn get_collection(name: &str) -> Result<Option<Collection>> {
    with_conn(|conn| {
        let row: Option<String> = conn
            .query_row(
                "SELECT config_json FROM collections WHERE name = ?1",
                params![name],
                |r| r.get(0),
            )
            .ok();
        Ok(row.and_then(|s| serde_json::from_str(&s).ok()))
    })
}

pub fn list_collections() -> Result<Vec<(String, Option<String>)>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT name, last_indexed FROM collections ORDER BY name")?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

pub fn set_last_indexed(name: &str, ts: &str) -> Result<()> {
    with_conn(|conn| {
        conn.execute(
            "UPDATE collections SET last_indexed = ?1 WHERE name = ?2",
            params![ts, name],
        )?;
        Ok(())
    })
}

// --- Documents ---

pub fn get_document(collection: &str, path: &str) -> Result<Option<Document>> {
    with_conn(|conn| {
        let row = conn
            .query_row(
                "SELECT id, collection, path, content_hash, indexed_at
                 FROM documents WHERE collection = ?1 AND path = ?2",
                params![collection, path],
                |r| {
                    Ok(Document {
                        id: r.get(0)?,
                        collection: r.get(1)?,
                        path: r.get(2)?,
                        content_hash: r.get(3)?,
                        indexed_at: r.get(4)?,
                    })
                },
            )
            .ok();
        Ok(row)
    })
}

pub fn upsert_document(collection: &str, path: &str, hash: &str, indexed_at: &str) -> Result<i64> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO documents (collection, path, content_hash, indexed_at) VALUES (?1,?2,?3,?4)
             ON CONFLICT(collection, path) DO UPDATE SET
               content_hash = excluded.content_hash,
               indexed_at   = excluded.indexed_at",
            params![collection, path, hash, indexed_at],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM documents WHERE collection = ?1 AND path = ?2",
            params![collection, path],
            |r| r.get(0),
        )?;
        Ok(id)
    })
}

pub fn list_documents(collection: &str) -> Result<Vec<Document>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, collection, path, content_hash, indexed_at
             FROM documents WHERE collection = ?1",
        )?;
        let rows = stmt
            .query_map(params![collection], |r| {
                Ok(Document {
                    id: r.get(0)?,
                    collection: r.get(1)?,
                    path: r.get(2)?,
                    content_hash: r.get(3)?,
                    indexed_at: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

pub fn delete_document(doc_id: i64) -> Result<()> {
    with_conn(|conn| {
        conn.execute("DELETE FROM documents WHERE id = ?1", params![doc_id])?;
        Ok(())
    })
}

// --- Chunks ---

pub fn insert_chunks(chunks: &[(i64, &str, usize)]) -> Result<()> {
    with_conn(|conn| {
        let tx = conn.unchecked_transaction()?;
        for (doc_id, content, offset) in chunks {
            tx.execute(
                "INSERT INTO chunks (doc_id, content, start_offset, status) VALUES (?1,?2,?3,'pending')",
                params![doc_id, content, *offset as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    })
}

pub fn delete_chunks_for_doc(doc_id: i64) -> Result<()> {
    with_conn(|conn| {
        conn.execute("DELETE FROM chunks WHERE doc_id = ?1", params![doc_id])?;
        Ok(())
    })
}

pub fn get_pending_chunks(collection: &str, limit: usize) -> Result<Vec<Chunk>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT c.id, c.doc_id, c.content, c.start_offset, c.embedding, c.status
             FROM chunks c
             JOIN documents d ON d.id = c.doc_id
             WHERE d.collection = ?1 AND c.status = 'pending'
             ORDER BY c.id ASC
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![collection, limit as i64], |r| {
                let emb_blob: Option<Vec<u8>> = r.get(4)?;
                Ok(Chunk {
                    id: r.get(0)?,
                    doc_id: r.get(1)?,
                    content: r.get(2)?,
                    start_offset: r.get::<_, i64>(3)? as usize,
                    embedding: emb_blob.map(blob_to_f32),
                    status: ChunkStatus::parse_status(&r.get::<_, String>(5)?),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

pub fn get_all_chunks_with_embeddings(collection: &str) -> Result<Vec<Chunk>> {
    with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT c.id, c.doc_id, c.content, c.start_offset, c.embedding, c.status
             FROM chunks c
             JOIN documents d ON d.id = c.doc_id
             WHERE d.collection = ?1 AND c.status = 'done' AND c.embedding IS NOT NULL",
        )?;
        let rows = stmt
            .query_map(params![collection], |r| {
                let emb_blob: Option<Vec<u8>> = r.get(4)?;
                Ok(Chunk {
                    id: r.get(0)?,
                    doc_id: r.get(1)?,
                    content: r.get(2)?,
                    start_offset: r.get::<_, i64>(3)? as usize,
                    embedding: emb_blob.map(blob_to_f32),
                    status: ChunkStatus::parse_status(&r.get::<_, String>(5)?),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    })
}

pub fn update_chunk_embedding(chunk_id: i64, embedding: &[f32]) -> Result<()> {
    let blob = f32_to_blob(embedding);
    with_conn(|conn| {
        conn.execute(
            "UPDATE chunks SET embedding = ?1, status = 'done' WHERE id = ?2",
            params![blob, chunk_id],
        )?;
        Ok(())
    })
}

pub fn get_doc_path(doc_id: i64) -> Result<Option<String>> {
    with_conn(|conn| {
        let row = conn
            .query_row(
                "SELECT path FROM documents WHERE id = ?1",
                params![doc_id],
                |r| r.get(0),
            )
            .ok();
        Ok(row)
    })
}

pub fn count_pending_chunks(collection: &str) -> Result<usize> {
    with_conn(|conn| {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chunks c
             JOIN documents d ON d.id = c.doc_id
             WHERE d.collection = ?1 AND c.status = 'pending'",
            params![collection],
            |r| r.get(0),
        )?;
        Ok(count as usize)
    })
}

// --- Path contexts ---

pub fn upsert_path_context(collection: &str, ctx: &PathContext) -> Result<()> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO path_contexts (collection, path_prefix, description, priority)
             VALUES (?1,?2,?3,?4)
             ON CONFLICT DO NOTHING",
            params![collection, ctx.path_prefix, ctx.description, ctx.priority],
        )?;
        Ok(())
    })
}

// Retorna contexto mais específico para um caminho (RD-19)
pub fn get_context_for_path(collection: &str, path: &str) -> Result<Option<String>> {
    with_conn(|conn| {
        // Seleciona contexto com o prefix mais longo que seja prefixo do path
        let mut stmt = conn.prepare(
            "SELECT description FROM path_contexts
             WHERE collection = ?1 AND ?2 LIKE (path_prefix || '%')
             ORDER BY LENGTH(path_prefix) DESC, priority DESC
             LIMIT 1",
        )?;
        let row = stmt
            .query_row(params![collection, path], |r| r.get::<_, String>(0))
            .ok();
        Ok(row)
    })
}

// --- Op cache ---

pub fn cache_get_op(input_hash: &str, op_type: &str) -> Result<Option<String>> {
    with_conn(|conn| {
        let row = conn
            .query_row(
                "SELECT result_json FROM op_cache WHERE input_hash = ?1 AND op_type = ?2",
                params![input_hash, op_type],
                |r| r.get(0),
            )
            .ok();
        Ok(row)
    })
}

pub fn cache_set_op(
    input_hash: &str,
    op_type: &str,
    result_json: &str,
    created_at: &str,
) -> Result<()> {
    with_conn(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO op_cache (input_hash, op_type, result_json, created_at)
             VALUES (?1,?2,?3,?4)",
            params![input_hash, op_type, result_json, created_at],
        )?;
        Ok(())
    })
}

// --- Health report ---

pub fn collection_health(name: &str) -> Result<CollectionHealth> {
    with_conn(|conn| {
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE collection = ?1",
                params![name],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let pending: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chunks c
                 JOIN documents d ON d.id = c.doc_id
                 WHERE d.collection = ?1 AND c.status = 'pending'",
                params![name],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let last_indexed: Option<String> = conn
            .query_row(
                "SELECT last_indexed FROM collections WHERE name = ?1",
                params![name],
                |r| r.get(0),
            )
            .ok();

        Ok(CollectionHealth {
            name: name.to_string(),
            total_documents: total as usize,
            pending_embeddings: pending as usize,
            last_indexed,
            consistent: pending == 0,
        })
    })
}

// --- Compact (RD-30) ---

pub fn compact() -> Result<()> {
    with_conn(|conn| {
        // Remove orphan chunks (documentos deletados sem cascade aplicado)
        conn.execute_batch(
            "DELETE FROM chunks WHERE doc_id NOT IN (SELECT id FROM documents);
             VACUUM;",
        )?;
        Ok(())
    })
}

// --- Helpers de serialização de vetores ---

fn f32_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_f32(b: Vec<u8>) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
