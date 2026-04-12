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
        // Flat White — 純白×黒、影なし、完全フラットデザイン
        ThemeInfo {
            id: "flat-white".into(),
            label: "Flat White".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#ffffff".into()),
                ("--bg-secondary".into(), "#ffffff".into()),
                ("--bg-button".into(), "#f5f5f5".into()),
                ("--bg-button-hover".into(), "#e8e8e8".into()),
                ("--bg-button-active".into(), "#d9d9d9".into()),
                ("--bg-button-empty".into(), "rgba(0, 0, 0, 0.03)".into()),
                ("--text-primary".into(), "#000000".into()),
                ("--text-secondary".into(), "#333333".into()),
                ("--text-muted".into(), "#999999".into()),
                ("--border-color".into(), "#d0d0d0".into()),
                ("--accent-color".into(), "#000000".into()),
                ("--accent-hover".into(), "#333333".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0)".into()),
                ("--danger-color".into(), "#cc0000".into()),
                ("--success-color".into(), "#008800".into()),
                ("--warning-color".into(), "#cc8800".into()),
                ("--border-radius".into(), "0px".into()),
                ("--border-radius-sm".into(), "0px".into()),
            ]),
        },
        // Flat Dark — 純黒×白文字、影なし、完全フラットデザイン
        ThemeInfo {
            id: "flat-dark".into(),
            label: "Flat Dark".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#000000".into()),
                ("--bg-secondary".into(), "#000000".into()),
                ("--bg-button".into(), "#1a1a1a".into()),
                ("--bg-button-hover".into(), "#2a2a2a".into()),
                ("--bg-button-active".into(), "#3a3a3a".into()),
                ("--bg-button-empty".into(), "rgba(255, 255, 255, 0.04)".into()),
                ("--text-primary".into(), "#ffffff".into()),
                ("--text-secondary".into(), "#cccccc".into()),
                ("--text-muted".into(), "#666666".into()),
                ("--border-color".into(), "#333333".into()),
                ("--accent-color".into(), "#ffffff".into()),
                ("--accent-hover".into(), "#cccccc".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0)".into()),
                ("--danger-color".into(), "#ff4444".into()),
                ("--success-color".into(), "#44cc44".into()),
                ("--warning-color".into(), "#ffaa00".into()),
                ("--border-radius".into(), "0px".into()),
                ("--border-radius-sm".into(), "0px".into()),
            ]),
        },
        // Mono — グレースケール中間調、影なし、ニュートラルフラット
        ThemeInfo {
            id: "mono".into(),
            label: "Mono".into(),
            author: "builtin".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#f0f0f0".into()),
                ("--bg-secondary".into(), "#e6e6e6".into()),
                ("--bg-button".into(), "#dcdcdc".into()),
                ("--bg-button-hover".into(), "#c8c8c8".into()),
                ("--bg-button-active".into(), "#b4b4b4".into()),
                ("--bg-button-empty".into(), "rgba(0, 0, 0, 0.05)".into()),
                ("--text-primary".into(), "#111111".into()),
                ("--text-secondary".into(), "#444444".into()),
                ("--text-muted".into(), "#888888".into()),
                ("--border-color".into(), "#b0b0b0".into()),
                ("--accent-color".into(), "#222222".into()),
                ("--accent-hover".into(), "#000000".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0)".into()),
                ("--danger-color".into(), "#990000".into()),
                ("--success-color".into(), "#006600".into()),
                ("--warning-color".into(), "#996600".into()),
                ("--border-radius".into(), "0px".into()),
                ("--border-radius-sm".into(), "0px".into()),
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
        let a_builtin = matches!(a.id.as_str(), "dark" | "light" | "classic" | "flat-white" | "flat-dark" | "mono");
        let b_builtin = matches!(b.id.as_str(), "dark" | "light" | "classic" | "flat-white" | "flat-dark" | "mono");
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
