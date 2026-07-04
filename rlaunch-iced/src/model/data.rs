//! launcher-data.json 互換データモデル。
//! 旧 Tauri 版のフィールド名（camelCase）を維持し、未知フィールドは `extra` に温存して
//! ラウンドトリップで壊さない。

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// 未知フィールドの受け皿（旧版・将来版とのデータ互換用）
pub type Extra = Map<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LauncherData {
    #[serde(default)]
    pub settings: AppSettings,
    #[serde(default)]
    pub tabs: Vec<Tab>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "d_hotkey")]
    pub hotkey: String,
    #[serde(default = "d_grid_columns")]
    pub default_grid_columns: u32,
    #[serde(default = "d_grid_rows")]
    pub default_grid_rows: u32,
    #[serde(default = "d_cell_size")]
    pub cell_size: u32,
    #[serde(default = "d_true")]
    pub show_labels: bool,
    #[serde(default = "d_label_font_size")]
    pub label_font_size: u32,
    #[serde(default = "d_theme")]
    pub theme: String,
    /// フォーカス喪失で自動非表示（旧版では未配線だった設定。iced 版で配線）
    #[serde(default = "d_true")]
    pub auto_hide: bool,
    #[serde(default = "d_true")]
    pub hide_on_launch: bool,
    /// "center" | "cursor" | "remember"
    #[serde(default = "d_window_position")]
    pub window_position: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_y: Option<i32>,
    #[serde(default = "d_app_title")]
    pub app_title: String,
    /// "grid" | "list"
    #[serde(default = "d_view_mode")]
    pub view_mode: String,
    #[serde(default = "d_list_columns")]
    pub list_columns: u32,
    /// CLaunch 風: カーソルがウィンドウ外に出たら非表示（iced 版の新設定）
    #[serde(default)]
    pub hide_on_cursor_out: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

fn d_hotkey() -> String {
    "Ctrl+Space".into()
}
fn d_grid_columns() -> u32 {
    8
}
fn d_grid_rows() -> u32 {
    4
}
fn d_cell_size() -> u32 {
    64
}
fn d_true() -> bool {
    true
}
fn d_label_font_size() -> u32 {
    10
}
fn d_theme() -> String {
    "dark".into()
}
fn d_window_position() -> String {
    "cursor".into()
}
fn d_app_title() -> String {
    "RLaunch".into()
}
fn d_view_mode() -> String {
    "grid".into()
}
fn d_list_columns() -> u32 {
    1
}

impl Default for AppSettings {
    fn default() -> Self {
        serde_json::from_str("{}").expect("empty AppSettings must deserialize with defaults")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "d_grid_columns")]
    pub grid_columns: u32,
    #[serde(default = "d_grid_rows")]
    pub grid_rows: u32,
    #[serde(default)]
    pub items: Vec<Option<GridCell>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_columns: Option<u32>,
    #[serde(flatten)]
    pub extra: Extra,
}

impl Tab {
    pub fn new(label: &str, columns: u32, rows: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            label: label.to_string(),
            order: 0,
            grid_columns: columns,
            grid_rows: rows,
            items: vec![None; (columns * rows) as usize],
            view_mode: None,
            list_columns: None,
            extra: Extra::new(),
        }
    }

    /// items の長さを cols×rows に合わせる（行列座標を保ってリマップ）
    pub fn resize_grid(&mut self, new_cols: u32, new_rows: u32) {
        let old_cols = self.grid_columns.max(1) as usize;
        let new_len = (new_cols * new_rows) as usize;
        let mut new_items: Vec<Option<GridCell>> = vec![None; new_len];
        for (i, cell) in self.items.drain(..).enumerate() {
            if cell.is_none() {
                continue;
            }
            let (r, c) = (i / old_cols, i % old_cols);
            if c < new_cols as usize && r < new_rows as usize {
                new_items[r * new_cols as usize + c] = cell;
            }
        }
        self.items = new_items;
        self.grid_columns = new_cols;
        self.grid_rows = new_rows;
    }

    /// items 長の整合性を保証（ロード直後の正規化用）。
    /// クランプで列/行数が変わる場合は行列座標リマップで位置崩れを防ぐ。
    pub fn normalize(&mut self) {
        let new_cols = self.grid_columns.clamp(1, 20);
        let new_rows = self.grid_rows.clamp(1, 10);
        if new_cols != self.grid_columns || new_rows != self.grid_rows {
            // resize_grid は self.grid_columns を旧列数として解釈する
            self.resize_grid(new_cols, new_rows);
        } else {
            let want = (new_cols * new_rows) as usize;
            self.items.resize(want, None);
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.iter().filter(|c| c.is_some()).count()
    }
}

/// グリッドセル。旧データの `type` フィールドで判別する。
#[derive(Debug, Clone)]
pub enum GridCell {
    Launcher(LauncherItem),
    Group(GroupItem),
    /// ウィジェットは機能廃止済みだがデータ互換のため温存（UI では不活性表示）
    Widget(WidgetItem),
}

