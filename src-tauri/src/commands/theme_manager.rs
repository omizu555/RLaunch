/* ============================================================
   theme_manager - テーマフォルダの管理・読み込み
   themes/ フォルダ内の JSON をスキャンして動的テーマ一覧を提供
   ============================================================ */
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// テーマ情報（フロントエンドに返す）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    pub label: String,
    pub author: String,
    pub variables: HashMap<String, String>,
}

/// テーマフォルダのパスを取得
fn get_themes_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("themes"))
}

/// ビルトインテーマ定義
fn builtin_themes() -> Vec<ThemeInfo> {
    vec![
        ThemeInfo {
            id: "dark".into(),
            label: "ダーク".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#1e1e2e".into()),
                ("--bg-secondary".into(), "#181825".into()),
                ("--bg-button".into(), "#313244".into()),
                ("--bg-button-hover".into(), "#45475a".into()),
                ("--bg-button-active".into(), "#585b70".into()),
                ("--bg-button-empty".into(), "rgba(69, 71, 90, 0.25)".into()),
                ("--text-primary".into(), "#cdd6f4".into()),
                ("--text-secondary".into(), "#a6adc8".into()),
                ("--text-muted".into(), "#6c7086".into()),
                ("--border-color".into(), "#45475a".into()),
                ("--accent-color".into(), "#89b4fa".into()),
                ("--accent-hover".into(), "#74c7ec".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.4)".into()),
                ("--danger-color".into(), "#f38ba8".into()),
                ("--success-color".into(), "#a6e3a1".into()),
                ("--warning-color".into(), "#fab387".into()),
                ("--border-radius".into(), "8px".into()),
                ("--border-radius-sm".into(), "4px".into()),
            ]),
        },
        ThemeInfo {
            id: "light".into(),
            label: "ライト".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#eff1f5".into()),
                ("--bg-secondary".into(), "#e6e9ef".into()),
                ("--bg-button".into(), "#dce0e8".into()),
                ("--bg-button-hover".into(), "#ccd0da".into()),
                ("--bg-button-active".into(), "#bcc0cc".into()),
                ("--bg-button-empty".into(), "rgba(172, 176, 190, 0.25)".into()),
                ("--text-primary".into(), "#4c4f69".into()),
                ("--text-secondary".into(), "#5c5f77".into()),
                ("--text-muted".into(), "#9ca0b0".into()),
                ("--border-color".into(), "#ccd0da".into()),
                ("--accent-color".into(), "#1e66f5".into()),
                ("--accent-hover".into(), "#2a6ef5".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.12)".into()),
                ("--danger-color".into(), "#d20f39".into()),
                ("--success-color".into(), "#40a02b".into()),
                ("--warning-color".into(), "#df8e1d".into()),
            ]),
        },
        ThemeInfo {
            id: "classic".into(),
            label: "クラシック".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#d4d0c8".into()),
                ("--bg-secondary".into(), "#c0c0c0".into()),
                ("--bg-button".into(), "#d4d0c8".into()),
                ("--bg-button-hover".into(), "#e0dcd4".into()),
                ("--bg-button-active".into(), "#bab6ae".into()),
                ("--bg-button-empty".into(), "rgba(180, 176, 168, 0.35)".into()),
                ("--text-primary".into(), "#000000".into()),
                ("--text-secondary".into(), "#333333".into()),
                ("--text-muted".into(), "#808080".into()),
                ("--border-color".into(), "#a0a0a0".into()),
                ("--accent-color".into(), "#0a246a".into()),
                ("--accent-hover".into(), "#0d2d80".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.2)".into()),
                ("--danger-color".into(), "#c0392b".into()),
                ("--success-color".into(), "#27ae60".into()),
                ("--warning-color".into(), "#e67e22".into()),
                ("--border-radius".into(), "2px".into()),
                ("--border-radius-sm".into(), "1px".into()),
            ]),
        },
        // Paper White — 純白紙ベース、くっきり黒文字、ミニマル
        ThemeInfo {
            id: "paper-white".into(),
            label: "Paper White".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#ffffff".into()),
                ("--bg-secondary".into(), "#f8f9fa".into()),
                ("--bg-button".into(), "#f0f1f3".into()),
                ("--bg-button-hover".into(), "#e4e6e9".into()),
                ("--bg-button-active".into(), "#d5d8dc".into()),
                ("--bg-button-empty".into(), "rgba(0, 0, 0, 0.04)".into()),
                ("--text-primary".into(), "#1a1a1a".into()),
                ("--text-secondary".into(), "#4a4a4a".into()),
                ("--text-muted".into(), "#9e9e9e".into()),
                ("--border-color".into(), "#e0e0e0".into()),
                ("--accent-color".into(), "#2563eb".into()),
                ("--accent-hover".into(), "#1d4ed8".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.08)".into()),
                ("--danger-color".into(), "#dc2626".into()),
                ("--success-color".into(), "#16a34a".into()),
                ("--warning-color".into(), "#d97706".into()),
                ("--border-radius".into(), "6px".into()),
                ("--border-radius-sm".into(), "3px".into()),
            ]),
        },
        // Soft Cream — 暖色系クリーム、ダークブラウン文字、目に優しい
        ThemeInfo {
            id: "soft-cream".into(),
            label: "Soft Cream".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#faf6f0".into()),
                ("--bg-secondary".into(), "#f3ede4".into()),
                ("--bg-button".into(), "#ebe4d8".into()),
                ("--bg-button-hover".into(), "#e0d6c6".into()),
                ("--bg-button-active".into(), "#d4c8b4".into()),
                ("--bg-button-empty".into(), "rgba(139, 119, 90, 0.08)".into()),
                ("--text-primary".into(), "#2c2416".into()),
                ("--text-secondary".into(), "#5c4f3c".into()),
                ("--text-muted".into(), "#a09482".into()),
                ("--border-color".into(), "#d8cebe".into()),
                ("--accent-color".into(), "#b45309".into()),
                ("--accent-hover".into(), "#92400e".into()),
                ("--shadow-color".into(), "rgba(80, 60, 30, 0.1)".into()),
                ("--danger-color".into(), "#b91c1c".into()),
                ("--success-color".into(), "#3f6212".into()),
                ("--warning-color".into(), "#b45309".into()),
                ("--border-radius".into(), "8px".into()),
                ("--border-radius-sm".into(), "4px".into()),
            ]),
        },
        // Cool Silver — 青みがかったシルバー、ネイビー文字、スタイリッシュ
        ThemeInfo {
            id: "cool-silver".into(),
            label: "Cool Silver".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#f4f6f9".into()),
                ("--bg-secondary".into(), "#ebeef3".into()),
                ("--bg-button".into(), "#e1e5ed".into()),
                ("--bg-button-hover".into(), "#d3d9e4".into()),
                ("--bg-button-active".into(), "#c5ccda".into()),
                ("--bg-button-empty".into(), "rgba(70, 90, 120, 0.06)".into()),
                ("--text-primary".into(), "#1e293b".into()),
                ("--text-secondary".into(), "#475569".into()),
                ("--text-muted".into(), "#94a3b8".into()),
                ("--border-color".into(), "#cbd5e1".into()),
                ("--accent-color".into(), "#0369a1".into()),
                ("--accent-hover".into(), "#075985".into()),
                ("--shadow-color".into(), "rgba(30, 41, 59, 0.1)".into()),
                ("--danger-color".into(), "#be123c".into()),
                ("--success-color".into(), "#15803d".into()),
                ("--warning-color".into(), "#c2410c".into()),
                ("--border-radius".into(), "6px".into()),
                ("--border-radius-sm".into(), "3px".into()),
            ]),
        },
    ]
}

