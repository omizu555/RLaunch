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
    ]
}

/// 初回起動時: themes/ フォルダにビルトイン + サンプルテーマを書き出す
#[tauri::command]
pub fn init_themes(app: tauri::AppHandle) -> Result<(), String> {
    let themes_dir = get_themes_dir(&app)?;

    if !themes_dir.exists() {
        fs::create_dir_all(&themes_dir)
            .map_err(|e| format!("Failed to create themes dir: {}", e))?;

        // ビルトインテーマを書き出し
        for theme in builtin_themes() {
            write_theme(&themes_dir, &theme)?;
        }

        // サンプルテーマを書き出し
        for theme in sample_themes() {
            write_theme(&themes_dir, &theme)?;
        }
    }

    Ok(())
}

/// テーマ一覧を取得（themes/ フォルダをスキャン）
#[tauri::command]
pub fn list_themes(app: tauri::AppHandle) -> Result<Vec<ThemeInfo>, String> {
    let themes_dir = get_themes_dir(&app)?;

    if !themes_dir.exists() {
        // フォルダ未作成ならビルトインのみ返す
        return Ok(builtin_themes());
    }

    let mut themes: Vec<ThemeInfo> = Vec::new();

    let entries = fs::read_dir(&themes_dir)
        .map_err(|e| format!("Failed to read themes dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<ThemeInfo>(&content) {
                    Ok(theme) => themes.push(theme),
                    Err(e) => eprintln!("Invalid theme file {:?}: {}", path, e),
                },
                Err(e) => eprintln!("Failed to read theme {:?}: {}", path, e),
            }
        }
    }

    // id でソート（builtin を先頭に）
    themes.sort_by(|a, b| {
        let a_builtin = matches!(a.id.as_str(), "dark" | "light" | "classic");
        let b_builtin = matches!(b.id.as_str(), "dark" | "light" | "classic");
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

/// サンプルテーマ 10 個
fn sample_themes() -> Vec<ThemeInfo> {
    vec![
        // 1. Nord - 北欧の冬を思わせる冷たいブルー
        ThemeInfo {
            id: "nord".into(),
            label: "Nord".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#2e3440".into()),
                ("--bg-secondary".into(), "#3b4252".into()),
                ("--bg-button".into(), "#434c5e".into()),
                ("--bg-button-hover".into(), "#4c566a".into()),
                ("--bg-button-active".into(), "#5e6779".into()),
                ("--bg-button-empty".into(), "rgba(67, 76, 94, 0.3)".into()),
                ("--text-primary".into(), "#eceff4".into()),
                ("--text-secondary".into(), "#d8dee9".into()),
                ("--text-muted".into(), "#7b88a1".into()),
                ("--border-color".into(), "#4c566a".into()),
                ("--accent-color".into(), "#88c0d0".into()),
                ("--accent-hover".into(), "#8fbcbb".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.35)".into()),
                ("--danger-color".into(), "#bf616a".into()),
                ("--success-color".into(), "#a3be8c".into()),
                ("--warning-color".into(), "#ebcb8b".into()),
                ("--border-radius".into(), "6px".into()),
                ("--border-radius-sm".into(), "3px".into()),
            ]),
        },
        // 2. Solarized Dark - 目に優しい計算されたカラーパレット
        ThemeInfo {
            id: "solarized-dark".into(),
            label: "Solarized Dark".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#002b36".into()),
                ("--bg-secondary".into(), "#073642".into()),
                ("--bg-button".into(), "#0a4050".into()),
                ("--bg-button-hover".into(), "#1a5060".into()),
                ("--bg-button-active".into(), "#2a6070".into()),
                ("--bg-button-empty".into(), "rgba(7, 54, 66, 0.4)".into()),
                ("--text-primary".into(), "#fdf6e3".into()),
                ("--text-secondary".into(), "#eee8d5".into()),
                ("--text-muted".into(), "#657b83".into()),
                ("--border-color".into(), "#586e75".into()),
                ("--accent-color".into(), "#268bd2".into()),
                ("--accent-hover".into(), "#2aa198".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.5)".into()),
                ("--danger-color".into(), "#dc322f".into()),
                ("--success-color".into(), "#859900".into()),
                ("--warning-color".into(), "#b58900".into()),
                ("--border-radius".into(), "6px".into()),
                ("--border-radius-sm".into(), "3px".into()),
            ]),
        },
        // 3. Monokai - コーディングの定番、ビビッドなアクセント
        ThemeInfo {
            id: "monokai".into(),
            label: "Monokai".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#272822".into()),
                ("--bg-secondary".into(), "#1e1f1c".into()),
                ("--bg-button".into(), "#3e3d32".into()),
                ("--bg-button-hover".into(), "#4e4d42".into()),
                ("--bg-button-active".into(), "#5e5d52".into()),
                ("--bg-button-empty".into(), "rgba(62, 61, 50, 0.3)".into()),
                ("--text-primary".into(), "#f8f8f2".into()),
                ("--text-secondary".into(), "#cfcfc2".into()),
                ("--text-muted".into(), "#75715e".into()),
                ("--border-color".into(), "#49483e".into()),
                ("--accent-color".into(), "#66d9ef".into()),
                ("--accent-hover".into(), "#a6e22e".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.45)".into()),
                ("--danger-color".into(), "#f92672".into()),
                ("--success-color".into(), "#a6e22e".into()),
                ("--warning-color".into(), "#e6db74".into()),
                ("--border-radius".into(), "6px".into()),
                ("--border-radius-sm".into(), "3px".into()),
            ]),
        },
        // 4. Dracula - 人気のダークテーマ、パープル系
        ThemeInfo {
            id: "dracula".into(),
            label: "Dracula".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#282a36".into()),
                ("--bg-secondary".into(), "#21222c".into()),
                ("--bg-button".into(), "#44475a".into()),
                ("--bg-button-hover".into(), "#545772".into()),
                ("--bg-button-active".into(), "#626591".into()),
                ("--bg-button-empty".into(), "rgba(68, 71, 90, 0.3)".into()),
                ("--text-primary".into(), "#f8f8f2".into()),
                ("--text-secondary".into(), "#d4d4e8".into()),
                ("--text-muted".into(), "#6272a4".into()),
                ("--border-color".into(), "#6272a4".into()),
                ("--accent-color".into(), "#bd93f9".into()),
                ("--accent-hover".into(), "#ff79c6".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.4)".into()),
                ("--danger-color".into(), "#ff5555".into()),
                ("--success-color".into(), "#50fa7b".into()),
                ("--warning-color".into(), "#f1fa8c".into()),
                ("--border-radius".into(), "8px".into()),
                ("--border-radius-sm".into(), "4px".into()),
            ]),
        },
        // 5. Gruvbox Dark - レトロでウォームな配色
        ThemeInfo {
            id: "gruvbox-dark".into(),
            label: "Gruvbox Dark".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#282828".into()),
                ("--bg-secondary".into(), "#1d2021".into()),
                ("--bg-button".into(), "#3c3836".into()),
                ("--bg-button-hover".into(), "#504945".into()),
                ("--bg-button-active".into(), "#665c54".into()),
                ("--bg-button-empty".into(), "rgba(60, 56, 54, 0.35)".into()),
                ("--text-primary".into(), "#ebdbb2".into()),
                ("--text-secondary".into(), "#d5c4a1".into()),
                ("--text-muted".into(), "#928374".into()),
                ("--border-color".into(), "#504945".into()),
                ("--accent-color".into(), "#fabd2f".into()),
                ("--accent-hover".into(), "#fe8019".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.4)".into()),
                ("--danger-color".into(), "#fb4934".into()),
                ("--success-color".into(), "#b8bb26".into()),
                ("--warning-color".into(), "#fabd2f".into()),
                ("--border-radius".into(), "4px".into()),
                ("--border-radius-sm".into(), "2px".into()),
            ]),
        },
        // 6. Tokyo Night - VS Code で人気の深い夜空
        ThemeInfo {
            id: "tokyo-night".into(),
            label: "Tokyo Night".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#1a1b26".into()),
                ("--bg-secondary".into(), "#16161e".into()),
                ("--bg-button".into(), "#292e42".into()),
                ("--bg-button-hover".into(), "#343b58".into()),
                ("--bg-button-active".into(), "#3d4570".into()),
                ("--bg-button-empty".into(), "rgba(41, 46, 66, 0.35)".into()),
                ("--text-primary".into(), "#c0caf5".into()),
                ("--text-secondary".into(), "#a9b1d6".into()),
                ("--text-muted".into(), "#565f89".into()),
                ("--border-color".into(), "#3b4261".into()),
                ("--accent-color".into(), "#7aa2f7".into()),
                ("--accent-hover".into(), "#7dcfff".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.5)".into()),
                ("--danger-color".into(), "#f7768e".into()),
                ("--success-color".into(), "#9ece6a".into()),
                ("--warning-color".into(), "#e0af68".into()),
                ("--border-radius".into(), "8px".into()),
                ("--border-radius-sm".into(), "4px".into()),
            ]),
        },
        // 7. Rosé Pine - やわらかなピンクとパープルのエレガントなテーマ
        ThemeInfo {
            id: "rose-pine".into(),
            label: "Rosé Pine".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#191724".into()),
                ("--bg-secondary".into(), "#1f1d2e".into()),
                ("--bg-button".into(), "#26233a".into()),
                ("--bg-button-hover".into(), "#2a2740".into()),
                ("--bg-button-active".into(), "#393552".into()),
                ("--bg-button-empty".into(), "rgba(38, 35, 58, 0.35)".into()),
                ("--text-primary".into(), "#e0def4".into()),
                ("--text-secondary".into(), "#c4a7e7".into()),
                ("--text-muted".into(), "#6e6a86".into()),
                ("--border-color".into(), "#393552".into()),
                ("--accent-color".into(), "#ebbcba".into()),
                ("--accent-hover".into(), "#f6c177".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.5)".into()),
                ("--danger-color".into(), "#eb6f92".into()),
                ("--success-color".into(), "#9ccfd8".into()),
                ("--warning-color".into(), "#f6c177".into()),
                ("--border-radius".into(), "10px".into()),
                ("--border-radius-sm".into(), "5px".into()),
            ]),
        },
        // 8. Everforest - 自然のグリーンをベースにした森テーマ
        ThemeInfo {
            id: "everforest".into(),
            label: "Everforest".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#2d353b".into()),
                ("--bg-secondary".into(), "#272e33".into()),
                ("--bg-button".into(), "#374145".into()),
                ("--bg-button-hover".into(), "#414b50".into()),
                ("--bg-button-active".into(), "#4f585e".into()),
                ("--bg-button-empty".into(), "rgba(55, 65, 69, 0.3)".into()),
                ("--text-primary".into(), "#d3c6aa".into()),
                ("--text-secondary".into(), "#c5b89a".into()),
                ("--text-muted".into(), "#7a8478".into()),
                ("--border-color".into(), "#4f585e".into()),
                ("--accent-color".into(), "#a7c080".into()),
                ("--accent-hover".into(), "#83c092".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.35)".into()),
                ("--danger-color".into(), "#e67e80".into()),
                ("--success-color".into(), "#a7c080".into()),
                ("--warning-color".into(), "#dbbc7f".into()),
                ("--border-radius".into(), "6px".into()),
                ("--border-radius-sm".into(), "3px".into()),
            ]),
        },
        // 9. Cyberpunk - ネオンカラーの近未来テーマ
        ThemeInfo {
            id: "cyberpunk".into(),
            label: "Cyberpunk".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#0d0221".into()),
                ("--bg-secondary".into(), "#150535".into()),
                ("--bg-button".into(), "#1a0a3e".into()),
                ("--bg-button-hover".into(), "#2b1055".into()),
                ("--bg-button-active".into(), "#3c1a6e".into()),
                ("--bg-button-empty".into(), "rgba(26, 10, 62, 0.4)".into()),
                ("--text-primary".into(), "#0ff0fc".into()),
                ("--text-secondary".into(), "#ff2a6d".into()),
                ("--text-muted".into(), "#7b5ea7".into()),
                ("--border-color".into(), "#541388".into()),
                ("--accent-color".into(), "#f706cf".into()),
                ("--accent-hover".into(), "#0ff0fc".into()),
                ("--shadow-color".into(), "rgba(247, 6, 207, 0.3)".into()),
                ("--danger-color".into(), "#ff2a6d".into()),
                ("--success-color".into(), "#05ffa1".into()),
                ("--warning-color".into(), "#fcee09".into()),
                ("--border-radius".into(), "2px".into()),
                ("--border-radius-sm".into(), "1px".into()),
            ]),
        },
        // 10. Sunset - 夕焼けの暖かいグラデーション
        ThemeInfo {
            id: "sunset".into(),
            label: "Sunset Glow".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#2b1b2e".into()),
                ("--bg-secondary".into(), "#231526".into()),
                ("--bg-button".into(), "#3d2442".into()),
                ("--bg-button-hover".into(), "#4e3054".into()),
                ("--bg-button-active".into(), "#5f3d66".into()),
                ("--bg-button-empty".into(), "rgba(61, 36, 66, 0.3)".into()),
                ("--text-primary".into(), "#ffecd2".into()),
                ("--text-secondary".into(), "#f0c4a8".into()),
                ("--text-muted".into(), "#9a7b8c".into()),
                ("--border-color".into(), "#5f3d66".into()),
                ("--accent-color".into(), "#ff6b6b".into()),
                ("--accent-hover".into(), "#ffa06b".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.4)".into()),
                ("--danger-color".into(), "#ff4757".into()),
                ("--success-color".into(), "#7bed9f".into()),
                ("--warning-color".into(), "#ffa502".into()),
                ("--border-radius".into(), "10px".into()),
                ("--border-radius-sm".into(), "5px".into()),
            ]),
        },
        // 11. High Contrast - P-28: ハイコントラストモード (WCAG AAA 4.5:1+)
        ThemeInfo {
            id: "high-contrast".into(),
            label: "ハイコントラスト".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "#000000".into()),
                ("--bg-secondary".into(), "#0a0a0a".into()),
                ("--bg-button".into(), "#1a1a1a".into()),
                ("--bg-button-hover".into(), "#333333".into()),
                ("--bg-button-active".into(), "#4d4d4d".into()),
                ("--bg-button-empty".into(), "rgba(255, 255, 255, 0.08)".into()),
                ("--text-primary".into(), "#ffffff".into()),
                ("--text-secondary".into(), "#e0e0e0".into()),
                ("--text-muted".into(), "#b0b0b0".into()),
                ("--border-color".into(), "#ffffff".into()),
                ("--accent-color".into(), "#ffff00".into()),
                ("--accent-hover".into(), "#00ffff".into()),
                ("--shadow-color".into(), "rgba(255, 255, 255, 0.1)".into()),
                ("--danger-color".into(), "#ff6666".into()),
                ("--success-color".into(), "#66ff66".into()),
                ("--warning-color".into(), "#ffcc00".into()),
                ("--border-radius".into(), "4px".into()),
                ("--border-radius-sm".into(), "2px".into()),
            ]),
        },
        // 12. Glass Dark - 半透明ダークテーマ（背景が透けて見える）
        ThemeInfo {
            id: "glass".into(),
            label: "Glass Dark (透過)".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "rgba(30, 30, 46, 0.6)".into()),
                ("--bg-secondary".into(), "rgba(24, 24, 37, 0.7)".into()),
                ("--bg-button".into(), "rgba(49, 50, 68, 0.5)".into()),
                ("--bg-button-hover".into(), "rgba(69, 71, 90, 0.6)".into()),
                ("--bg-button-active".into(), "rgba(88, 91, 112, 0.7)".into()),
                ("--bg-button-empty".into(), "rgba(69, 71, 90, 0.15)".into()),
                ("--text-primary".into(), "#cdd6f4".into()),
                ("--text-secondary".into(), "#a6adc8".into()),
                ("--text-muted".into(), "#6c7086".into()),
                ("--border-color".into(), "rgba(69, 71, 90, 0.5)".into()),
                ("--accent-color".into(), "#89b4fa".into()),
                ("--accent-hover".into(), "#74c7ec".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.2)".into()),
                ("--danger-color".into(), "#f38ba8".into()),
                ("--success-color".into(), "#a6e3a1".into()),
                ("--warning-color".into(), "#fab387".into()),
                ("--border-radius".into(), "12px".into()),
                ("--border-radius-sm".into(), "6px".into()),
                ("--window-opacity".into(), "0.85".into()),
                ("--window-effect".into(), "acrylic".into()),
            ]),
        },
        // 13. Glass Light - 半透明ライトテーマ
        ThemeInfo {
            id: "glass-light".into(),
            label: "Glass Light (透過)".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "rgba(239, 241, 245, 0.6)".into()),
                ("--bg-secondary".into(), "rgba(230, 233, 239, 0.7)".into()),
                ("--bg-button".into(), "rgba(204, 208, 218, 0.5)".into()),
                ("--bg-button-hover".into(), "rgba(188, 192, 204, 0.6)".into()),
                ("--bg-button-active".into(), "rgba(172, 176, 190, 0.7)".into()),
                ("--bg-button-empty".into(), "rgba(204, 208, 218, 0.15)".into()),
                ("--text-primary".into(), "#4c4f69".into()),
                ("--text-secondary".into(), "#5c5f77".into()),
                ("--text-muted".into(), "#9ca0b0".into()),
                ("--border-color".into(), "rgba(172, 176, 190, 0.5)".into()),
                ("--accent-color".into(), "#1e66f5".into()),
                ("--accent-hover".into(), "#2a6ef7".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.1)".into()),
                ("--danger-color".into(), "#d20f39".into()),
                ("--success-color".into(), "#40a02b".into()),
                ("--warning-color".into(), "#df8e1d".into()),
                ("--border-radius".into(), "12px".into()),
                ("--border-radius-sm".into(), "6px".into()),
                ("--window-opacity".into(), "0.85".into()),
                ("--window-effect".into(), "mica".into()),
            ]),
        },
        // 14. Wireframe - 線だけ見えてバックグラウンドが透けるテーマ
        ThemeInfo {
            id: "wireframe".into(),
            label: "Wireframe (スケルトン)".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "rgba(0, 0, 0, 0.08)".into()),
                ("--bg-secondary".into(), "rgba(0, 0, 0, 0.05)".into()),
                ("--bg-button".into(), "rgba(255, 255, 255, 0.03)".into()),
                ("--bg-button-hover".into(), "rgba(255, 255, 255, 0.12)".into()),
                ("--bg-button-active".into(), "rgba(255, 255, 255, 0.18)".into()),
                ("--bg-button-empty".into(), "rgba(255, 255, 255, 0.02)".into()),
                ("--text-primary".into(), "rgba(255, 255, 255, 0.9)".into()),
                ("--text-secondary".into(), "rgba(255, 255, 255, 0.6)".into()),
                ("--text-muted".into(), "rgba(255, 255, 255, 0.3)".into()),
                ("--border-color".into(), "rgba(255, 255, 255, 0.25)".into()),
                ("--accent-color".into(), "rgba(137, 180, 250, 0.8)".into()),
                ("--accent-hover".into(), "rgba(116, 199, 236, 0.8)".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.0)".into()),
                ("--danger-color".into(), "#f38ba8".into()),
                ("--success-color".into(), "#a6e3a1".into()),
                ("--warning-color".into(), "#fab387".into()),
                ("--border-radius".into(), "8px".into()),
                ("--border-radius-sm".into(), "4px".into()),
                ("--window-opacity".into(), "0.9".into()),
                ("--window-effect".into(), "acrylic".into()),
            ]),
        },
        // 15. Frosted - すりガラス風テーマ
        ThemeInfo {
            id: "frosted".into(),
            label: "Frosted (すりガラス)".into(),
            author: "sample".into(),
            variables: HashMap::from([
                ("--bg-primary".into(), "rgba(30, 30, 50, 0.45)".into()),
                ("--bg-secondary".into(), "rgba(20, 20, 40, 0.55)".into()),
                ("--bg-button".into(), "rgba(60, 60, 90, 0.35)".into()),
                ("--bg-button-hover".into(), "rgba(80, 80, 120, 0.45)".into()),
                ("--bg-button-active".into(), "rgba(100, 100, 140, 0.55)".into()),
                ("--bg-button-empty".into(), "rgba(60, 60, 90, 0.1)".into()),
                ("--text-primary".into(), "rgba(255, 255, 255, 0.95)".into()),
                ("--text-secondary".into(), "rgba(200, 200, 230, 0.8)".into()),
                ("--text-muted".into(), "rgba(160, 160, 200, 0.5)".into()),
                ("--border-color".into(), "rgba(120, 120, 180, 0.3)".into()),
                ("--accent-color".into(), "#b4befe".into()),
                ("--accent-hover".into(), "#cba6f7".into()),
                ("--shadow-color".into(), "rgba(0, 0, 0, 0.15)".into()),
                ("--danger-color".into(), "#f38ba8".into()),
                ("--success-color".into(), "#a6e3a1".into()),
                ("--warning-color".into(), "#fab387".into()),
                ("--border-radius".into(), "14px".into()),
                ("--border-radius-sm".into(), "6px".into()),
                ("--window-opacity".into(), "0.8".into()),
                ("--window-effect".into(), "acrylic".into()),
            ]),
        },
    ]
}
