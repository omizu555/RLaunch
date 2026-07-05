//! メインウィンドウのビュー。

use crate::app::{layout, App, DragState, GridRef, Message};
use crate::model::data::GridCell;
use crate::ui::{grid, overlays, style};
use iced::widget::{button, column, container, mouse_area, row, stack, text, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;

    let base = column![
        titlebar(app),
        tabbar(app),
        container(grid::grid_view(app, GridRef::Tab(app.active_tab)))
            .width(Length::Fill)
            .height(Length::Fill),
        statusbar(app),
    ];

    let root = container(base)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::window_root(ui));

    // 全域のマウス追跡（カーソル位置・ドラッグ・背景クリック）
    let tracked = mouse_area(root)
        .on_move(Message::CursorMoved)
        .on_release(Message::RootReleased)
        .on_press(Message::CloseCtxMenu);

    let mut layers: Vec<Element<'_, Message>> = vec![tracked.into()];

    // ドラッグゴースト
    if let DragState::Dragging { grid, index, .. } = &app.drag {
        if let Some(el) = drag_ghost(app, *grid, *index) {
            layers.push(el);
        }
    }
    // 検索バー
    if app.search.is_some() {
        layers.push(overlays::search_overlay(app));
    }
    // コンテキストメニュー（ポップアップ側のグリッドが対象のものはポップアップに描画）
    if let Some(menu) = &app.ctx_menu {
        let in_popup = matches!(menu.target, crate::app::CtxTarget::Cell(g, _)
            if app.popup_grid() == Some(g));
        if !in_popup {
            layers.push(overlays::context_menu(app));
        }
    }
    // モーダルオーバーレイ（ポップアップのグリッドが対象のものはポップアップに描画）
    let overlay_in_popup = app
        .overlay_grid()
        .map(|g| app.popup_grid() == Some(g))
        .unwrap_or(false);
    if !overlay_in_popup {
        if let Some(el) = overlays::modal_overlay(app) {
            layers.push(el);
        }
    }
    // トースト
    if let Some((msg, _)) = &app.toast {
        layers.push(overlays::toast(app, msg));
    }

    stack(layers).into()
}

