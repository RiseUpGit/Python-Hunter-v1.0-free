mod blacklist;
mod db;

use db::DbState;
use std::sync::Mutex;
use sysinfo::{Pid, System};
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_mb: f64,
    pub cmdline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub process_count: usize,
    pub total_memory_gb: f64,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("python-hunter.db");
            let conn = db::open_db(db_path.to_str().unwrap())?;
            let db_state = DbState(Mutex::new(conn));

            app.manage(db_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_sysinfo,
            list_processes,
            find_python_processes,
            kill_process,
            db_add_kill_record,
            db_list_kill_records,
            db_add_blacklist_entry,
            db_list_blacklist,
            db_remove_blacklist_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn debug_sysinfo() -> SystemInfo {
    let mut system = System::new_all();
    system.refresh_all();
    SystemInfo {
        process_count: system.processes().len(),
        total_memory_gb: system.total_memory() as f64 / (1024.0 * 1024.0),
    }
}

#[tauri::command]
fn list_processes(limit: Option<usize>) -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_all();
    let mut processes: Vec<ProcessInfo> = system
        .processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string(),
            memory_mb: p.memory() as f64 / 1024.0,
            cmdline: p.cmd().join(" "),
        })
        .collect();

    processes.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap());
    processes.truncate(limit.unwrap_or(50));
    processes
}

#[tauri::command]
fn find_python_processes() -> Vec<ProcessInfo> {
    let mut system = System::new_all();
    system.refresh_all();
    system
        .processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32(),
            name: p.name().to_string(),
            memory_mb: p.memory() as f64 / 1024.0,
            cmdline: p.cmd().join(" "),
        })
        .filter(|p| p.name.to_lowercase().contains("python"))
        .collect()
}

#[tauri::command]
fn kill_process(pid: u32) -> KillResult {
    let mut system = System::new_all();
    system.refresh_all();
    let pid = Pid::from_u32(pid);
    if let Some(process) = system.process(pid) {
        let name = process.name().to_string();
        match process.kill() {
            true => KillResult {
                success: true,
                message: format!("Killed {} (PID {})", name, pid),
            },
            false => KillResult {
                success: false,
                message: format!("Failed to kill {} (PID {})", name, pid),
            },
        }
    } else {
        KillResult {
            success: false,
            message: format!("Process {} not found", pid),
        }
    }
}

#[tauri::command]
fn db_add_kill_record(state: tauri::State<'_, DbState>, record: db::KillRecord) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO kill_records (timestamp, pid, name, memory_mb, method, command_line, reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            record.timestamp,
            record.pid,
            record.name,
            record.memory_mb,
            record.method,
            record.command_line,
            record.reason,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn db_list_kill_records(state: tauri::State<'_, DbState>, limit: Option<u32>) -> Vec<db::KillRecord> {
    let conn = match state.0.lock() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        "SELECT timestamp, pid, name, memory_mb, method, command_line, reason FROM kill_records ORDER BY timestamp DESC LIMIT ?1"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = match stmt.query_map([limit.unwrap_or(200)], |row| {
        Ok(db::KillRecord {
            timestamp: row.get(0)?,
            pid: row.get(1)?,
            name: row.get(2)?,
            memory_mb: row.get(3)?,
            method: row.get(4)?,
            command_line: row.get(5)?,
            reason: row.get(6)?,
        })
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut records = Vec::new();
    for row in rows {
        if let Ok(r) = row {
            records.push(r);
        }
    }
    records
}

#[tauri::command]
fn db_add_blacklist_entry(state: tauri::State<'_, DbState>, entry: db::BlacklistEntry) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO blacklist (name, reason) VALUES (?1, ?2)",
        rusqlite::params![entry.name, entry.reason],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn db_list_blacklist(state: tauri::State<'_, DbState>) -> Vec<db::BlacklistEntry> {
    let conn = match state.0.lock() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare("SELECT name, reason FROM blacklist") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = match stmt.query_map([], |row| {
        Ok(db::BlacklistEntry {
            name: row.get(0)?,
            reason: row.get(1)?,
        })
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    for row in rows {
        if let Ok(e) = row {
            entries.push(e);
        }
    }
    entries
}

#[tauri::command]
fn db_remove_blacklist_entry(state: tauri::State<'_, DbState>, name: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM blacklist WHERE name = ?1", [&name])
        .map_err(|e| e.to_string())?;
    Ok(())
}
