/* ============================================================
   icon_library - アイコンライブラリ管理
   AppData/icons/ フォルダ内の SVG/PNG をスキャンしてアイコン一覧を提供
   ============================================================ */
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
pub struct IconInfo {
    pub filename: String,
    pub data_url: String,
}

/// アイコンフォルダのパスを取得
fn get_icons_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("icons"))
}

/// バイト列を data URL に変換
fn to_data_url(data: &[u8], media_type: &str) -> String {
    format!("data:{};base64,{}", media_type, STANDARD.encode(data))
}

/// デフォルトアイコン (SVG)
fn default_icons() -> Vec<(&'static str, &'static str)> {
    vec![
        // ── 基本シェイプ ──
        ("app.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="3" fill="#60a5fa"/></svg>"##),
        ("document.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" fill="#e2e8f0"/><path d="M14 2v6h6" fill="none" stroke="#94a3b8" stroke-width="2"/></svg>"##),
        ("folder.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" fill="#fbbf24"/></svg>"##),
        ("folder-open.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" fill="#f59e0b"/><path d="M2 10h20v9a2 2 0 01-2 2H4a2 2 0 01-2-2z" fill="#fbbf24"/></svg>"##),
        // ── ネット・通信 ──
        ("globe.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 014 10 15 15 0 01-4 10 15 15 0 01-4-10A15 15 0 0112 2z"/></svg>"##),
        ("mail.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="2" y="4" width="20" height="16" rx="2" fill="#f97316"/><path d="M22 4L12 13 2 4" fill="none" stroke="#fff7ed" stroke-width="2"/></svg>"##),
        ("chat.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" fill="#38bdf8"/></svg>"##),
        ("phone.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M22 16.92v3a2 2 0 01-2.18 2 19.79 19.79 0 01-8.63-3.07 19.5 19.5 0 01-6-6A19.79 19.79 0 012.12 4.18 2 2 0 014.11 2h3a2 2 0 012 1.72c.127.96.361 1.903.7 2.81a2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0122 16.92z" fill="#22c55e"/></svg>"##),
        ("wifi.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#60a5fa" stroke-width="2" stroke-linecap="round"><path d="M5 12.55a11 11 0 0114.08 0M1.42 9a16 16 0 0121.16 0M8.53 16.11a6 6 0 016.95 0"/><circle cx="12" cy="20" r="1" fill="#60a5fa"/></svg>"##),
        ("link.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#818cf8" stroke-width="2" stroke-linecap="round"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>"##),
        // ── 開発 ──
        ("terminal.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="2" y="3" width="20" height="18" rx="2" fill="#1e293b"/><path d="M6 8l4 4-4 4" stroke="#22d3ee" stroke-width="2" fill="none" stroke-linecap="round"/><path d="M12 16h6" stroke="#22d3ee" stroke-width="2" stroke-linecap="round"/></svg>"##),
        ("code.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#a78bfa" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 18l6-6-6-6M8 6l-6 6 6 6"/></svg>"##),
        ("database.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#f472b6" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>"##),
        ("bug.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="2" stroke-linecap="round"><rect x="8" y="6" width="8" height="14" rx="4" fill="#fecaca"/><path d="M6 10H2M22 10h-4M6 14H2M22 14h-4M6 18H2M22 18h-4M9 2l1.5 2M15 2l-1.5 2"/></svg>"##),
        // ── メディア ──
        ("music.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M9 18V5l12-2v13" fill="none" stroke="#f472b6" stroke-width="2"/><circle cx="6" cy="18" r="3" fill="#f472b6"/><circle cx="18" cy="16" r="3" fill="#f472b6"/></svg>"##),
        ("image.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2" fill="#6366f1"/><circle cx="8.5" cy="8.5" r="1.5" fill="#e0e7ff"/><path d="M21 15l-5-5L5 21" stroke="#e0e7ff" stroke-width="2" fill="none"/></svg>"##),
        ("video.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="2" y="4" width="15" height="16" rx="2" fill="#dc2626"/><path d="M17 8l5-3v14l-5-3z" fill="#fca5a5"/></svg>"##),
        ("camera.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2z" fill="#64748b"/><circle cx="12" cy="13" r="4" fill="#e2e8f0"/></svg>"##),
        ("headphones.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#8b5cf6" stroke-width="2"><path d="M3 18v-6a9 9 0 0118 0v6"/><path d="M21 19a2 2 0 01-2 2h-1a2 2 0 01-2-2v-3a2 2 0 012-2h3zM3 19a2 2 0 002 2h1a2 2 0 002-2v-3a2 2 0 00-2-2H3z" fill="#c4b5fd"/></svg>"##),
        // ── ツール・設定 ──
        ("bolt.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" fill="#facc15"/></svg>"##),
        ("shield.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" fill="#22c55e"/></svg>"##),
        ("gear.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09a1.65 1.65 0 00-1.08-1.51 1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09a1.65 1.65 0 001.51-1.08 1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001.08 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9c.26.604.852.997 1.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1.08z"/></svg>"##),
        ("wrench.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"><path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/></svg>"##),
        ("key.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#eab308" stroke-width="2" stroke-linecap="round"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.78 7.78 5.5 5.5 0 017.78-7.78zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>"##),
        // ── お気に入り・ステータス ──
        ("star.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" fill="#fbbf24"/></svg>"##),
        ("heart.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 00-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 00-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 000-7.78z" fill="#ef4444"/></svg>"##),
        ("bookmark.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" fill="#f97316"/></svg>"##),
        ("flag.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z" fill="#ef4444"/><line x1="4" y1="22" x2="4" y2="15" stroke="#94a3b8" stroke-width="2"/></svg>"##),
        // ── エンタメ ──
        ("game.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="2" y="6" width="20" height="12" rx="3" fill="#7c3aed"/><circle cx="8" cy="12" r="2" fill="#ddd6fe"/><circle cx="16" cy="12" r="2" fill="#ddd6fe"/></svg>"##),
        ("palette.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10c.83 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-1 0-.83.67-1.5 1.5-1.5H16c3.31 0 6-2.69 6-6 0-4.96-4.48-9-10-9z" fill="#e2e8f0"/><circle cx="6.5" cy="11.5" r="1.5" fill="#ef4444"/><circle cx="9.5" cy="7.5" r="1.5" fill="#f59e0b"/><circle cx="14.5" cy="7.5" r="1.5" fill="#22c55e"/><circle cx="17.5" cy="11.5" r="1.5" fill="#3b82f6"/></svg>"##),
        ("trophy.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M8 21h8m-4-4v4M6 4h12v2a6 6 0 01-12 0z" fill="none" stroke="#eab308" stroke-width="2" stroke-linecap="round"/><path d="M6 4H4a1 1 0 00-1 1v1a4 4 0 004 4m11-6h2a1 1 0 011 1v1a4 4 0 01-4 4" fill="none" stroke="#eab308" stroke-width="2"/><path d="M7 3h10v3a5 5 0 01-10 0z" fill="#fde047"/></svg>"##),
        ("dice.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="3" fill="#f1f5f9"/><circle cx="8" cy="8" r="1.5" fill="#1e293b"/><circle cx="16" cy="8" r="1.5" fill="#1e293b"/><circle cx="12" cy="12" r="1.5" fill="#1e293b"/><circle cx="8" cy="16" r="1.5" fill="#1e293b"/><circle cx="16" cy="16" r="1.5" fill="#1e293b"/></svg>"##),
        // ── 時間・カレンダー ──
        ("clock.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#64748b" stroke-width="2"><circle cx="12" cy="12" r="10" fill="#f1f5f9"/><path d="M12 6v6l4 2" stroke-linecap="round"/></svg>"##),
        ("calendar.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="18" rx="2" fill="#3b82f6"/><path d="M3 10h18" stroke="#1e40af" stroke-width="2"/><rect x="3" y="10" width="18" height="12" rx="0" fill="#eff6ff"/><path d="M8 2v4M16 2v4" stroke="#3b82f6" stroke-width="2" stroke-linecap="round"/><rect x="7" y="14" width="3" height="3" rx="0.5" fill="#3b82f6"/></svg>"##),
        ("hourglass.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2" stroke-linecap="round"><path d="M5 22h14M5 2h14M17 22v-3.87a3.37 3.37 0 00-.94-2.32L12 12l-4.06 3.81A3.37 3.37 0 007 18.13V22M17 2v3.87a3.37 3.37 0 01-.94 2.32L12 12l-4.06-3.81A3.37 3.37 0 017 5.87V2"/></svg>"##),
        // ── ロケーション・移動 ──
        ("map-pin.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z" fill="#ef4444"/><circle cx="12" cy="10" r="3" fill="#fecaca"/></svg>"##),
        ("home.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" fill="#60a5fa"/><rect x="9" y="14" width="6" height="8" fill="#bfdbfe"/></svg>"##),
        ("rocket.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#f97316" stroke-width="2" stroke-linecap="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 00-2.91-.09zM12 15l-3-3a22 22 0 012-3.95A12.88 12.88 0 0122 2c0 2.72-.78 7.5-6 11a22 22 0 01-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg>"##),
        // ── 人物・コミュニティ ──
        ("user.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#64748b" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2" fill="#e2e8f0"/><circle cx="12" cy="7" r="4" fill="#e2e8f0"/></svg>"##),
        ("users.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#64748b" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" fill="#e2e8f0"/><circle cx="9" cy="7" r="4" fill="#e2e8f0"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>"##),
        // ── その他 ──
        ("download.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>"##),
        ("trash.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="2" stroke-linecap="round"><path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>"##),
        ("search.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#64748b" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg>"##),
        ("edit.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2" stroke-linecap="round"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.12 2.12 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>"##),
        ("clipboard.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect x="8" y="2" width="8" height="4" rx="1" fill="#94a3b8"/><path d="M16 4h2a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2h2" fill="#e2e8f0" stroke="#94a3b8" stroke-width="1.5"/><path d="M9 12h6M9 16h4" stroke="#64748b" stroke-width="2" stroke-linecap="round"/></svg>"##),
        ("box.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#a78bfa" stroke-width="2"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" fill="#ede9fe"/><path d="M3.27 6.96L12 12.01l8.73-5.05M12 22.08V12"/></svg>"##),
    ]
}

/// 初回起動時: icons/ フォルダにデフォルトアイコンを書き出す
#[tauri::command]
pub fn init_icon_library(app: tauri::AppHandle) -> Result<(), String> {
    let icons_dir = get_icons_dir(&app)?;

    if !icons_dir.exists() {
        fs::create_dir_all(&icons_dir)
            .map_err(|e| format!("Failed to create icons dir: {}", e))?;
    }

    // 新しいデフォルトアイコンがあれば追加（既存ファイルはスキップ）
    for (filename, content) in default_icons() {
        let path = icons_dir.join(filename);
        if !path.exists() {
            fs::write(&path, content)
                .map_err(|e| format!("Failed to write icon {}: {}", filename, e))?;
        }
    }

    Ok(())
}

/// アイコン一覧を取得（icons/ フォルダをスキャン → data URL 付きで返す）
#[tauri::command]
pub fn list_icon_library(app: tauri::AppHandle) -> Result<Vec<IconInfo>, String> {
    let icons_dir = get_icons_dir(&app)?;

    if !icons_dir.exists() {
        return Ok(vec![]);
    }

    let mut icons: Vec<IconInfo> = Vec::new();

    let entries = fs::read_dir(&icons_dir)
        .map_err(|e| format!("Failed to read icons dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let media_type = match ext.as_str() {
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            _ => continue,
        };

        let filename = entry.file_name().to_string_lossy().to_string();

        match fs::read(&path) {
            Ok(data) => {
                let data_url = to_data_url(&data, media_type);
                icons.push(IconInfo { filename, data_url });
            }
            Err(e) => {
                eprintln!("Failed to read icon {:?}: {}", path, e);
            }
        }
    }

    // ファイル名順にソート
    icons.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));

    Ok(icons)
}

/// アイコンフォルダのパスを返す
#[tauri::command]
pub fn get_icon_library_dir_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = get_icons_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}
