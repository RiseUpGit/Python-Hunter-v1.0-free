mod blacklist;
mod db;

use db::DbState;
use std::sync::Mutex;
use sysinfo::{Pid, System};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_mb: f64,
    pub cmdline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDetail {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub cmdline: String,
    pub exe_path: String,
    pub start_time: u64,
    pub status: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonInfo {
    pub script_path: String,
    pub working_dir: String,
    pub venv_path: String,
    pub is_venv: bool,
    pub python_version: String,
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
            // Process
            debug_sysinfo,
            list_processes,
            find_python_processes,
            kill_process,
            process_detail,
            inspect_python,
            open_terminal,
            open_editor,
            // History
            db_add_kill_record,
            db_list_kill_records,
            db_clear_kill_records,
            db_export_kill_records,
            // Blacklist
            db_add_blacklist_entry,
            db_list_blacklist,
            db_remove_blacklist_entry,
            check_blacklist,
            auto_kill_blacklisted,
            // Rules
            export_rules,
            import_rules,
            fetch_rules,
            // Settings
            setting_set,
            setting_get,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ========== PROCESS COMMANDS ==========

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
fn process_detail(pid: u32) -> Option<ProcessDetail> {
    let mut system = System::new_all();
    system.refresh_all();
    let pid = Pid::from_u32(pid);
    system.process(pid).map(|p| ProcessDetail {
        pid: p.pid().as_u32(),
        name: p.name().to_string(),
        parent_pid: p.parent().map(|pp| pp.as_u32()),
        cpu_percent: p.cpu_usage(),
        memory_mb: p.memory() as f64 / 1024.0,
        cmdline: p.cmd().join(" "),
        exe_path: p.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
        start_time: p.start_time(),
        status: "running".to_string(),
    })
}

#[tauri::command]
fn inspect_python(pid: u32) -> Option<PythonInfo> {
    let mut system = System::new_all();
    system.refresh_all();
    let pid = Pid::from_u32(pid);
    system.process(pid).map(|p| {
        let cmd = p.cmd();
        let script_path = if cmd.len() > 1 { cmd[1].clone() } else { String::new() };
        let working_dir = p.cwd().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let exe_path = p.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let is_venv = exe_path.contains("venv") || exe_path.contains(".venv");
        let venv_path = if is_venv {
            exe_path.split("venv").next().unwrap_or("").to_string()
        } else {
            String::new()
        };
        let python_version = if exe_path.contains("python3.11") { "3.11".to_string() }
            else if exe_path.contains("python3.12") { "3.12".to_string() }
            else if exe_path.contains("python3.10") { "3.10".to_string() }
            else if exe_path.contains("python3.14") { "3.14".to_string() }
            else { String::new() };
        PythonInfo {
            script_path,
            working_dir,
            venv_path,
            is_venv,
            python_version,
        }
    })
}

#[tauri::command]
#[cfg(windows)]
fn open_terminal(path: String) -> Result<(), String> {
    use std::process::Command;
    Command::new("cmd.exe")
        .args(&["/K", "cd", "/d", &path])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[cfg(windows)]
fn open_editor(path: String) -> Result<(), String> {
    use std::process::Command;
    Command::new("cmd.exe")
        .args(&["/C", "start", "", &path])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
fn open_terminal(_path: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
#[cfg(not(windows))]
fn open_editor(_path: String) -> Result<(), String> { Ok(()) }

// ========== HISTORY COMMANDS ==========

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
fn db_clear_kill_records(state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM kill_records", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn db_export_kill_records(state: tauri::State<'_, DbState>) -> Vec<db::KillRecord> {
    db_list_kill_records(state, None)
}

// ========== BLACKLIST COMMANDS ==========

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

#[tauri::command]
fn check_blacklist(state: tauri::State<'_, DbState>, name: String) -> bool {
    let conn = match state.0.lock() {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut stmt = match conn.prepare("SELECT 1 FROM blacklist WHERE name = ?1") {
        Ok(s) => s,
        Err(_) => return false,
    };
    stmt.query_row([&name], |row| row.get::<_, i32>(0)).is_ok()
}

#[tauri::command]
fn auto_kill_blacklisted(state: tauri::State<'_, DbState>) -> Vec<KillResult> {
    let mut results = Vec::new();
    let blacklisted: Vec<String> = {
        let conn = match state.0.lock() {
            Ok(c) => c,
            Err(_) => return results,
        };
        let mut stmt = match conn.prepare("SELECT name FROM blacklist") {
            Ok(s) => s,
            Err(_) => return results,
        };
        let rows = stmt.query_map([], |row| row.get::<_, String>(0));
        match rows {
            Ok(r) => r.filter_map(|x| x.ok()).collect(),
            Err(_) => return results,
        }
    };

    let mut system = System::new_all();
    system.refresh_all();
    for name in blacklisted {
        for process in system.processes().values() {
            if process.name().to_lowercase().contains(&name.to_lowercase()) {
                let pid = process.pid().as_u32();
                match process.kill() {
                    true => results.push(KillResult {
                        success: true,
                        message: format!("Auto-killed {} (PID {})", process.name(), pid),
                    }),
                    false => results.push(KillResult {
                        success: false,
                        message: format!("Failed to auto-kill {} (PID {})", process.name(), pid),
                    }),
                }
            }
        }
    }
    results
}

// ========== RULES COMMANDS ==========

#[tauri::command]
fn export_rules(state: tauri::State<'_, DbState>) -> String {
    let blacklist = db_list_blacklist(state);
    serde_json::json!({
        "version": 1,
        "blacklist": blacklist,
    }).to_string()
}

#[tauri::command]
fn import_rules(state: tauri::State<'_, DbState>, json: String) -> Result<(), String> {
    let parsed: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if let Some(list) = parsed["blacklist"].as_array() {
        for entry in list {
            let name = entry["name"].as_str().unwrap_or("").to_string();
            let reason = entry["reason"].as_str().map(|s| s.to_string());
            if !name.is_empty() {
                db_add_blacklist_entry(state.clone(), db::BlacklistEntry { name, reason })?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn fetch_rules(url: String) -> Result<String, String> {
    ureq::get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())
}

// ========== SETTINGS COMMANDS ==========

#[tauri::command]
fn setting_set(state: tauri::State<'_, DbState>, key: String, value: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn setting_get(state: tauri::State<'_, DbState>, key: String) -> Option<String> {
    let conn = match state.0.lock() {
        Ok(c) => c,
        Err(_) => return None,
    };
    let mut stmt = match conn.prepare("SELECT value FROM settings WHERE key = ?1") {
        Ok(s) => s,
        Err(_) => return None,
    };
    stmt.query_row([&key], |row| row.get::<_, String>(0)).ok()
}
