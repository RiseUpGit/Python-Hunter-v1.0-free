use tauri_build::{AppManifest, Attributes};

fn main() {
    tauri_build::try_build(
        Attributes::default().app_manifest(
            AppManifest::new().commands(&[
                "debug_sysinfo",
                "list_processes",
                "find_python_processes",
                "kill_process",
                "db_add_kill_record",
                "db_list_kill_records",
                "db_add_blacklist_entry",
                "db_list_blacklist",
                "db_remove_blacklist_entry",
            ]),
        ),
    )
    .expect("Failed to build Tauri app");
}
