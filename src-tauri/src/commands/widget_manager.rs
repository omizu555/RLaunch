/* ============================================================
   widget_manager - ウィジェットフォルダの管理・読み込み
   widgets/ フォルダ内のサブディレクトリ（manifest.json + widget.js）を
   スキャンしてプラグインウィジェット一覧を提供
   ============================================================ */
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// マニフェスト設定スキーマのフィールド
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigField {
    pub key: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub label: String,
    #[serde(default)]
    pub default: serde_json::Value,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// ウィジェットマニフェスト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetManifest {
    pub id: String,
    pub label: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_emoji")]
    pub emoji: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_interval")]
    pub update_interval: u64,
    #[serde(default)]
    pub needs_system_info: bool,
    #[serde(default)]
    pub config_schema: Vec<ConfigField>,
}

fn default_author() -> String { "community".into() }
fn default_emoji() -> String { "🧩".into() }
fn default_version() -> String { "1.0.0".into() }
fn default_interval() -> u64 { 1000 }

/// ウィジェットフォルダのパスを取得
fn get_widgets_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("widgets"))
}

/// 初回起動時: widgets/ フォルダにビルトイン + サンプルウィジェットを書き出す
#[tauri::command]
pub fn init_widgets(app: tauri::AppHandle) -> Result<(), String> {
    let widgets_dir = get_widgets_dir(&app)?;

    if !widgets_dir.exists() {
        fs::create_dir_all(&widgets_dir)
            .map_err(|e| format!("Failed to create widgets dir: {}", e))?;
    }

    // 各ウィジェットパッケージを書き出し（既存も最新スクリプトに更新）
    for (manifest, script) in all_widget_packages() {
        let pkg_dir = widgets_dir.join(&manifest.id);
        if !pkg_dir.exists() {
            fs::create_dir_all(&pkg_dir)
                .map_err(|e| format!("Failed to create widget dir: {}", e))?;
        }

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Serialize error: {}", e))?;
        fs::write(pkg_dir.join("manifest.json"), manifest_json)
            .map_err(|e| format!("Write manifest error: {}", e))?;
        fs::write(pkg_dir.join("widget.js"), script)
            .map_err(|e| format!("Write script error: {}", e))?;
    }

    // テンプレートも書き出し
    write_template(&widgets_dir)?;

    Ok(())
}

