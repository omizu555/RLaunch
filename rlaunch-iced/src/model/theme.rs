//! テーマ: 旧版互換の CSS 変数 JSON（{id,label,author,variables}）を読み込み、
//! iced 用の解決済みカラー構造体 `UiTheme` へ変換する。

use iced::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// バンドルテーマ（ビルトイン6種 + サンプル19種）
const BUNDLED_THEMES: &str = include_str!("../../assets/themes-bundled.json");

pub const BUILTIN_IDS: [&str; 6] = [
    "dark",
    "light",
    "classic",
    "flat-white",
    "flat-dark",
    "mono",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

/// 解決済みテーマ（描画で直接使う）
#[derive(Debug, Clone)]
pub struct UiTheme {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_button: Color,
    pub bg_button_hover: Color,
    pub bg_button_active: Color,
    pub bg_button_empty: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub border_color: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub danger: Color,
    pub success: Color,
    pub warning: Color,
    pub border_radius: f32,
    pub border_radius_sm: f32,
    /// ウィンドウ全体の不透明度（0.0-1.0、透過テーマ用）
    pub window_opacity: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        // ビルトイン dark 相当
        Self {
            bg_primary: hex("#1e1e2e"),
            bg_secondary: hex("#181825"),
            bg_button: hex("#313244"),
            bg_button_hover: hex("#45475a"),
            bg_button_active: hex("#585b70"),
            bg_button_empty: Color::from_rgba8(69, 71, 90, 0.25),
            text_primary: hex("#cdd6f4"),
            text_secondary: hex("#a6adc8"),
            text_muted: hex("#6c7086"),
            border_color: hex("#45475a"),
            accent: hex("#89b4fa"),
            accent_hover: hex("#74c7ec"),
            danger: hex("#f38ba8"),
            success: hex("#a6e3a1"),
            warning: hex("#fab387"),
            border_radius: 8.0,
            border_radius_sm: 4.0,
            window_opacity: 1.0,
        }
    }
}

fn hex(s: &str) -> Color {
    parse_color(s).unwrap_or(Color::BLACK)
}

impl UiTheme {
    pub fn from_info(info: &ThemeInfo) -> Self {
        let d = UiTheme::default();
        let v = |key: &str, fallback: Color| -> Color {
            info.variables
                .get(key)
                .and_then(|s| parse_color(s))
                .unwrap_or(fallback)
        };
        let px = |key: &str, fallback: f32| -> f32 {
            info.variables
                .get(key)
                .and_then(|s| parse_px(s))
                .unwrap_or(fallback)
        };
        Self {
            bg_primary: v("--bg-primary", d.bg_primary),
            bg_secondary: v("--bg-secondary", d.bg_secondary),
            bg_button: v("--bg-button", d.bg_button),
            bg_button_hover: v("--bg-button-hover", d.bg_button_hover),
            bg_button_active: v("--bg-button-active", d.bg_button_active),
            bg_button_empty: v("--bg-button-empty", d.bg_button_empty),
            text_primary: v("--text-primary", d.text_primary),
            text_secondary: v("--text-secondary", d.text_secondary),
            text_muted: v("--text-muted", d.text_muted),
            border_color: v("--border-color", d.border_color),
            accent: v("--accent-color", d.accent),
            accent_hover: v("--accent-hover", d.accent_hover),
            danger: v("--danger-color", d.danger),
            success: v("--success-color", d.success),
            warning: v("--warning-color", d.warning),
            border_radius: px("--border-radius", d.border_radius),
            border_radius_sm: px("--border-radius-sm", d.border_radius_sm),
            window_opacity: info
                .variables
                .get("--window-opacity")
                .and_then(|s| s.trim().parse::<f32>().ok())
                .map(|o| o.clamp(0.1, 1.0))
                .unwrap_or(1.0),
        }
    }