fn titlebar(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let s = &app.data.settings;

    let title_area = mouse_area(
        container(
            text(s.app_title.clone())
                .size(12)
                .color(ui.text_secondary)
                .wrapping(text::Wrapping::None),
        )
        .padding([0, 8])
        .align_y(Alignment::Center)
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .on_press(Message::TitlebarDrag);

    let bar = row![
        button(text("☰").size(14))
            .style(style::icon_button(ui, false))
            .padding([4, 8])
            .on_press(Message::OpenSettings),
        title_area,
        button(text("📌").size(12))
            .style(style::icon_button(ui, app.pinned))
            .padding([4, 8])
            .on_press(Message::PinToggle),
        button(text("─").size(12))
            .style(style::icon_button(ui, false))
            .padding([4, 8])
            .on_press(Message::HidePressed),
        button(text("✕").size(12))
            .style(style::icon_button(ui, false))
            .padding([4, 8])
            .on_press(Message::HidePressed),
    ]
    .align_y(Alignment::Center)
    .padding([0, 4]);

    container(bar)
        .height(Length::Fixed(layout::TITLEBAR_HEIGHT))
        .width(Length::Fill)
        .style(style::bar(ui))
        .into()
}

fn tabbar(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let mut tabs_row = row![].spacing(2).align_y(Alignment::Center);

    let dragging = matches!(app.drag, DragState::Dragging { .. });
    let tab_dragging = app.tab_drag.as_ref().map(|d| d.dragging).unwrap_or(false);
    for (i, tab) in app.data.tabs.iter().enumerate() {
        let is_active = i == app.active_tab;
        let drop_hover = (dragging || tab_dragging) && app.hovered_tab == Some(i) && !is_active;
        // タブ D&D 並び替えのため button ではなく mouse_area + container で press/release を扱う
        let tab_el = container(
            text(tab.label.clone())
                .size(12)
                .color(if is_active {
                    ui.text_primary
                } else {
                    ui.text_secondary
                })
                .wrapping(text::Wrapping::None),
        )
        .padding([4, 10])
        .style(style::cell(
            ui,
            if is_active {
                style::CellState::Pressed
            } else if drop_hover || app.hovered_tab == Some(i) {
                style::CellState::Hovered
            } else {
                style::CellState::Empty
            },
        ));
        tabs_row = tabs_row.push(
            mouse_area(tab_el)
                .on_press(Message::TabPressed(i))
                .on_release(Message::TabReleased(i))
                .on_right_press(Message::TabRightClicked(i))
                .on_enter(Message::TabEntered(i))
                .on_exit(Message::TabExited(i)),
        );
    }
    tabs_row = tabs_row.push(
        button(text("＋").size(12))
            .style(style::icon_button(ui, false))
            .padding([4, 8])
            .on_press(Message::TabAdd),
    );

    let bar = mouse_area(
        container(
            iced::widget::scrollable(tabs_row)
                .direction(iced::widget::scrollable::Direction::Horizontal(
                    iced::widget::scrollable::Scrollbar::new()
                        .width(2)
                        .scroller_width(2),
                ))
                .width(Length::Fill),
        )
        .padding([2, 6])
        .height(Length::Fixed(layout::TABBAR_HEIGHT))
        .width(Length::Fill)
        .align_y(Alignment::Center),
    )
    .on_enter(Message::TabbarEntered)
    .on_exit(Message::TabbarExited);

    container(bar)
        .style(style::bar_plain(ui))
        .height(Length::Fixed(layout::TABBAR_HEIGHT))
        .width(Length::Fill)
        .into()
}

fn statusbar(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let tab = app.data.tabs.get(app.active_tab);
    let left = match tab {
        Some(t) => format!(
            "{} — {} アイテム / {} スロット",
            t.label,
            t.item_count(),
            t.items.len()
        ),
        None => String::new(),
    };
    let right = if app.pinned {
        "📌 ピン留め中".to_string()
    } else {
        format!("{} で表示切替", app.data.settings.hotkey)
    };

    container(
        row![
            text(left).size(10).color(ui.text_muted),
            Space::new().width(Length::Fill),
            text(right).size(10).color(ui.text_muted),
        ]
        .align_y(Alignment::Center)
        .padding([0, 10]),
    )
    .height(Length::Fixed(layout::STATUSBAR_HEIGHT))
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .style(style::statusbar(ui))
    .into()
}

/// ドラッグ中のゴースト（カーソル追従）
fn drag_ghost(app: &App, grid: GridRef, index: usize) -> Option<Element<'_, Message>> {
    // メインウィンドウ内のドラッグのみ（ポップアップは各ウィンドウで描画）
    if app.popup_grid() == Some(grid) {
        return None;
    }
    let cells = app.cells(grid)?;
    let cell = cells.get(index)?.as_ref()?;

    let (icon_el, label): (Element<'_, Message>, String) = match cell {
        GridCell::Launcher(item) => {
            let icon: Element<'_, Message> = if let Some(icon) = app.icons.get(&item.id) {
                crate::app::icon_element(icon, 24.0)
            } else {
                text("⚙").size(18).into()
            };
            (icon, item.label.clone())
        }
        GridCell::Group(g) => (
            text(g.icon.clone().unwrap_or_else(|| "📂".into()))
                .size(18)
                .into(),
            g.label.clone(),
        ),
        GridCell::Widget(_) => (text("🧩").size(18).into(), "ウィジェット".into()),
    };

    let ghost = container(
        row![icon_el, text(label).size(11).color(app.ui.text_primary)]
            .spacing(6)
            .align_y(Alignment::Center),
    )
    .padding(6)
    .style(style::panel_translucent(&app.ui));

    // カーソルの右下少しオフセット
    let x = (app.cursor.x + 12.0).max(0.0);
    let y = (app.cursor.y + 12.0).max(0.0);
    Some(
        container(ghost)
            .padding(iced::Padding {
                top: y,
                left: x,
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}