/// 初回起動時: themes/ フォルダにビルトインテーマを書き出す
#[tauri::command]
pub fn init_themes(app: tauri::AppHandle) -> Result<(), String> {
    let themes_dir = get_themes_dir(&app)?;

    if !themes_dir.exists() {
        fs::create_dir_all(&themes_dir)
            .map_err(|e| format!("Failed to create themes dir: {}", e))?;
    }

    // ビルトインテーマが未書き出しなら追加
    for theme in builtin_themes() {
        let path = themes_dir.join(format!("{}.json", theme.id));
        if !path.exists() {
            write_theme(&themes_dir, &theme)?;
        }
    }

    Ok(())
}

/// テーマ一覧を取得（themes/ フォルダ + リソースのサンプルテーマをスキャン）
#[tauri::command]
pub fn list_themes(app: tauri::AppHandle) -> Result<Vec<ThemeInfo>, String> {
    let themes_dir = get_themes_dir(&app)?;
    let mut themes: Vec<ThemeInfo> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // ユーザーテーマフォルダ（AppData/themes/）をスキャン
    if themes_dir.exists() {
        let entries = fs::read_dir(&themes_dir)
            .map_err(|e| format!("Failed to read themes dir: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<ThemeInfo>(&content) {
                        Ok(theme) => {
                            seen_ids.insert(theme.id.clone());
                            themes.push(theme);
                        }
                        Err(e) => eprintln!("Invalid theme file {:?}: {}", path, e),
                    },
                    Err(e) => eprintln!("Failed to read theme {:?}: {}", path, e),
                }
            }
        }
    } else {
        // フォルダ未作成ならビルトインのみ先に追加
        for theme in builtin_themes() {
            seen_ids.insert(theme.id.clone());
            themes.push(theme);
        }
    }

    // バンドルされたサンプルテーマ（resources/sample-themes/）をスキャン
    if let Ok(resource_dir) = app.path().resource_dir() {
        let sample_dir = resource_dir.join("resources").join("sample-themes");
        if sample_dir.exists() {
            if let Ok(entries) = fs::read_dir(&sample_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("json") {
                        match fs::read_to_string(&path) {
                            Ok(content) => match serde_json::from_str::<ThemeInfo>(&content) {
                                Ok(theme) => {
                                    // ユーザーテーマに同じIDがあれば上書きしない
                                    if !seen_ids.contains(&theme.id) {
                                        seen_ids.insert(theme.id.clone());
                                        themes.push(theme);
                                    }
                                }
                                Err(e) => eprintln!("Invalid sample theme {:?}: {}", path, e),
                            },
                            Err(e) => eprintln!("Failed to read sample theme {:?}: {}", path, e),
                        }
                    }
                }
            }
        }
    }

    // id でソート（builtin を先頭に）
    themes.sort_by(|a, b| {
        let a_builtin = matches!(a.id.as_str(), "dark" | "light" | "classic" | "paper-white" | "soft-cream" | "cool-silver");
        let b_builtin = matches!(b.id.as_str(), "dark" | "light" | "classic" | "paper-white" | "soft-cream" | "cool-silver");
        match (a_builtin, b_builtin) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.label.cmp(&b.label),
        }
    });

    Ok(themes)
}

/// テーマ JSON ファイルを書き出す
fn write_theme(dir: &PathBuf, theme: &ThemeInfo) -> Result<(), String> {
    let path = dir.join(format!("{}.json", theme.id));
    let json = serde_json::to_string_pretty(theme)
        .map_err(|e| format!("Failed to serialize theme: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write theme file: {}", e))?;
    Ok(())
}

/// テーマフォルダのパスを返す（設定画面のフォルダを開くボタン用）
#[tauri::command]
pub fn get_themes_dir_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = get_themes_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

/// サンプルテーマフォルダのパスを返す（リソースディレクトリ内）
#[tauri::command]
pub fn get_sample_themes_dir_path(app: tauri::AppHandle) -> Result<String, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;
    let sample_dir = resource_dir.join("resources").join("sample-themes");
    Ok(sample_dir.to_string_lossy().to_string())
}