/// ウィジェット一覧を取得（widgets/ フォルダをスキャン）
#[tauri::command]
pub fn list_widgets(app: tauri::AppHandle) -> Result<Vec<WidgetManifest>, String> {
    let widgets_dir = get_widgets_dir(&app)?;
    let mut manifests: Vec<WidgetManifest> = Vec::new();

    if !widgets_dir.exists() {
        return Ok(manifests);
    }

    let entries = fs::read_dir(&widgets_dir)
        .map_err(|e| format!("Read dir error: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        // _template はスキップ
        if path.file_name().and_then(|n| n.to_str()) == Some("_template") { continue; }

        let manifest_path = path.join("manifest.json");
        if manifest_path.exists() {
            match fs::read_to_string(&manifest_path) {
                Ok(content) => match serde_json::from_str::<WidgetManifest>(&content) {
                    Ok(m) => manifests.push(m),
                    Err(e) => eprintln!("Invalid manifest {:?}: {}", manifest_path, e),
                },
                Err(e) => eprintln!("Read error {:?}: {}", manifest_path, e),
            }
        }
    }

    // ビルトインを先頭に、残りはラベル順
    let builtin_ids = ["analog-clock", "digital-clock", "countdown-timer",
                       "cpu-monitor", "memory-monitor", "date-calendar"];
    manifests.sort_by(|a, b| {
        let a_bi = builtin_ids.iter().position(|&id| id == a.id);
        let b_bi = builtin_ids.iter().position(|&id| id == b.id);
        match (a_bi, b_bi) {
            (Some(ai), Some(bi)) => ai.cmp(&bi),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.label.cmp(&b.label),
        }
    });

    Ok(manifests)
}

/// ウィジェットの widget.js スクリプトを取得
#[tauri::command]
pub fn get_widget_script(app: tauri::AppHandle, widget_id: String) -> Result<String, String> {
    let widgets_dir = get_widgets_dir(&app)?;
    let script_path = widgets_dir.join(&widget_id).join("widget.js");
    fs::read_to_string(&script_path)
        .map_err(|e| format!("Script read error for '{}': {}", widget_id, e))
}

/// ウィジェットフォルダのパスを返す
#[tauri::command]
pub fn get_widgets_dir_path(app: tauri::AppHandle) -> Result<String, String> {
    let dir = get_widgets_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

/// テンプレートフォルダを書き出す
fn write_template(widgets_dir: &PathBuf) -> Result<(), String> {
    let tmpl_dir = widgets_dir.join("_template");
    if tmpl_dir.exists() { return Ok(()); }

    fs::create_dir_all(&tmpl_dir)
        .map_err(|e| format!("Create template dir error: {}", e))?;

    let manifest = r##"{
  "id": "my-widget",
  "label": "マイウィジェット",
  "author": "あなたの名前",
  "description": "カスタムウィジェットの説明",
  "emoji": "🧩",
  "version": "1.0.0",
  "updateInterval": 1000,
  "needsSystemInfo": false,
  "configSchema": [
    { "key": "textColor", "type": "color", "label": "文字色", "default": "#ffffff" },
    { "key": "backgroundColor", "type": "color", "label": "背景色", "default": "transparent" },
    { "key": "message", "type": "text", "label": "メッセージ", "default": "Hello!" }
  ]
}"##;

    let script = r#"// ウィジェット描画関数
// ctx: CanvasRenderingContext2D - Canvas 2D コンテキスト
// w: number - キャンバスの幅 (px)
// h: number - キャンバスの高さ (px)
// config: object - manifest.json の configSchema で定義した設定値
// data: { now: Date, systemInfo?: { cpu_usage: number, memory_usage: number } }
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);

  // 背景
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }

  // テキスト描画
  ctx.fillStyle = config.textColor || '#ffffff';
  ctx.font = Math.max(8, w * 0.15) + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(config.message || 'Hello!', w / 2, h / 2);
}
"#;

    let readme = r#"# ウィジェットの作り方

## 手順
1. このフォルダをコピーしてリネーム（例: `my-cool-widget`）
2. `manifest.json` を編集:
   - `id`: フォルダ名と同じユニーク ID
   - `label`: 表示名
   - `configSchema`: 設定項目を定義
3. `widget.js` を編集:
   - `draw(ctx, w, h, config, data)` 関数に Canvas 描画コードを記述

## configSchema のフィールドタイプ
- `color`: カラーピッカー
- `checkbox`: チェックボックス
- `select`: ドロップダウン（`options` 配列を定義）
- `text`: テキスト入力
- `number`: 数値入力（`min`, `max`, `step` が使えます）
- `datetime`: 日時ピッカー

## draw() の引数
- `ctx`: CanvasRenderingContext2D
- `w`, `h`: キャンバスサイズ (px)
- `config`: configSchema で定義した設定値のオブジェクト
- `data.now`: 現在の Date オブジェクト
- `data.systemInfo`: CPU/メモリ使用率 (needsSystemInfo: true の場合)

## Tips
- Canvas API リファレンス: https://developer.mozilla.org/ja/docs/Web/API/CanvasRenderingContext2D
- `ctx.clearRect(0, 0, w, h)` を最初に呼ぶこと
- DPR (デバイスピクセル比) は呼び出し元でスケーリング済み
"#;

    fs::write(tmpl_dir.join("manifest.json"), manifest)
        .map_err(|e| format!("Write template manifest: {}", e))?;
    fs::write(tmpl_dir.join("widget.js"), script)
        .map_err(|e| format!("Write template script: {}", e))?;
    fs::write(tmpl_dir.join("README.md"), readme)
        .map_err(|e| format!("Write template readme: {}", e))?;

    Ok(())
}

