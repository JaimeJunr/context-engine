// Store SQLite para o grafo de símbolos.
//
// Schema:
//   symbols(id, name, qualified, kind, file, line, language)
//     PK: id (autoincrement)
//     UNIQUE: qualified
//     INDEX: name, file
//
//   calls(id, caller_qualified, callee_name, file, line)
//     INDEX: caller_qualified, callee_name
//
//   imports(id, file, module, alias)
//     INDEX: file, module
//
// Localização: ~/.cache/context_engine/graph.db (separado do cache de map/catalog).

use anyhow::Result;
use rusqlite::{params, Connection};

use super::types::{CallSite, Symbol, SymbolKind};

/// Abre conexão com o store no caminho default (`~/.cache/context_engine/graph.db`).
pub fn open_default() -> Result<Connection> {
    let path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("não foi possível determinar HOME"))?
        .join(".cache")
        .join("context_engine")
        .join("graph.db");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Cria/migra schema do grafo. Idempotente.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS symbols (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL,
            qualified    TEXT NOT NULL UNIQUE,
            kind         TEXT NOT NULL,
            file         TEXT NOT NULL,
            line         INTEGER NOT NULL,
            language     TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
        CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file);

        CREATE TABLE IF NOT EXISTS calls (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            caller_qualified  TEXT NOT NULL,
            callee_name       TEXT NOT NULL,
            file              TEXT NOT NULL,
            line              INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_calls_caller ON calls(caller_qualified);
        CREATE INDEX IF NOT EXISTS idx_calls_callee ON calls(callee_name);

        CREATE TABLE IF NOT EXISTS imports (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            file    TEXT NOT NULL,
            module  TEXT NOT NULL,
            alias   TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_imports_file ON imports(file);
        CREATE INDEX IF NOT EXISTS idx_imports_module ON imports(module);
        ",
    )?;
    Ok(())
}

/// Limpa todos os dados de um arquivo antes de re-indexar (idempotência).
pub fn clear_file(conn: &Connection, file: &str) -> Result<()> {
    conn.execute("DELETE FROM symbols WHERE file = ?1", params![file])?;
    conn.execute("DELETE FROM calls WHERE file = ?1", params![file])?;
    conn.execute("DELETE FROM imports WHERE file = ?1", params![file])?;
    Ok(())
}

/// Insere símbolo (ignora se já existe pelo `qualified`).
pub fn insert_symbol(conn: &Connection, sym: &Symbol) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO symbols (name, qualified, kind, file, line, language)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            sym.name,
            sym.qualified,
            sym.kind.as_str(),
            sym.file,
            sym.line,
            sym.language,
        ],
    )?;
    Ok(())
}

/// Insere call site.
pub fn insert_call(conn: &Connection, call: &CallSite) -> Result<()> {
    conn.execute(
        "INSERT INTO calls (caller_qualified, callee_name, file, line)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            call.caller_qualified,
            call.callee_name,
            call.file,
            call.line
        ],
    )?;
    Ok(())
}

/// Insere import.
pub fn insert_import(
    conn: &Connection,
    file: &str,
    module: &str,
    alias: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO imports (file, module, alias) VALUES (?1, ?2, ?3)",
        params![file, module, alias],
    )?;
    Ok(())
}

