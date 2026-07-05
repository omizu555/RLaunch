//! 設定ウィンドウのビュー。変更は即時反映・即時保存（フォントのみ再起動後）。
//!
//! セクション構成の方針:
//! - 「動作」= 呼び出し方と消え方（最もよく触る）
//! - 「外観」= 見た目（テーマ・フォント・セル・タイトル）
//! - 「新規タブの既定」= 新しいタブを作るときの初期値。既存タブには影響しない
//!   （既存タブの列/行/表示モードはタブ右クリック→タブ設定で個別に変更する）
//! - 「データ」= バックアップ・移行

use crate::app::{App, FormMsg, Message, Overlay, SettingsMsg};
use crate::ui::style;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, slider, text, text_input,
    Space,
};
use iced::{Alignment, Element, Length};

/// フォント候補（Windows 標準搭載の日本語対応フォント中心）
const FONT_DEFAULT_LABEL: &str = "既定 (Yu Gothic UI)";
const FONT_CHOICES: [&str; 7] = [
    FONT_DEFAULT_LABEL,
    "Meiryo UI",
    "メイリオ",
    "BIZ UDPゴシック",
    "MS UI Gothic",
    "Yu Gothic",
    "Segoe UI",
];

fn section<'a>(app: &'a App, title: &'a str) -> Element<'a, Message> {
    text(title.to_string()).size(13).color(app.ui.accent).into()
}

fn note(app: &App, body: impl Into<String>) -> Element<'_, Message> {
    text(body.into()).size(9).color(app.ui.text_muted).into()
}