impl Serialize for GridCell {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            GridCell::Launcher(v) => v.serialize(s),
            GridCell::Group(v) => v.serialize(s),
            GridCell::Widget(v) => v.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for GridCell {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        let ty = v.get("type").and_then(Value::as_str).unwrap_or_default();
        let result = match ty {
            "group" => GroupItem::deserialize(v).map(GridCell::Group),
            "widget" => WidgetItem::deserialize(v).map(GridCell::Widget),
            _ => LauncherItem::deserialize(v).map(GridCell::Launcher),
        };
        result.map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherItem {
    pub id: String,
    #[serde(default)]
    pub label: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_icon: Option<String>,
    /// "executable" | "shortcut" | "folder" | "url" | "document"
    #[serde(rename = "type", default)]
    pub item_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as: Option<bool>,
    /// "normal" | "maximized" | "minimized"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotkey: Option<String>,
    /// "open" | "browse"（フォルダのみ）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_launched_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: Extra,
}

impl LauncherItem {
    pub fn is_url(&self) -> bool {
        self.item_type == "url"
    }
    pub fn is_folder(&self) -> bool {
        self.item_type == "folder"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupItem {
    pub id: String,
    /// 常に "group"
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_icon: Option<String>,
    #[serde(default)]
    pub items: Vec<Option<GridCell>>,
    #[serde(default = "d_group_cols")]
    pub grid_columns: u32,
    #[serde(default = "d_group_rows")]
    pub grid_rows: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_columns: Option<u32>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(flatten)]
    pub extra: Extra,
}

fn d_group_cols() -> u32 {
    4
}
fn d_group_rows() -> u32 {
    2
}

impl GroupItem {
    pub fn new(label: &str) -> Self {
        let now = now_iso8601();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: "group".into(),
            label: label.to_string(),
            icon: None,
            icon_color: None,
            icon_base64: None,
            library_icon: None,
            items: vec![None; 8],
            grid_columns: 4,
            grid_rows: 2,
            view_mode: None,
            list_columns: None,
            created_at: now.clone(),
            updated_at: now,
            extra: Extra::new(),
        }
    }

    pub fn normalize(&mut self) {
        self.grid_columns = self.grid_columns.clamp(1, 8);
        self.grid_rows = self.grid_rows.clamp(1, 6);
        let want = (self.grid_columns * self.grid_rows) as usize;
        self.items.resize(want, None);
    }
}

/// ウィジェット（機能廃止・データ互換のためだけに温存）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetItem {
    pub id: String,
    /// 常に "widget"
    #[serde(rename = "type")]
    pub item_type: String,
    pub widget_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub config: Value,
    #[serde(default)]
    pub update_interval: u64,
    #[serde(flatten)]
    pub extra: Extra,
}

pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_roundtrip_preserves_unknown_fields() {
        let src = r#"{
            "settings": { "hotkey": "Ctrl+Space", "theme": "dark", "windowEffect": "mica", "cellSize": 72 },
            "tabs": [{
                "id": "t1", "label": "メイン", "order": 0,
                "gridColumns": 2, "gridRows": 1,
                "items": [
                    { "id": "a", "label": "メモ帳", "path": "C:\\Windows\\notepad.exe", "type": "executable",
                      "createdAt": "2025-01-01T00:00:00Z", "updatedAt": "2025-01-01T00:00:00Z", "customField": 42 },
                    null
                ]
            }]
        }"#;
        let data: LauncherData = serde_json::from_str(src).expect("parse legacy");
        assert_eq!(data.settings.cell_size, 72);
        assert_eq!(data.settings.extra.get("windowEffect").unwrap(), "mica");
        assert_eq!(data.tabs.len(), 1);
        let cell = data.tabs[0].items[0].as_ref().expect("cell");
        match cell {
            GridCell::Launcher(item) => {
                assert_eq!(item.label, "メモ帳");
                assert_eq!(item.extra.get("customField").unwrap(), 42);
            }
            _ => panic!("expected launcher item"),
        }
        assert!(data.tabs[0].items[1].is_none());

        let out = serde_json::to_value(&data).expect("serialize");
        assert_eq!(out["settings"]["windowEffect"], "mica");
        assert_eq!(out["tabs"][0]["items"][0]["customField"], 42);
        assert_eq!(out["tabs"][0]["items"][0]["type"], "executable");
        assert_eq!(out["tabs"][0]["items"][1], Value::Null);
    }

    #[test]
    fn cell_type_discrimination() {
        let group = r#"{ "id":"g","type":"group","label":"G","items":[null],"gridColumns":1,"gridRows":1,
                        "createdAt":"","updatedAt":"" }"#;
        let widget = r#"{ "id":"w","type":"widget","widgetType":"analog-clock","config":{},"updateInterval":1000 }"#;
        let folder = r#"{ "id":"f","label":"F","path":"C:\\","type":"folder","createdAt":"","updatedAt":"" }"#;
        assert!(matches!(
            serde_json::from_str::<GridCell>(group).unwrap(),
            GridCell::Group(_)
        ));
        assert!(matches!(
            serde_json::from_str::<GridCell>(widget).unwrap(),
            GridCell::Widget(_)
        ));
        assert!(matches!(
            serde_json::from_str::<GridCell>(folder).unwrap(),
            GridCell::Launcher(_)
        ));
    }

    #[test]
    fn tab_resize_keeps_row_col_positions() {
        let mut tab = Tab::new("t", 3, 2);
        tab.items[0] = Some(GridCell::Group(GroupItem::new("a"))); // (0,0)
        tab.items[4] = Some(GridCell::Group(GroupItem::new("b"))); // (1,1)
        tab.resize_grid(2, 2);
        assert!(tab.items[0].is_some()); // (0,0) 保持
        assert!(tab.items[3].is_some()); // (1,1) → index 1*2+1=3
        assert_eq!(tab.item_count(), 2);
    }
}