/// Busca símbolo por nome simples — pode retornar múltiplos (overload, mesmo nome em módulos).
pub fn find_symbols_by_name(conn: &Connection, name: &str) -> Result<Vec<Symbol>> {
    let mut stmt = conn.prepare(
        "SELECT name, qualified, kind, file, line, language
         FROM symbols WHERE name = ?1",
    )?;
    let rows = stmt.query_map(params![name], |row| {
        let kind_str: String = row.get(2)?;
        Ok(Symbol {
            name: row.get(0)?,
            qualified: row.get(1)?,
            kind: SymbolKind::parse(&kind_str).unwrap_or(SymbolKind::Function),
            file: row.get(3)?,
            line: row.get(4)?,
            language: row.get(5)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Lista call sites onde `callee_name` é chamado.
pub fn find_callers(conn: &Connection, callee_name: &str) -> Result<Vec<CallSite>> {
    let mut stmt = conn.prepare(
        "SELECT caller_qualified, callee_name, file, line
         FROM calls WHERE callee_name = ?1",
    )?;
    let rows = stmt.query_map(params![callee_name], |row| {
        Ok(CallSite {
            caller_qualified: row.get(0)?,
            callee_name: row.get(1)?,
            file: row.get(2)?,
            line: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Lista call sites originados em `caller_qualified` (o que ela chama).
pub fn find_callees(conn: &Connection, caller_qualified: &str) -> Result<Vec<CallSite>> {
    let mut stmt = conn.prepare(
        "SELECT caller_qualified, callee_name, file, line
         FROM calls WHERE caller_qualified = ?1",
    )?;
    let rows = stmt.query_map(params![caller_qualified], |row| {
        Ok(CallSite {
            caller_qualified: row.get(0)?,
            callee_name: row.get(1)?,
            file: row.get(2)?,
            line: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(name: &str, qualified: &str, kind: SymbolKind, file: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
            qualified: qualified.to_string(),
            kind,
            file: file.to_string(),
            line: 1,
            language: "rust".to_string(),
        }
    }

    #[test]
    fn migrate_e_idempotente() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }

    #[test]
    fn insere_e_busca_simbolo_por_nome() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        insert_symbol(&conn, &sym("foo", "mod::foo", SymbolKind::Function, "a.rs")).unwrap();
        insert_symbol(
            &conn,
            &sym("foo", "other::foo", SymbolKind::Function, "b.rs"),
        )
        .unwrap();

        let found = find_symbols_by_name(&conn, "foo").unwrap();
        assert_eq!(found.len(), 2, "deveria encontrar ambos os símbolos");
    }

    #[test]
    fn callers_retorna_quem_chamou() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        insert_call(
            &conn,
            &CallSite {
                caller_qualified: "main".to_string(),
                callee_name: "foo".to_string(),
                file: "main.rs".to_string(),
                line: 10,
            },
        )
        .unwrap();
        insert_call(
            &conn,
            &CallSite {
                caller_qualified: "other".to_string(),
                callee_name: "foo".to_string(),
                file: "other.rs".to_string(),
                line: 20,
            },
        )
        .unwrap();

        let callers = find_callers(&conn, "foo").unwrap();
        assert_eq!(callers.len(), 2);
        assert!(callers.iter().any(|c| c.caller_qualified == "main"));
        assert!(callers.iter().any(|c| c.caller_qualified == "other"));
    }

    #[test]
    fn callees_retorna_o_que_uma_funcao_chama() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        insert_call(
            &conn,
            &CallSite {
                caller_qualified: "main".to_string(),
                callee_name: "foo".to_string(),
                file: "main.rs".to_string(),
                line: 10,
            },
        )
        .unwrap();
        insert_call(
            &conn,
            &CallSite {
                caller_qualified: "main".to_string(),
                callee_name: "bar".to_string(),
                file: "main.rs".to_string(),
                line: 11,
            },
        )
        .unwrap();

        let callees = find_callees(&conn, "main").unwrap();
        assert_eq!(callees.len(), 2);
    }

    #[test]
    fn clear_file_remove_simbolos_e_calls() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        insert_symbol(&conn, &sym("foo", "mod::foo", SymbolKind::Function, "a.rs")).unwrap();
        insert_call(
            &conn,
            &CallSite {
                caller_qualified: "main".to_string(),
                callee_name: "foo".to_string(),
                file: "a.rs".to_string(),
                line: 1,
            },
        )
        .unwrap();

        clear_file(&conn, "a.rs").unwrap();

        assert!(find_symbols_by_name(&conn, "foo").unwrap().is_empty());
        assert!(find_callers(&conn, "foo").unwrap().is_empty());
    }
}