fn setting_row<'a>(
    app: &'a App,
    label: impl Into<String>,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    row![
        text(label.into())
            .size(11)
            .color(app.ui.text_secondary)
            .width(Length::Fixed(130.0)),
        control,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let s = &app.data.settings;

    // テーマ選択
    let theme_labels: Vec<String> = app
        .themes
        .iter()
        .map(|t| format!("{} ({})", t.label, t.id))
        .collect();
    let selected_theme = app
        .themes
        .iter()
        .find(|t| t.id == s.theme)
        .map(|t| format!("{} ({})", t.label, t.id));

    // フォント選択
    let font_labels: Vec<String> = FONT_CHOICES.iter().map(|f| f.to_string()).collect();
    let selected_font = s
        .font_family
        .clone()
        .unwrap_or_else(|| FONT_DEFAULT_LABEL.to_string());

    let view_modes = ["grid".to_string(), "list".to_string()];
    let window_positions = [
        "cursor".to_string(),
        "center".to_string(),
        "remember".to_string(),
    ];

    let content = column![
        // ---------------- 動作 ----------------
        section(app, "動作"),
        setting_row(
            app,
            "ホットキー",
            row![
                text_input("Ctrl+Space", &app.settings_hotkey_draft)
                    .on_input(|v| Message::Settings(SettingsMsg::HotkeyInput(v)))
                    .on_submit(Message::Settings(SettingsMsg::HotkeyApply))
                    .size(11)
                    .style(style::input(ui)),
                button(text("適用").size(11))
                    .style(style::dialog_button(ui, true))
                    .padding([4, 10])
                    .on_press(Message::Settings(SettingsMsg::HotkeyApply)),
            ]
            .spacing(4)
            .into(),
        ),
        setting_row(
            app,
            "表示位置",
            pick_list(
                window_positions.to_vec(),
                Some(s.window_position.clone()),
                |v| Message::Settings(SettingsMsg::WindowPos(v)),
            )
            .text_size(11)
            .into(),
        ),
        setting_row(
            app,
            "自動非表示",
            checkbox(s.auto_hide)
                .label("フォーカスを失ったら隠す")
                .size(14)
                .text_size(11)
                .on_toggle(|v| Message::Settings(SettingsMsg::AutoHide(v)))
                .into(),
        ),
        setting_row(
            app,
            "",
            checkbox(s.hide_on_cursor_out)
                .label("カーソルが外に出たら隠す (CLaunch風)")
                .size(14)
                .text_size(11)
                .on_toggle(|v| Message::Settings(SettingsMsg::HideOnCursorOut(v)))
                .into(),
        ),
        setting_row(
            app,
            "",
            checkbox(s.hide_on_launch)
                .label("アイテム起動後に隠す")
                .size(14)
                .text_size(11)
                .on_toggle(|v| Message::Settings(SettingsMsg::HideOnLaunch(v)))
                .into(),
        ),
        setting_row(
            app,
            "自動起動",
            checkbox(s.auto_start)
                .label("Windows 起動時に実行")
                .size(14)
                .text_size(11)
                .on_toggle(|v| Message::Settings(SettingsMsg::AutoStart(v)))
                .into(),
        ),
        // ---------------- 外観 ----------------
        section(app, "外観"),
        setting_row(
            app,
            "テーマ",
            pick_list(theme_labels, selected_theme, |v| {
                // "ラベル (id)" から id を取り出す
                let id = v
                    .rsplit_once('(')
                    .map(|(_, rest)| rest.trim_end_matches(')').to_string())
                    .unwrap_or(v);
                Message::Settings(SettingsMsg::ThemeSelected(id))
            })
            .text_size(11)
            .width(Length::Fill)
            .into(),
        ),
        setting_row(
            app,
            "フォント",
            pick_list(font_labels, Some(selected_font), |v| {
                let name = if v == FONT_DEFAULT_LABEL { None } else { Some(v) };
                Message::Settings(SettingsMsg::FontFamily(name))
            })
            .text_size(11)
            .width(Length::Fill)
            .into(),
        ),
        note(app, "フォントの変更はアプリの再起動後に反映されます"),
        setting_row(
            app,
            format!("セルサイズ: {}px", s.cell_size),
            slider(40..=120u32, s.cell_size, |v| {
                Message::Settings(SettingsMsg::CellSize(v / 4 * 4))
            })
            .into(),
        ),
        setting_row(
            app,
            "ボタンラベル",
            checkbox(s.show_labels)
                .label("表示する")
                .size(14)
                .text_size(11)
                .on_toggle(|v| Message::Settings(SettingsMsg::ShowLabels(v)))
                .into(),
        ),
        setting_row(
            app,
            format!("ラベルサイズ: {}px", s.label_font_size),
            slider(8..=16u32, s.label_font_size, |v| {
                Message::Settings(SettingsMsg::LabelFontSize(v))
            })
            .into(),
        ),
        setting_row(
            app,
            "ウィンドウタイトル",
            text_input("RLaunch", &s.app_title)
                .on_input(|v| Message::Settings(SettingsMsg::AppTitle(v)))
                .size(11)
                .style(style::input(ui))
                .into(),
        ),
        row![
            Space::new().width(130.0),
            button(text("📁 テーマフォルダを開く").size(11))
                .style(style::dialog_button(ui, false))
                .padding([4, 10])
                .on_press(Message::Settings(SettingsMsg::OpenThemesDir)),
        ]
        .spacing(8),
        // ---------------- 新規タブの既定 ----------------
        section(app, "新規タブの既定"),
        note(
            app,
            "新しくタブを作るときの初期値です。既存のタブには影響しません\n（既存タブの列・行・表示モードは、タブを右クリック → タブ設定 で個別に変更できます）",
        ),
        setting_row(
            app,
            format!("列数: {}", s.default_grid_columns),
            slider(1..=20u32, s.default_grid_columns, |v| {
                Message::Settings(SettingsMsg::GridCols(v))
            })
            .into(),
        ),
        setting_row(
            app,
            format!("行数: {}", s.default_grid_rows),
            slider(1..=10u32, s.default_grid_rows, |v| {
                Message::Settings(SettingsMsg::GridRows(v))
            })
            .into(),
        ),
        setting_row(
            app,
            "表示モード",
            pick_list(view_modes.to_vec(), Some(s.view_mode.clone()), |v| {
                Message::Settings(SettingsMsg::ViewMode(v))
            })
            .text_size(11)
            .into(),
        ),
        setting_row(
            app,
            format!("リスト列数: {}", s.list_columns),
            slider(1..=4u32, s.list_columns, |v| {
                Message::Settings(SettingsMsg::ListColumns(v))
            })
            .into(),
        ),
        // ---------------- データ ----------------
        section(app, "データ"),
        row![
            Space::new().width(130.0),
            button(text("📋 コピー").size(11))
                .style(style::dialog_button(ui, false))
                .padding([4, 10])
                .on_press(Message::Settings(SettingsMsg::CopyData)),
            button(text("📤 エクスポート").size(11))
                .style(style::dialog_button(ui, false))
                .padding([4, 10])
                .on_press(Message::Settings(SettingsMsg::Export)),
            button(text("📥 インポート").size(11))
                .style(style::dialog_button(ui, false))
                .padding([4, 10])
                .on_press(Message::Settings(SettingsMsg::Import)),
            button(text("📁 データフォルダ").size(11))
                .style(style::dialog_button(ui, false))
                .padding([4, 10])
                .on_press(Message::Settings(SettingsMsg::OpenDataDir)),
        ]
        .spacing(6),
        note(
            app,
            format!("保存先: {}", crate::model::store::data_file_path().display()),
        ),
    ]
    .spacing(10)
    .padding(16);

    let base = container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::window_root(ui));

    // インポート方法の選択はこのウィンドウに重ねて表示
    if matches!(app.overlay, Overlay::ImportChoice { .. }) {
        let panel = container(
            column![
                text("インポート方法を選択してください")
                    .size(13)
                    .color(ui.text_primary),
                row![
                    button(text("置換").size(12))
                        .style(style::danger_button(ui))
                        .padding([6, 16])
                        .on_press(Message::Form(FormMsg::ImportReplace)),
                    button(text("マージ").size(12))
                        .style(style::dialog_button(ui, true))
                        .padding([6, 16])
                        .on_press(Message::Form(FormMsg::ImportMerge)),
                    button(text("キャンセル").size(12))
                        .style(style::dialog_button(ui, false))
                        .padding([6, 16])
                        .on_press(Message::OverlayCancel),
                ]
                .spacing(8),
            ]
            .spacing(12),
        )
        .padding(16)
        .style(style::panel(ui));
        return iced::widget::stack![
            base,
            container(panel)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(style::scrim()),
        ]
        .into();
    }

    base.into()
}
