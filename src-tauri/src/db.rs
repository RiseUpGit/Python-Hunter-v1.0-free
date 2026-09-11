use serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRecord {
    pub timestamp: String,
    pub pid: u32,
    pub name: String,
    pub memory_mb: f64,
    pub method: String,
    pub command_line: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub name: String,
    pub reason: Option<String>,
}

pub struct DbState(pub Mutex<Connection>);

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS kill_records (
            timestamp TEXT NOT NULL,
            pid INTEGER NOT NULL,
            name TEXT NOT NULL,
            memory_mb REAL NOT NULL,
            method TEXT NOT NULL,
            command_line TEXT,
            reason TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS blacklist (
            name TEXT PRIMARY KEY NOT NULL,
            reason TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
    Ok(())
}

pub fn open_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    init_db(&conn)?;
    Ok(conn)
}
