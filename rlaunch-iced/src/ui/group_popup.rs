//! グループポップアップウィンドウのビュー（ミニランチャー）。

use crate::app::{layout, App, DragState, Message};
use crate::model::data::GridCell;
use crate::ui::{grid, overlays, style};
use iced::widget::{button, column, container, mouse_area, row, stack, text};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let Some(grid_ref) = app.popup_grid() else {
        return text("").into();
    };
    // グループ名を取得
    let label = match &app.popup {
        Some(p) => app
            .data
            .tabs
            .get(p.tab)
            .and_then(|t| t.items.get(p.cell))
            .and_then(|c| c.as_ref())
            .and_then(|c| match c {
                GridCell::Group(g) => Some(g.label.clone()),
                _ => None,
            })
            .unwrap_or_default(),
        None => String::new(),
    };

    let header = row![
        mouse_area(
            container(
                text(label)
                    .size(12)
                    .color(ui.text_secondary)
                    .wrapping(text::Wrapping::None),
            )
            .padding([0, 8])
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill),
        )
        .on_press(Message::PopupHeaderDrag),
        button(text("✕").size(11))
            .style(style::icon_button(ui, false))
            .padding([2, 8])
            .on_press(Message::PopupClose),
    ]
    .align_y(Alignment::Center)
    .padding([0, 4]);

    let base = column![
        container(header)
            .height(Length::Fixed(layout::POPUP_HEADER_HEIGHT))
            .width(Length::Fill)
            .style(style::bar(ui)),
        container(grid::grid_view(app, grid_ref))
            .width(Length::Fill)
            .height(Length::Fill),
    ];

    let root = container(base)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::window_root(ui));

    let tracked = mouse_area(root)
        .on_move(Message::PopupCursorMoved)
        .on_release(Message::RootReleased)
        .on_press(Message::CloseCtxMenu);

    let mut layers: Vec<Element<'_, Message>> = vec![tracked.into()];

    // ポップアップ内ドラッグゴースト
    if let DragState::Dragging { grid: g, index, .. } = &app.drag {
        if app.popup_grid() == Some(*g) {
            if let Some(cells) = app.cells(*g) {
                if let Some(Some(cell)) = cells.get(*index) {
                    let (icon_el, glabel): (Element<'_, Message>, String) = match cell {
                        GridCell::Launcher(item) => {
                            let icon: Element<'_, Message> =
                                if let Some(icon) = app.icons.get(&item.id) {
                                    crate::app::icon_element(icon, 20.0)
                                } else {
                                    text("⚙").size(14).into()
                                };
                            (icon, item.label.clone())
                        }
                        _ => (text("⚙").size(14).into(), String::new()),
                    };
                    let ghost = container(
                        row![icon_el, text(glabel).size(10).color(ui.text_primary)]
                            .spacing(4)
                            .align_y(Alignment::Center),
                    )
                    .padding(4)
                    .style(style::panel_translucent(ui));
                    layers.push(
                        container(ghost)
                            .padding(iced::Padding {
                                top: (app.popup_cursor.y + 10.0).max(0.0),
                                left: (app.popup_cursor.x + 10.0).max(0.0),
                                ..Default::default()
                            })
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into(),
                    );
                }
            }
        }
    }

    // ポップアップ内のコンテキストメニュー（対象がポップアップのグリッドの場合）
    if let Some(menu) = &app.ctx_menu {
        if let crate::app::CtxTarget::Cell(g, _) = menu.target {
            if app.popup_grid() == Some(g) {
                layers.push(overlays::context_menu(app));
            }
        }
    }
    // ポップアップのグリッドを対象にしたモーダル（編集/確認/URL登録）はこちらに描画
    let overlay_in_popup = app
        .overlay_grid()
        .map(|g| app.popup_grid() == Some(g))
        .unwrap_or(false);
    if overlay_in_popup {
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
