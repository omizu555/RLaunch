/* ============================================================
   system_info - CPU / メモリ使用率取得
   sysinfo クレートを使用して動的情報を返す
   ============================================================ */
use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;

// グローバルに System インスタンスを保持（毎回生成すると重い）
static SYSTEM: std::sync::LazyLock<Mutex<System>> =
    std::sync::LazyLock::new(|| Mutex::new(System::new_all()));

#[derive(Serialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let mut sys = SYSTEM.lock().unwrap();

    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_usage();
    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let memory_usage = if total_memory > 0.0 {
        (used_memory / total_memory * 100.0) as f32
    } else {
        0.0
    };

    SystemInfo {
        cpu_usage,
        memory_usage,
    }
}
