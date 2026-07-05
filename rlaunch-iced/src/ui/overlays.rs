//! オーバーレイ群: コンテキストメニュー / 各種ダイアログ / 検索 / トースト。

use crate::app::{App, CtxAction, CtxTarget, FormMsg, GridRef, Message, Overlay};
use crate::model::data::GridCell;
use crate::ui::style;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, row, slider, text, text_input,
    Space,
};
use iced::{Alignment, Element, Length};

const GROUP_EMOJIS: [&str; 16] = [
    "📂", "📁", "🗂", "⭐", "🔧", "🎮", "🎵", "🎬", "📷", "💼", "🌐", "📝", "🎨", "📊", "🔒", "❤",
];
const GROUP_COLORS: [&str; 8] = [
    "#e74c3c", "#e67e22", "#f1c40f", "#2ecc71", "#3498db", "#9b59b6", "#e91e63", "#95a5a6",
];

/// 指定位置に置くヘルパー（stack 内で padding により位置決め）。
/// est_w/est_h は要素の推定サイズで、ウィンドウからはみ出す場合はクランプする。
fn positioned<'a>(
    el: Element<'a, Message>,
    x: f32,
    y: f32,
    est_w: f32,
    est_h: f32,
    bounds: iced::Size,
) -> Element<'a, Message> {
    let x = x.min(bounds.width - est_w).max(0.0);
    let y = y.min(bounds.height - est_h).max(0.0);
    container(el)
        .padding(iced::Padding {
            top: y,
            left: x,
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// コンテキストメニュー
pub fn context_menu(app: &App) -> Element<'_, Message> {
    let Some(menu) = &app.ctx_menu else {
        return text("").into();
    };
    let ui = &app.ui;

    let mut items: Vec<(String, Message, bool)> = Vec::new(); // (label, msg, danger)
    match menu.target {
        CtxTarget::Cell(grid, idx) => {
            let cell = app
                .cells(grid)
                .and_then(|c| c.get(idx))
                .and_then(|c| c.as_ref());
            match cell {
                Some(GridCell::Launcher(item)) => {
                    items.push((
                        "▶ 起動".into(),
                        Message::Ctx(menu.target, CtxAction::Launch),
                        false,
                    ));
                    if !item.is_url() {
                        items.push((
                            "🛡 管理者として起動".into(),
                            Message::Ctx(menu.target, CtxAction::RunAsAdmin),
                            false,
                        ));
                    }
                    if item.is_folder() {
                        items.push((
                            "🗂 内蔵ブラウザで参照".into(),
                            Message::Ctx(menu.target, CtxAction::BrowseFolder),
                            false,
                        ));
                        let cur = item.folder_action.as_deref().unwrap_or("open");
                        let label = if cur == "browse" {
                            "🔄 クリック動作: 参照 → 開く"
                        } else {
                            "🔄 クリック動作: 開く → 参照"
                        };
                        items.push((
                            label.into(),
                            Message::Ctx(menu.target, CtxAction::ToggleFolderAction),
                            false,
                        ));
                    }
                    if !item.is_url() {
                        items.push((
                            "📂 ファイルの場所を開く".into(),
                            Message::Ctx(menu.target, CtxAction::OpenLocation),
                            false,
                        ));
                    }
                    items.push((
                        "✏ 編集".into(),
                        Message::Ctx(menu.target, CtxAction::EditItem),
                        false,
                    ));
                    items.push((
                        "🗑 登録解除".into(),
                        Message::Ctx(menu.target, CtxAction::RemoveItem),
                        true,
                    ));
                }
                Some(GridCell::Group(_)) => {
                    items.push((
                        "📂 開く".into(),
                        Message::Ctx(menu.target, CtxAction::OpenGroup),
                        false,
                    ));
                    items.push((
                        "✏ 編集".into(),
                        Message::Ctx(menu.target, CtxAction::EditGroup),
                        false,
                    ));
                    items.push((
                        "🗑 削除".into(),
                        Message::Ctx(menu.target, CtxAction::RemoveItem),
                        true,
                    ));
                }
                Some(GridCell::Widget(_)) => {
                    items.push((
                        "🗑 解除（ウィジェット機能は廃止）".into(),
                        Message::Ctx(menu.target, CtxAction::RemoveItem),
                        true,
                    ));
                }
                None => {
                    items.push((
                        "📁 ファイルを選択して追加".into(),
                        Message::Ctx(menu.target, CtxAction::RegisterFile),
                        false,
                    ));
                    items.push((
                        "📂 フォルダを選択して追加".into(),
                        Message::Ctx(menu.target, CtxAction::RegisterFolder),
                        false,
                    ));
                    items.push((
                        "🌐 URLを登録".into(),
                        Message::Ctx(menu.target, CtxAction::RegisterUrl),
                        false,
                    ));
                    // グループはメイングリッドのみ（ネスト不可）
                    if matches!(grid, GridRef::Tab(_)) {
                        items.push((
                            "🗂 サブグループを作成".into(),
                            Message::Ctx(menu.target, CtxAction::CreateGroup),
                            false,
                        ));
                    }
                }
            }
        }
        CtxTarget::Tab(_) => {
            items.push((
                "⚙ タブ設定（名前・サイズ）".into(),
                Message::Ctx(menu.target, CtxAction::TabSettings),
                false,
            ));
            items.push((
                "📋 タブを複製".into(),
                Message::Ctx(menu.target, CtxAction::TabDuplicate),
                false,
            ));
            items.push((
                "🗑 タブを削除".into(),
                Message::Ctx(menu.target, CtxAction::TabDelete),
                true,
            ));
        }
    }

    let item_count = items.len();
    let mut menu_col = column![].spacing(1);
    for (label, msg, danger) in items {
        menu_col = menu_col.push(
            button(text(label).size(12))
                .style(style::menu_item(ui, danger))
                .padding([5, 12])
                .width(Length::Fill)
                .on_press(msg),
        );
    }

    // メニューの推定サイズ（項目高 ~27px + パディング）とウィンドウサイズでクランプ
    let est_h = item_count as f32 * 27.0 + 12.0;
    let bounds = match menu.target {
        CtxTarget::Cell(grid, _) if app.popup_grid() == Some(grid) => {
            // ポップアップウィンドウ内: グリッド構成からサイズを再計算
            crate::ui::grid::GridParams::resolve(app, grid)
                .map(|p| {
                    let (w, h) = p.inner_size(app.data.settings.cell_size as f32);
                    iced::Size::new(
                        w + crate::app::layout::GRID_PADDING,
                        h + crate::app::layout::GRID_PADDING
                            + crate::app::layout::POPUP_HEADER_HEIGHT,
                    )
                })
                .unwrap_or(iced::Size::new(400.0, 300.0))
        }
        _ => crate::app::compute_main_size(&app.data, app.active_tab),
    };

    let panel = container(menu_col)
        .width(Length::Fixed(220.0))
        .padding(4)
        .style(style::panel(ui));

    positioned(panel.into(), menu.at.x, menu.at.y, 220.0, est_h, bounds)
}

/// モーダルオーバーレイ（ダイアログ）
pub fn modal_overlay(app: &App) -> Option<Element<'_, Message>> {
    let ui = &app.ui;
    let content: Element<'_, Message> = match &app.overlay {
        Overlay::None => return None,
        Overlay::ItemEdit { form, .. } => {
            let states = ["normal".to_string(), "maximized".to_string(), "minimized".to_string()];
            column![
                text("アイテムの編集").size(14).color(ui.text_primary),
                labeled_input(app, "名前", &form.label, |v| Message::Form(FormMsg::ItemLabel(v))),
                labeled_input(app, "パス", &form.path, |v| Message::Form(FormMsg::ItemPath(v))),
                labeled_input(app, "引数", &form.args, |v| Message::Form(FormMsg::ItemArgs(v))),
                labeled_input(app, "作業フォルダ", &form.working_dir, |v| {
                    Message::Form(FormMsg::ItemWorkDir(v))
                }),
                labeled_input(app, "ホットキー (例 Ctrl+Alt+N)", &form.hotkey, |v| {
                    Message::Form(FormMsg::ItemHotkey(v))
                }),
                row![
                    text("実行時の大きさ").size(11).color(ui.text_secondary).width(Length::Fixed(110.0)),
                    pick_list(states.to_vec(), Some(form.window_state.clone()), |v| {
                        Message::Form(FormMsg::ItemWindowState(v))
                    })
                    .text_size(11),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                checkbox(form.run_as)
                    .label("管理者として実行")
                    .size(14)
                    .text_size(12)
                    .on_toggle(|v| Message::Form(FormMsg::ItemRunAs(v))),
                row![
                    button(text("🖼 画像をアイコンにする").size(11))
                        .style(style::dialog_button(ui, false))
                        .padding([4, 10])
                        .on_press(Message::Form(FormMsg::PickImage)),
                    text(if form.icon_override.is_some() { "画像設定済み" } else { "" })
                        .size(10)
                        .color(ui.success),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(form.stats.clone()).size(10).color(ui.text_muted),
                dialog_buttons(app, "保存"),
            ]
            .spacing(8)
            .into()
        }
        Overlay::GroupEdit { form, existing, .. } => {
            let mut emoji_row1 = row![].spacing(2);
            let mut emoji_row2 = row![].spacing(2);
            for (i, e) in GROUP_EMOJIS.iter().enumerate() {
                let b = button(text(e.to_string()).size(14))
                    .style(style::icon_button(ui, form.icon == *e))
                    .padding(4)
                    .on_press(Message::Form(FormMsg::GroupIcon(e.to_string())));
                if i < 8 {
                    emoji_row1 = emoji_row1.push(b);
                } else {
                    emoji_row2 = emoji_row2.push(b);
                }
            }
            let mut color_row = row![].spacing(2);
            color_row = color_row.push(
                button(text("なし").size(10))
                    .style(style::icon_button(ui, form.icon_color.is_none()))
                    .padding(4)
                    .on_press(Message::Form(FormMsg::GroupColor(None))),
            );
            for c in GROUP_COLORS {
                let color = crate::model::theme::parse_color(c).unwrap_or(iced::Color::WHITE);
                let selected = form.icon_color.as_deref() == Some(c);
                color_row = color_row.push(
                    button(text("●").size(12).color(color))
                        .style(style::icon_button(ui, selected))
                        .padding(4)
                        .on_press(Message::Form(FormMsg::GroupColor(Some(c.to_string())))),
                );
            }
            column![
                text(if *existing { "グループの編集" } else { "グループの作成" })
                    .size(14)
                    .color(ui.text_primary),
                labeled_input(app, "名前", &form.label, |v| {
                    Message::Form(FormMsg::GroupLabel(v))
                }),
                text("アイコン").size(11).color(ui.text_secondary),
                emoji_row1,
                emoji_row2,
                text("カラー").size(11).color(ui.text_secondary),
                color_row,
                row![
                    text(format!("列数: {}", form.cols)).size(11).color(ui.text_secondary).width(Length::Fixed(80.0)),
                    slider(1..=8u32, form.cols, |v| Message::Form(FormMsg::GroupCols(v))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text(format!("行数: {}", form.rows)).size(11).color(ui.text_secondary).width(Length::Fixed(80.0)),
                    slider(1..=6u32, form.rows, |v| Message::Form(FormMsg::GroupRows(v))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(format!("スロット数: {}", form.cols * form.rows))
                    .size(10)
                    .color(ui.text_muted),
                {
                    let modes = ["親設定を使用".to_string(), "グリッド".to_string(), "リスト".to_string()];
                    let selected = match form.view_mode.as_deref() {
                        Some("grid") => "グリッド".to_string(),
                        Some("list") => "リスト".to_string(),
                        _ => "親設定を使用".to_string(),
                    };
                    row![
                        text("表示モード").size(11).color(ui.text_secondary).width(Length::Fixed(80.0)),
                        pick_list(modes.to_vec(), Some(selected), |v| {
                            let vm = match v.as_str() {
                                "グリッド" => Some("grid".to_string()),
                                "リスト" => Some("list".to_string()),
                                _ => None,
                            };
                            Message::Form(FormMsg::GroupViewMode(vm))
                        })
                        .text_size(11),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                },
                row![
                    text(format!(
                        "リスト列数: {}",
                        form.list_columns.map_or("親設定".to_string(), |v| v.to_string())
                    ))
                    .size(11)
                    .color(ui.text_secondary)
                    .width(Length::Fixed(80.0)),
                    slider(0..=4u32, form.list_columns.unwrap_or(0), |v| {
                        Message::Form(FormMsg::GroupListCols(if v == 0 { None } else { Some(v) }))
                    }),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    button(text("🖼 画像をアイコンにする").size(11))
                        .style(style::dialog_button(ui, false))
                        .padding([4, 10])
                        .on_press(Message::Form(FormMsg::PickImage)),
                    text(if form.icon_override.is_some() { "画像設定済み" } else { "" })
                        .size(10)
                        .color(ui.success),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                dialog_buttons(app, "保存"),
            ]
            .spacing(8)
            .into()
        }
        Overlay::TabSettings { form, .. } => {
            let modes = ["全体設定を使用".to_string(), "グリッド".to_string(), "リスト".to_string()];
            let selected_mode = match form.view_mode.as_deref() {
                Some("grid") => "グリッド".to_string(),
                Some("list") => "リスト".to_string(),
                _ => "全体設定を使用".to_string(),
            };
            column![
                text("タブ設定").size(14).color(ui.text_primary),
                labeled_input(app, "タブ名", &form.label, |v| {
                    Message::Form(FormMsg::TabLabel(v))
                }),
                row![
                    text(format!("列数: {}", form.cols)).size(11).color(ui.text_secondary).width(Length::Fixed(80.0)),
                    slider(1..=20u32, form.cols, |v| Message::Form(FormMsg::TabCols(v))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text(format!("行数: {}", form.rows)).size(11).color(ui.text_secondary).width(Length::Fixed(80.0)),
                    slider(1..=10u32, form.rows, |v| Message::Form(FormMsg::TabRows(v))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text("表示モード").size(11).color(ui.text_secondary).width(Length::Fixed(80.0)),
                    pick_list(modes.to_vec(), Some(selected_mode), |v| {
                        let vm = match v.as_str() {
                            "グリッド" => Some("grid".to_string()),
                            "リスト" => Some("list".to_string()),
                            _ => None,
                        };
                        Message::Form(FormMsg::TabViewMode(vm))
                    })
                    .text_size(11),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    text(format!("リスト列数: {}", form.list_columns))
                        .size(11)
                        .color(ui.text_secondary)
                        .width(Length::Fixed(80.0)),
                    slider(1..=4u32, form.list_columns, |v| {
                        Message::Form(FormMsg::TabListCols(v))
                    }),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                dialog_buttons(app, "保存"),
            ]
            .spacing(8)
            .into()
        }
        Overlay::UrlPrompt { url, .. } => column![
            text("URL を登録").size(14).color(ui.text_primary),
            text_input("https://example.com", url)
                .on_input(|v| Message::Form(FormMsg::UrlChanged(v)))
                .on_submit(Message::Form(FormMsg::Save))
                .size(12)
                .style(style::input(ui)),
            dialog_buttons(app, "登録"),
        ]
        .spacing(8)
        .into(),
        Overlay::ConfirmClear { label, .. } => column![
            text(format!("「{}」の登録を解除しますか？", label))
                .size(13)
                .color(ui.text_primary),
            confirm_buttons(app, "解除"),
        ]
        .spacing(12)
        .into(),
        Overlay::ConfirmTabDelete { tab } => {
            let (label, count) = app
                .data
                .tabs
                .get(*tab)
                .map(|t| (t.label.clone(), t.item_count()))
                .unwrap_or_default();
            column![
                text(format!(
                    "タブ「{}」を削除しますか？（{} アイテムが失われます）",
                    label, count
                ))
                .size(13)
                .color(ui.text_primary),
                confirm_buttons(app, "削除"),
            ]
            .spacing(12)
            .into()
        }
        Overlay::ImportChoice { .. } => column![
            text("インポート方法を選択してください").size(13).color(ui.text_primary),
            text("置換: 現在のデータを破棄して読み込みます\nマージ: 既存タブを残し、新しいタブを追加します")
                .size(11)
                .color(ui.text_secondary),
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
        .spacing(12)
        .into(),
    };

    let panel = container(content)
        .padding(16)
        .width(Length::Fixed(360.0))
        .style(style::panel(ui));

    Some(
        // スクリムでマウス操作をキャプチャし、ダイアログ外のタイトルバー/タブ等への
        // クリックが素通りしないようにする（ホイールとキーは update 側でガード）
        mouse_area(
            container(panel)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(style::scrim()),
        )
        .on_press(Message::Noop)
        .on_right_press(Message::Noop)
        .on_release(Message::Noop)
        .into(),
    )
}

fn labeled_input<'a>(
    app: &'a App,
    label: &'a str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(11).color(app.ui.text_secondary),
        text_input("", value)
            .on_input(on_input)
            .size(12)
            .style(style::input(&app.ui)),
    ]
    .spacing(2)
    .into()
}

fn dialog_buttons<'a>(app: &'a App, save_label: &'a str) -> Element<'a, Message> {
    row![
        Space::new().width(Length::Fill),
        button(text("キャンセル").size(12))
            .style(style::dialog_button(&app.ui, false))
            .padding([6, 16])
            .on_press(Message::OverlayCancel),
        button(text(save_label.to_string()).size(12))
            .style(style::dialog_button(&app.ui, true))
            .padding([6, 16])
            .on_press(Message::Form(FormMsg::Save)),
    ]
    .spacing(8)
    .into()
}

fn confirm_buttons<'a>(app: &'a App, yes_label: &'a str) -> Element<'a, Message> {
    row![
        Space::new().width(Length::Fill),
        button(text("キャンセル").size(12))
            .style(style::dialog_button(&app.ui, false))
            .padding([6, 16])
            .on_press(Message::OverlayCancel),
        button(text(yes_label.to_string()).size(12))
            .style(style::danger_button(&app.ui))
            .padding([6, 16])
            .on_press(Message::Form(FormMsg::ConfirmYes)),
    ]
    .spacing(8)
    .into()
}

/// 検索オーバーレイ（上部バー + 結果リスト）
pub fn search_overlay(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let Some(search) = &app.search else {
        return text("").into();
    };
    let hits = app.search_hits();

    let mut content = column![text_input("検索（ラベル・パス）", &search.query)
        .id("search-input")
        .on_input(Message::SearchInput)
        .on_submit(Message::SearchSubmit)
        .size(13)
        .style(style::input(ui)),]
    .spacing(4);

    if !search.query.is_empty() {
        content = content.push(
            text(format!("{} 件", hits.len()))
                .size(10)
                .color(ui.text_muted),
        );
        for (i, hit) in hits.iter().take(8).enumerate() {
            let selected = i == search.selected;
            content = content.push(
                button(
                    column![
                        text(format!("{}  [{}]", hit.label, hit.tab_label)).size(12),
                        text(hit.path.clone())
                            .size(9)
                            .color(ui.text_muted)
                            .wrapping(text::Wrapping::None),
                    ]
                    .spacing(1),
                )
                .style(style::tab_button(ui, selected, false))
                .padding([4, 8])
                .width(Length::Fill)
                .on_press(Message::SearchClicked(i)),
            );
        }
    }

    let panel = container(content)
        .padding(10)
        .width(Length::Fixed(320.0))
        .style(style::panel(ui));

    container(panel)
        .center_x(Length::Fill)
        .padding(iced::Padding {
            top: crate::app::layout::TITLEBAR_HEIGHT + 4.0,
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}

/// ホバー中セルのツールチップ（stack レイヤーで自前描画。iced の tooltip ウィジェットは
/// tiny-skia で残像化するため使わない）。in_popup=true ならポップアップ座標系で配置する。
pub fn hover_tooltip(app: &App, in_popup: bool) -> Option<Element<'_, Message>> {
    // ドラッグ中・メニュー/オーバーレイ表示中は出さない
    if app.ctx_menu.is_some() || !matches!(app.overlay, Overlay::None) {
        return None;
    }
    if !matches!(app.drag, crate::app::DragState::Idle) {
        return None;
    }
    let (grid, idx) = app.hovered_cell?;
    // メイン用ツールチップはメインのグリッド、ポップアップ用はポップアップのグリッドのみ
    let is_popup_grid = app.popup_grid() == Some(grid);
    if is_popup_grid != in_popup {
        return None;
    }
    let tip = crate::ui::grid::tooltip_text(app, grid, idx)?;
    let cursor = if in_popup {
        app.popup_cursor
    } else {
        app.cursor
    };

    let ui = &app.ui;
    let panel = container(text(tip).size(11).color(ui.text_primary))
        .padding(6)
        .max_width(360.0)
        .style(style::panel(ui));

    // カーソルの少し下に配置。おおよその推定サイズでウィンドウ内にクランプ。
    let bounds = if in_popup {
        // ポップアップサイズは概算（グリッド構成から）
        crate::ui::grid::GridParams::resolve(app, grid)
            .map(|p| {
                let (w, h) = p.inner_size(app.data.settings.cell_size as f32);
                iced::Size::new(
                    w + crate::app::layout::GRID_PADDING,
                    h + crate::app::layout::GRID_PADDING + crate::app::layout::POPUP_HEADER_HEIGHT,
                )
            })
            .unwrap_or(iced::Size::new(400.0, 300.0))
    } else {
        crate::app::compute_main_size(&app.data, app.active_tab)
    };
    Some(positioned(
        panel.into(),
        cursor.x + 12.0,
        cursor.y + 18.0,
        200.0,
        60.0,
        bounds,
    ))
}

/// トースト通知（下部中央）
pub fn toast<'a>(app: &'a App, message: &str) -> Element<'a, Message> {
    let panel = container(
        text(message.to_string())
            .size(12)
            .color(app.ui.text_primary),
    )
    .padding([8, 14])
    .style(style::toast(&app.ui));
    container(panel)
        .center_x(Length::Fill)
        .align_y(Alignment::End)
        .padding(iced::Padding {
            bottom: crate::app::layout::STATUSBAR_HEIGHT + 8.0,
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