    /// iced のベーステーマ（テキストデフォルト色などに効く）
    pub fn to_iced_theme(&self) -> iced::Theme {
        iced::Theme::custom(
            "rlaunch".to_string(),
            iced::theme::Palette {
                background: self.bg_primary,
                text: self.text_primary,
                primary: self.accent,
                success: self.success,
                warning: self.warning,
                danger: self.danger,
            },
        )
    }
}

/// テーマ一覧: バンドル25種 + ユーザーフォルダ（同IDはユーザー優先）。builtin 先頭ソート。
pub fn load_all(user_themes_dir: &Path) -> Vec<ThemeInfo> {
    let mut themes: Vec<ThemeInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if user_themes_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(user_themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(text) = std::fs::read_to_string(&path) {
                        match serde_json::from_str::<ThemeInfo>(&text) {
                            Ok(info) => {
                                seen.insert(info.id.clone());
                                themes.push(info);
                            }
                            Err(e) => eprintln!("テーマ {} を読めません: {}", path.display(), e),
                        }
                    }
                }
            }
        }
    }

    match serde_json::from_str::<Vec<ThemeInfo>>(BUNDLED_THEMES) {
        Ok(bundled) => {
            for info in bundled {
                if !seen.contains(&info.id) {
                    seen.insert(info.id.clone());
                    themes.push(info);
                }
            }
        }
        Err(e) => eprintln!("バンドルテーマのパースに失敗: {}", e),
    }

    themes.sort_by(|a, b| {
        let ab = BUILTIN_IDS.contains(&a.id.as_str());
        let bb = BUILTIN_IDS.contains(&b.id.as_str());
        match (ab, bb) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.label.cmp(&b.label),
        }
    });
    themes
}

pub fn find<'a>(themes: &'a [ThemeInfo], id: &str) -> Option<&'a ThemeInfo> {
    themes.iter().find(|t| t.id == id)
}

/// CSS カラー文字列をパース: #rgb / #rrggbb / #rrggbbaa / rgb(...) / rgba(...)
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let hex = hex.trim();
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color::from_rgb8(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::from_rgb8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::from_rgba8(r, g, b, a as f32 / 255.0))
            }
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("rgb") {
        let inner = s.find('(').and_then(|start| {
            s.rfind(')')
                .filter(|end| *end > start)
                .map(|end| &s[start + 1..end])
        })?;
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() < 3 {
            return None;
        }
        let r = parts[0].parse::<f32>().ok()?;
        let g = parts[1].parse::<f32>().ok()?;
        let b = parts[2].parse::<f32>().ok()?;
        let a = if parts.len() >= 4 {
            parts[3].parse::<f32>().ok()?
        } else {
            1.0
        };
        return Some(Color::from_rgba(
            (r / 255.0).clamp(0.0, 1.0),
            (g / 255.0).clamp(0.0, 1.0),
            (b / 255.0).clamp(0.0, 1.0),
            a.clamp(0.0, 1.0),
        ));
    }
    None
}

fn parse_px(s: &str) -> Option<f32> {
    s.trim().trim_end_matches("px").trim().parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_themes_parse() {
        let themes: Vec<ThemeInfo> = serde_json::from_str(BUNDLED_THEMES).expect("bundle parses");
        assert_eq!(themes.len(), 25);
        for id in BUILTIN_IDS {
            assert!(themes.iter().any(|t| t.id == id), "missing builtin {}", id);
        }
    }

    #[test]
    fn color_parsing() {
        assert!(parse_color("#1e1e2e").is_some());
        assert!(parse_color("#fff").is_some());
        assert!(parse_color("#11223344").is_some());
        let c = parse_color("rgba(69, 71, 90, 0.25)").unwrap();
        assert!((c.a - 0.25).abs() < 0.001);
        assert!(parse_color("rgb(255, 0, 0)").is_some());
        assert!(parse_color("nonsense").is_none());
    }

    #[test]
    fn ui_theme_resolves_all_builtins() {
        let themes: Vec<ThemeInfo> = serde_json::from_str(BUNDLED_THEMES).unwrap();
        for info in &themes {
            let ui = UiTheme::from_info(info);
            assert!(ui.window_opacity > 0.0);
        }
    }
}
