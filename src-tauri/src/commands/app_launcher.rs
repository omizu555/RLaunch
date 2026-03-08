/* ============================================================
   app_launcher - アプリ起動 / 管理者実行 / ファイルの場所を開く / ウィンドウ効果
   ============================================================ */
use std::process::Command;
use tauri::Manager;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::POINT;

/// UTF-16 のヌル終端ワイド文字列に変換
#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// ShellExecuteW でファイル/フォルダ/URL を開く (ウィンドウ非表示)
#[cfg(target_os = "windows")]
fn shell_execute(verb: &str, file: &str, params: Option<&str>) -> Result<(), String> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::Win32::Foundation::HWND;
    use windows::core::PCWSTR;

    let verb_w = to_wide(verb);
    let file_w = to_wide(file);
    let params_w = params.map(|p| to_wide(p));

    let params_ptr = params_w
        .as_ref()
        .map_or(PCWSTR(std::ptr::null()), |p| PCWSTR(p.as_ptr()));

    let result = unsafe {
        ShellExecuteW(
            HWND::default(),
            PCWSTR(verb_w.as_ptr()),
            PCWSTR(file_w.as_ptr()),
            params_ptr,
            PCWSTR(std::ptr::null()),
            SW_SHOWNORMAL,
        )
    };

    if (result.0 as isize) <= 32 {
        return Err(format!("ShellExecute failed (code {})", result.0 as isize));
    }
    Ok(())
}

/// アプリケーションを通常起動 (ShellExecuteW - コマンドプロンプト非表示)
#[tauri::command]
pub fn launch_app(path: String, args: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        shell_execute("open", &path, args.as_deref())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (path, args);
        Err("launch_app is only supported on Windows".to_string())
    }
}

/// 設定ファイルの保存先パスを取得
#[tauri::command]
pub fn get_store_path(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let store_file = path.join("launcher-data.json");
    Ok(store_file.to_string_lossy().to_string())
}

/// 管理者権限でアプリケーションを起動 (ShellExecuteW runas - コマンドプロンプト非表示)
#[tauri::command]
pub fn run_as_admin(path: String, args: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        shell_execute("runas", &path, args.as_deref())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (path, args);
        Err("run_as_admin is only supported on Windows".to_string())
    }
}

/// ファイルの場所をエクスプローラーで開く
#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(format!("/select,{}", path))
        .spawn()
        .map_err(|e| format!("Failed to open location: {}", e))?;
    Ok(())
}

/// ウィンドウ背景効果を設定 (Mica / Acrylic / None)
#[tauri::command]
pub fn set_window_effect(window: tauri::WebviewWindow, effect: String) -> Result<(), String> {
    use tauri::utils::config::{WindowEffectsConfig};
    use tauri::window::{Effect, EffectState};

    let effects = match effect.as_str() {
        "mica" => Some(WindowEffectsConfig {
            effects: vec![Effect::Mica],
            state: Some(EffectState::Active),
            radius: None,
            color: None,
        }),
        "acrylic" => Some(WindowEffectsConfig {
            effects: vec![Effect::Acrylic],
            state: Some(EffectState::Active),
            radius: None,
            color: None,
        }),
        _ => None,
    };

    window
        .set_effects(effects)
        .map_err(|e| format!("Failed to set window effect: {}", e))?;
    Ok(())
}

/// WebviewWindow をラベル指定で非表示にする
#[tauri::command]
pub fn hide_webview_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}

/// マウスカーソルの現在位置を取得 (Windows)
#[tauri::command]
pub fn get_cursor_position() -> Result<(i32, i32), String> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let mut point = POINT { x: 0, y: 0 };
            GetCursorPos(&mut point)
                .map_err(|e| format!("GetCursorPos failed: {}", e))?;
            Ok((point.x, point.y))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("get_cursor_position is only supported on Windows".to_string())
    }
}

/// カーソル位置 + カーソルがいるモニターの作業領域を取得
#[derive(serde::Serialize)]
pub struct CursorMonitorInfo {
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub monitor_x: i32,
    pub monitor_y: i32,
    pub monitor_w: i32,
    pub monitor_h: i32,
}

#[tauri::command]
pub fn get_cursor_monitor_info() -> Result<CursorMonitorInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Graphics::Gdi::{
            MonitorFromPoint, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        };

        unsafe {
            let mut point = POINT { x: 0, y: 0 };
            GetCursorPos(&mut point)
                .map_err(|e| format!("GetCursorPos failed: {}", e))?;

            let hmon = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let ok = GetMonitorInfoW(hmon, &mut mi);
            if !ok.as_bool() {
                return Err("GetMonitorInfoW failed".to_string());
            }

            let work = mi.rcWork;
            Ok(CursorMonitorInfo {
                cursor_x: point.x,
                cursor_y: point.y,
                monitor_x: work.left,
                monitor_y: work.top,
                monitor_w: work.right - work.left,
                monitor_h: work.bottom - work.top,
            })
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("get_cursor_monitor_info is only supported on Windows".to_string())
    }
}

/// ディレクトリエントリ (フォルダ階層ブラウズ用)
#[derive(serde::Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: String,
    pub size: u64,
}

/// フォルダの中身を一覧取得 (フォルダ→ファイルの順、アルファベットソート)
#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<DirectoryEntry>, String> {
    use std::fs;
    use std::path::Path;

    let dir_path = Path::new(&path);
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(dir_path).map_err(|e| format!("Read dir failed: {}", e))? {
        let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // アクセス拒否などはスキップ
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();
        let extension = entry
            .path()
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let size = metadata.len();
        let is_dir = metadata.is_dir();

        entries.push(DirectoryEntry {
            name,
            path: full_path,
            is_dir,
            extension,
            size,
        });
    }

    // フォルダを先頭に、それぞれ名前順
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}