/// ビルトイン＋サンプルウィジェットのパッケージ一覧
fn all_widget_packages() -> Vec<(WidgetManifest, &'static str)> {
    vec![
        // ═══════════════════ ビルトイン 6 種 ═══════════════════
        (WidgetManifest {
            id: "analog-clock".into(),
            label: "アナログ時計".into(),
            author: "builtin".into(),
            description: "カスタマイズ可能なアナログ時計".into(),
            emoji: "🕐".into(),
            version: "1.0.0".into(),
            update_interval: 1000,
            needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("#1e1e2e"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "dialStyle".into(), field_type: "select".into(), label: "文字盤".into(), default: serde_json::json!("simple"), options: vec![
                    SelectOption { value: "simple".into(), label: "シンプル".into() },
                    SelectOption { value: "roman".into(), label: "ローマ数字".into() },
                    SelectOption { value: "dots".into(), label: "ドット".into() },
                    SelectOption { value: "none".into(), label: "なし".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "hourHandColor".into(), field_type: "color".into(), label: "時針の色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "minuteHandColor".into(), field_type: "color".into(), label: "分針の色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "showSecondHand".into(), field_type: "checkbox".into(), label: "秒針を表示".into(), default: serde_json::json!(true), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "secondHandColor".into(), field_type: "color".into(), label: "秒針の色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
            ],
        }, ""), // ビルトインは widget.js 不要（コンパイル済み）

        (WidgetManifest {
            id: "digital-clock".into(),
            label: "デジタル時計".into(),
            author: "builtin".into(),
            description: "7セグメント風デジタル時計".into(),
            emoji: "⏰".into(),
            version: "1.0.0".into(),
            update_interval: 1000,
            needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "format".into(), field_type: "select".into(), label: "時刻形式".into(), default: serde_json::json!("24h"), options: vec![
                    SelectOption { value: "12h".into(), label: "12時間".into() },
                    SelectOption { value: "24h".into(), label: "24時間".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "fontStyle".into(), field_type: "select".into(), label: "フォント".into(), default: serde_json::json!("7segment"), options: vec![
                    SelectOption { value: "7segment".into(), label: "7セグメント".into() },
                    SelectOption { value: "digital".into(), label: "デジタル".into() },
                    SelectOption { value: "monospace".into(), label: "モノスペース".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "showDate".into(), field_type: "checkbox".into(), label: "日付を表示".into(), default: serde_json::json!(true), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "showWeekday".into(), field_type: "checkbox".into(), label: "曜日を表示".into(), default: serde_json::json!(false), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "showSeconds".into(), field_type: "checkbox".into(), label: "秒を表示".into(), default: serde_json::json!(false), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, ""),

        (WidgetManifest {
            id: "countdown-timer".into(),
            label: "カウントダウン".into(),
            author: "builtin".into(),
            description: "指定日時までのカウントダウン".into(),
            emoji: "⏳".into(),
            version: "1.0.0".into(),
            update_interval: 1000,
            needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "targetDate".into(), field_type: "datetime".into(), label: "目標日時".into(), default: serde_json::json!(""), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "targetLabel".into(), field_type: "text".into(), label: "ラベル".into(), default: serde_json::json!("目標"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "showDays".into(), field_type: "checkbox".into(), label: "日数を表示".into(), default: serde_json::json!(true), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "showHours".into(), field_type: "checkbox".into(), label: "時間を表示".into(), default: serde_json::json!(true), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#f9e2af"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, ""),

        (WidgetManifest {
            id: "cpu-monitor".into(),
            label: "CPU モニター".into(),
            author: "builtin".into(),
            description: "CPU 使用率のリアルタイム表示".into(),
            emoji: "📊".into(),
            version: "1.0.0".into(),
            update_interval: 2000,
            needs_system_info: true,
            config_schema: vec![
                ConfigField { key: "displayStyle".into(), field_type: "select".into(), label: "表示スタイル".into(), default: serde_json::json!("gauge"), options: vec![
                    SelectOption { value: "gauge".into(), label: "ゲージ".into() },
                    SelectOption { value: "bar".into(), label: "バー".into() },
                    SelectOption { value: "text".into(), label: "テキスト".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "gaugeColor".into(), field_type: "color".into(), label: "ゲージ色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "warningThreshold".into(), field_type: "number".into(), label: "警告閾値 (%)".into(), default: serde_json::json!(80), options: vec![], min: Some(0.0), max: Some(100.0), step: Some(1.0) },
                ConfigField { key: "warningColor".into(), field_type: "color".into(), label: "警告色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, ""),

        (WidgetManifest {
            id: "memory-monitor".into(),
            label: "メモリモニター".into(),
            author: "builtin".into(),
            description: "メモリ使用率のリアルタイム表示".into(),
            emoji: "💾".into(),
            version: "1.0.0".into(),
            update_interval: 5000,
            needs_system_info: true,
            config_schema: vec![
                ConfigField { key: "displayStyle".into(), field_type: "select".into(), label: "表示スタイル".into(), default: serde_json::json!("gauge"), options: vec![
                    SelectOption { value: "gauge".into(), label: "ゲージ".into() },
                    SelectOption { value: "bar".into(), label: "バー".into() },
                    SelectOption { value: "text".into(), label: "テキスト".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "gaugeColor".into(), field_type: "color".into(), label: "ゲージ色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "warningThreshold".into(), field_type: "number".into(), label: "警告閾値 (%)".into(), default: serde_json::json!(80), options: vec![], min: Some(0.0), max: Some(100.0), step: Some(1.0) },
                ConfigField { key: "warningColor".into(), field_type: "color".into(), label: "警告色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, ""),

        (WidgetManifest {
            id: "date-calendar".into(),
            label: "日付カレンダー".into(),
            author: "builtin".into(),
            description: "今日の日付と曜日を表示".into(),
            emoji: "📅".into(),
            version: "1.0.0".into(),
            update_interval: 60000,
            needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "showWeekday".into(), field_type: "checkbox".into(), label: "曜日を表示".into(), default: serde_json::json!(true), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "showYear".into(), field_type: "checkbox".into(), label: "年を表示".into(), default: serde_json::json!(false), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "accentColor".into(), field_type: "color".into(), label: "アクセント色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, ""),

        // ═══════════════════ サンプルウィジェット 20 種 ═══════════════════

        // 1. バイナリ時計
        (WidgetManifest {
            id: "binary-clock".into(), label: "バイナリ時計".into(), author: "sample".into(),
            description: "時刻を二進数で表示するギーク向け時計".into(), emoji: "🔢".into(),
            version: "1.0.0".into(), update_interval: 1000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "dotColor".into(), field_type: "color".into(), label: "ドット色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "offColor".into(), field_type: "color".into(), label: "OFF色".into(), default: serde_json::json!("#313244"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/binary-clock.js")),

        // 2. ストップウォッチ
        (WidgetManifest {
            id: "stopwatch".into(), label: "ストップウォッチ".into(), author: "sample".into(),
            description: "クリックで開始/停止できるストップウォッチ".into(), emoji: "⏱️".into(),
            version: "1.0.0".into(), update_interval: 100, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/stopwatch.js")),

        // 3. 世界時計
        (WidgetManifest {
            id: "world-clock".into(), label: "世界時計".into(), author: "sample".into(),
            description: "複数タイムゾーンの時刻を表示".into(), emoji: "🌍".into(),
            version: "1.0.0".into(), update_interval: 1000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "timezone".into(), field_type: "select".into(), label: "タイムゾーン".into(), default: serde_json::json!("America/New_York"), options: vec![
                    SelectOption { value: "America/New_York".into(), label: "ニューヨーク".into() },
                    SelectOption { value: "Europe/London".into(), label: "ロンドン".into() },
                    SelectOption { value: "Europe/Paris".into(), label: "パリ".into() },
                    SelectOption { value: "Asia/Shanghai".into(), label: "上海".into() },
                    SelectOption { value: "Asia/Tokyo".into(), label: "東京".into() },
                    SelectOption { value: "Australia/Sydney".into(), label: "シドニー".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#cdd6f4"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/world-clock.js")),

        // 4. 月齢表示
        (WidgetManifest {
            id: "moon-phase".into(), label: "月齢".into(), author: "sample".into(),
            description: "現在の月の満ち欠けを表示".into(), emoji: "🌙".into(),
            version: "1.0.0".into(), update_interval: 3600000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "moonColor".into(), field_type: "color".into(), label: "月の色".into(), default: serde_json::json!("#f9e2af"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("#1e1e2e"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/moon-phase.js")),

        // 5. 天気アイコン（アニメーション）
        (WidgetManifest {
            id: "weather-icon".into(), label: "天気アニメ".into(), author: "sample".into(),
            description: "天気をアニメーションで表示".into(), emoji: "🌤️".into(),
            version: "1.0.0".into(), update_interval: 50, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "weatherType".into(), field_type: "select".into(), label: "天気".into(), default: serde_json::json!("sunny"), options: vec![
                    SelectOption { value: "sunny".into(), label: "晴れ".into() },
                    SelectOption { value: "cloudy".into(), label: "曇り".into() },
                    SelectOption { value: "rainy".into(), label: "雨".into() },
                    SelectOption { value: "snowy".into(), label: "雪".into() },
                    SelectOption { value: "stormy".into(), label: "雷".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/weather-icon.js")),

        // 6. バッテリー表示
        (WidgetManifest {
            id: "battery-level".into(), label: "バッテリー".into(), author: "sample".into(),
            description: "バッテリー残量をアイコンで表示".into(), emoji: "🔋".into(),
            version: "1.0.0".into(), update_interval: 30000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "fullColor".into(), field_type: "color".into(), label: "満充電色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "lowColor".into(), field_type: "color".into(), label: "低残量色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/battery-level.js")),

        // 7. ポモドーロタイマー
        (WidgetManifest {
            id: "pomodoro".into(), label: "ポモドーロ".into(), author: "sample".into(),
            description: "25分作業/5分休憩のポモドーロタイマー".into(), emoji: "🍅".into(),
            version: "1.0.0".into(), update_interval: 1000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "workMinutes".into(), field_type: "number".into(), label: "作業時間 (分)".into(), default: serde_json::json!(25), options: vec![], min: Some(1.0), max: Some(120.0), step: Some(1.0) },
                ConfigField { key: "breakMinutes".into(), field_type: "number".into(), label: "休憩時間 (分)".into(), default: serde_json::json!(5), options: vec![], min: Some(1.0), max: Some(30.0), step: Some(1.0) },
                ConfigField { key: "soundEnabled".into(), field_type: "checkbox".into(), label: "通知音を鳴らす".into(), default: serde_json::json!(true), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "soundType".into(), field_type: "select".into(), label: "通知音の種類".into(), default: serde_json::json!("beep"), options: vec![
                    SelectOption { label: "ビープ音".into(), value: "beep".into() },
                    SelectOption { label: "音声ファイル".into(), value: "custom".into() },
                ], min: None, max: None, step: None },
                ConfigField { key: "soundFile".into(), field_type: "file".into(), label: "音声ファイル (mp3, wav 等)".into(), default: serde_json::json!(""), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "soundVolume".into(), field_type: "number".into(), label: "音量 (0-100)".into(), default: serde_json::json!(50), options: vec![], min: Some(0.0), max: Some(100.0), step: Some(5.0) },
                ConfigField { key: "soundMaxSeconds".into(), field_type: "number".into(), label: "最大再生秒数 (0=無制限)".into(), default: serde_json::json!(10), options: vec![], min: Some(0.0), max: Some(300.0), step: Some(5.0) },
                ConfigField { key: "workColor".into(), field_type: "color".into(), label: "作業色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "breakColor".into(), field_type: "color".into(), label: "休憩色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/pomodoro.js")),

        // 8. ネットワークモニター
        (WidgetManifest {
            id: "network-status".into(), label: "ネットワーク".into(), author: "sample".into(),
            description: "ネットワーク接続状態を表示".into(), emoji: "📶".into(),
            version: "1.0.0".into(), update_interval: 5000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "onlineColor".into(), field_type: "color".into(), label: "接続色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "offlineColor".into(), field_type: "color".into(), label: "切断色".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/network-status.js")),

        // 9. 年間プログレスバー
        (WidgetManifest {
            id: "year-progress".into(), label: "年間進捗".into(), author: "sample".into(),
            description: "今年の進捗をパーセントで表示".into(), emoji: "📈".into(),
            version: "1.0.0".into(), update_interval: 60000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "barColor".into(), field_type: "color".into(), label: "バー色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#cdd6f4"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/year-progress.js")),

        // 10. 日の出日の入り 
        (WidgetManifest {
            id: "daylight-info".into(), label: "日照時間".into(), author: "sample".into(),
            description: "日の出・日の入り時刻と昼の長さ".into(), emoji: "🌅".into(),
            version: "1.0.0".into(), update_interval: 60000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "latitude".into(), field_type: "number".into(), label: "緯度".into(), default: serde_json::json!(35.68), options: vec![], min: Some(-90.0), max: Some(90.0), step: Some(0.01) },
                ConfigField { key: "longitude".into(), field_type: "number".into(), label: "経度".into(), default: serde_json::json!(139.77), options: vec![], min: Some(-180.0), max: Some(180.0), step: Some(0.01) },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#f9e2af"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/daylight-info.js")),

        // 11. サイコロ
        (WidgetManifest {
            id: "dice".into(), label: "サイコロ".into(), author: "sample".into(),
            description: "クリックで振れるサイコロ".into(), emoji: "🎲".into(),
            version: "1.0.0".into(), update_interval: 60000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "diceColor".into(), field_type: "color".into(), label: "サイコロ色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "dotColor".into(), field_type: "color".into(), label: "ドット色".into(), default: serde_json::json!("#1e1e2e"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/dice.js")),

        // 12. パルスアニメーション
        (WidgetManifest {
            id: "pulse-animation".into(), label: "パルス".into(), author: "sample".into(),
            description: "美しいパルスアニメーション".into(), emoji: "💫".into(),
            version: "1.0.0".into(), update_interval: 30, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "color1".into(), field_type: "color".into(), label: "色1".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "color2".into(), field_type: "color".into(), label: "色2".into(), default: serde_json::json!("#f38ba8"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "speed".into(), field_type: "number".into(), label: "速度".into(), default: serde_json::json!(2), options: vec![], min: Some(0.5), max: Some(10.0), step: Some(0.5) },
            ],
        }, include_str!("../../../widget_scripts/pulse-animation.js")),

        // 13. メモ帳
        (WidgetManifest {
            id: "quick-note".into(), label: "メモ".into(), author: "sample".into(),
            description: "簡単なメモを表示".into(), emoji: "📝".into(),
            version: "1.0.0".into(), update_interval: 60000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "note".into(), field_type: "text".into(), label: "メモ内容".into(), default: serde_json::json!("メモ"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#cdd6f4"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("#313244"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/quick-note.js")),

        // 14. CPU+メモリダブルゲージ
        (WidgetManifest {
            id: "dual-monitor".into(), label: "デュアルモニタ".into(), author: "sample".into(),
            description: "CPU+メモリを1つのウィジェットに表示".into(), emoji: "📊".into(),
            version: "1.0.0".into(), update_interval: 2000, needs_system_info: true,
            config_schema: vec![
                ConfigField { key: "cpuColor".into(), field_type: "color".into(), label: "CPU色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "memColor".into(), field_type: "color".into(), label: "メモリ色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#ffffff"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/dual-monitor.js")),

        // 15. マトリックス風エフェクト
        (WidgetManifest {
            id: "matrix-rain".into(), label: "マトリックス".into(), author: "sample".into(),
            description: "マトリックス風のデジタルレインエフェクト".into(), emoji: "🟢".into(),
            version: "1.0.0".into(), update_interval: 60, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "charColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#00ff00"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("#000000"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/matrix-rain.js")),

        // 16. 今日の格言
        (WidgetManifest {
            id: "daily-quote".into(), label: "今日の格言".into(), author: "sample".into(),
            description: "日替わりの名言・格言を表示".into(), emoji: "💬".into(),
            version: "1.0.0".into(), update_interval: 3600000, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#cdd6f4"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/daily-quote.js")),

        // 17. 色相ホイール
        (WidgetManifest {
            id: "color-wheel".into(), label: "色相ホイール".into(), author: "sample".into(),
            description: "回転する虹色の色相ホイール".into(), emoji: "🌈".into(),
            version: "1.0.0".into(), update_interval: 30, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "speed".into(), field_type: "number".into(), label: "回転速度".into(), default: serde_json::json!(1), options: vec![], min: Some(0.1), max: Some(5.0), step: Some(0.1) },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/color-wheel.js")),

        // 18. 呼吸ガイド
        (WidgetManifest {
            id: "breathing-guide".into(), label: "呼吸ガイド".into(), author: "sample".into(),
            description: "吸って・止めて・吐いてのリズムガイド".into(), emoji: "🧘".into(),
            version: "1.0.0".into(), update_interval: 50, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "inhaleSeconds".into(), field_type: "number".into(), label: "吸う秒数".into(), default: serde_json::json!(4), options: vec![], min: Some(2.0), max: Some(10.0), step: Some(1.0) },
                ConfigField { key: "holdSeconds".into(), field_type: "number".into(), label: "止める秒数".into(), default: serde_json::json!(4), options: vec![], min: Some(0.0), max: Some(10.0), step: Some(1.0) },
                ConfigField { key: "exhaleSeconds".into(), field_type: "number".into(), label: "吐く秒数".into(), default: serde_json::json!(6), options: vec![], min: Some(2.0), max: Some(10.0), step: Some(1.0) },
                ConfigField { key: "circleColor".into(), field_type: "color".into(), label: "円の色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#cdd6f4"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/breathing-guide.js")),

        // 19. FPS カウンター
        (WidgetManifest {
            id: "fps-counter".into(), label: "FPS カウンタ".into(), author: "sample".into(),
            description: "描画フレームレートを表示".into(), emoji: "🎮".into(),
            version: "1.0.0".into(), update_interval: 100, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "textColor".into(), field_type: "color".into(), label: "文字色".into(), default: serde_json::json!("#a6e3a1"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("transparent"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/fps-counter.js")),

        // 20. ランダムパーティクル
        (WidgetManifest {
            id: "particles".into(), label: "パーティクル".into(), author: "sample".into(),
            description: "浮遊するパーティクルエフェクト".into(), emoji: "✨".into(),
            version: "1.0.0".into(), update_interval: 30, needs_system_info: false,
            config_schema: vec![
                ConfigField { key: "particleColor".into(), field_type: "color".into(), label: "粒子色".into(), default: serde_json::json!("#89b4fa"), options: vec![], min: None, max: None, step: None },
                ConfigField { key: "count".into(), field_type: "number".into(), label: "粒子数".into(), default: serde_json::json!(30), options: vec![], min: Some(5.0), max: Some(100.0), step: Some(5.0) },
                ConfigField { key: "backgroundColor".into(), field_type: "color".into(), label: "背景色".into(), default: serde_json::json!("#1e1e2e"), options: vec![], min: None, max: None, step: None },
            ],
        }, include_str!("../../../widget_scripts/particles.js")),
    ]
}
