//! ボタングリッド描画（メインタブ / グループポップアップ / リスト表示 共用）。

use crate::app::{layout, App, DragState, GridRef, Message};
use crate::model::data::{GridCell, LauncherItem};
use crate::model::theme::parse_color;
use crate::ui::style::{self, CellState};
use iced::widget::{column, container, mouse_area, row, stack, text};
use iced::{Alignment, Element, Length};

/// グリッドの表示パラメータ（GridRef から解決）
pub struct GridParams {
    pub cols: usize,
    pub rows: usize,
    pub list_mode: bool,
    pub list_cols: usize,
}

impl GridParams {
    pub fn resolve(app: &App, grid: GridRef) -> Option<GridParams> {
        let s = &app.data.settings;
        match grid {
            GridRef::Tab(t) => {
                let tab = app.data.tabs.get(t)?;
                let vm = tab.view_mode.as_deref().unwrap_or(&s.view_mode);
                Some(GridParams {
                    cols: tab.grid_columns as usize,
                    rows: tab.grid_rows as usize,
                    list_mode: vm == "list",
                    list_cols: tab.list_columns.unwrap_or(s.list_columns).clamp(1, 4) as usize,
                })
            }
            GridRef::Group { tab, cell } => {
                let parent = app.data.tabs.get(tab)?;
                let group = match parent.items.get(cell)?.as_ref()? {
                    GridCell::Group(g) => g,
                    _ => return None,
                };
                let vm = group
                    .view_mode
                    .as_deref()
                    .or(parent.view_mode.as_deref())
                    .unwrap_or(&s.view_mode);
                Some(GridParams {
                    cols: group.grid_columns as usize,
                    rows: group.grid_rows as usize,
                    list_mode: vm == "list",
                    list_cols: group
                        .list_columns
                        .or(parent.list_columns)
                        .unwrap_or(s.list_columns)
                        .clamp(1, 4) as usize,
                })
            }
        }
    }

    /// グリッド部の内容領域サイズ（論理px、パディング除く）
    pub fn inner_size(&self, cell_size: f32) -> (f32, f32) {
        use layout::*;
        let w = cell_size * self.cols as f32 + GRID_GAP * (self.cols as f32 - 1.0);
        let h = if self.list_mode {
            let total = self.cols * self.rows;
            let list_rows = total.div_ceil(self.list_cols);
            LIST_ROW_HEIGHT * list_rows as f32 + LIST_GAP * (list_rows as f32 - 1.0)
        } else {
            cell_size * self.rows as f32 + GRID_GAP * (self.rows as f32 - 1.0)
        };
        (w, h)
    }
}

/// グリッド全体を描画
pub fn grid_view<'a>(app: &'a App, grid: GridRef) -> Element<'a, Message> {
    let Some(params) = GridParams::resolve(app, grid) else {
        return text("").into();
    };
    let Some(cells) = app.cells(grid) else {
        return text("").into();
    };
    let cell_size = app.data.settings.cell_size as f32;
    let (inner_w, _) = params.inner_size(cell_size);

    let mut rows_col = column![].spacing(if params.list_mode {
        layout::LIST_GAP
    } else {
        layout::GRID_GAP
    });

    if params.list_mode {
        let total = params.cols * params.rows;
        let list_cols = params.list_cols;
        let cell_w = (inner_w - layout::GRID_GAP * (list_cols as f32 - 1.0)) / list_cols as f32;
        let list_rows = total.div_ceil(list_cols);
        for r in 0..list_rows {
            let mut row_el = row![].spacing(layout::GRID_GAP);
            for c in 0..list_cols {
                let idx = r * list_cols + c;
                if idx >= total {
                    break;
                }
                row_el = row_el.push(cell_view(
                    app,
                    grid,
                    idx,
                    cells.get(idx).and_then(|o| o.as_ref()),
                    cell_w,
                    layout::LIST_ROW_HEIGHT,
                    true,
                ));
            }
            rows_col = rows_col.push(row_el);
        }
    } else {
        for r in 0..params.rows {
            let mut row_el = row![].spacing(layout::GRID_GAP);
            for c in 0..params.cols {
                let idx = r * params.cols + c;
                row_el = row_el.push(cell_view(
                    app,
                    grid,
                    idx,
                    cells.get(idx).and_then(|o| o.as_ref()),
                    cell_size,
                    cell_size,
                    false,
                ));
            }
            rows_col = rows_col.push(row_el);
        }
    }

    container(rows_col)
        .padding(layout::GRID_PADDING / 2.0)
        .into()
}

/// セルの視覚状態を解決
fn cell_state(app: &App, grid: GridRef, idx: usize, occupied: bool) -> CellState {
    // ドラッグ元
    if let DragState::Dragging {
        grid: sg,
        index: si,
        over,
        ..
    } = &app.drag
    {
        if *sg == grid && *si == idx {
            return CellState::DragSource;
        }
        if over
            .map(|(og, oi)| og == grid && oi == idx)
            .unwrap_or(false)
        {
            return CellState::DropTarget;
        }
    }
    // 外部ファイルドロップのハイライト
    if app
        .drop_highlight
        .map(|(hg, hi)| hg == grid && hi == idx)
        .unwrap_or(false)
    {
        return CellState::DropTarget;
    }
    // キーボードフォーカス（メインのアクティブタブのみ）
    if grid == GridRef::Tab(app.active_tab) && app.focused_cell == Some(idx) {
        return CellState::Focused;
    }
    if app
        .hovered_cell
        .map(|(hg, hi)| hg == grid && hi == idx)
        .unwrap_or(false)
    {
        return CellState::Hovered;
    }
    if occupied {
        CellState::Normal
    } else {
        CellState::Empty
    }
}

/// 1セルを描画
#[allow(clippy::too_many_arguments)]
fn cell_view<'a>(
    app: &'a App,
    grid: GridRef,
    idx: usize,
    cell: Option<&'a GridCell>,
    w: f32,
    h: f32,
    list: bool,
) -> Element<'a, Message> {
    let ui = &app.ui;
    let state = cell_state(app, grid, idx, cell.is_some());
    let show_labels = app.data.settings.show_labels;
    let label_size = app.data.settings.label_font_size as f32;

    let content: Element<'a, Message> = match cell {
        Some(GridCell::Launcher(item)) => {
            launcher_content(app, item, w, h, list, show_labels, label_size)
        }
        Some(GridCell::Group(group)) => {
            let icon_el: Element<'a, Message> = if let Some(icon) = app.icons.get(&group.id) {
                crate::app::icon_element(icon, if list { 20.0 } else { 32.0 })
            } else {
                let emoji = group.icon.as_deref().unwrap_or("📂");
                let mut t = text(emoji.to_string()).size(if list { 16.0 } else { 24.0 });
                if let Some(c) = group.icon_color.as_deref().and_then(parse_color) {
                    t = t.color(c);
                }
                t.into()
            };
            item_layout(
                icon_el,
                &group.label,
                ui.text_primary,
                w,
                h,
                list,
                show_labels,
                label_size,
            )
        }
        Some(GridCell::Widget(widget)) => {
            let label = widget.label.clone().unwrap_or_else(|| "無効".into());
            item_layout(
                text("🧩").size(if list { 16.0 } else { 24.0 }).into(),
                &format!("{}（無効）", label),
                ui.text_muted,
                w,
                h,
                list,
                show_labels,
                label_size,
            )
        }
        None => text("").into(),
    };

    let boxed = container(content)
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .style(style::cell(ui, state));

    // ツールチップは iced の tooltip ウィジェット（overlay 機構）を使わない。
    // tiny-skia では overlay の消去が partial redraw で残像化するため、
    // ホバー中セルの情報は main_window/group_popup 側の stack レイヤーで自前描画する
    // （hover_tooltip）。ここでは mouse_area のみ返す。
    mouse_area(boxed)
        .on_press(Message::CellPressed(grid, idx))
        .on_release(Message::CellReleased(grid, idx))
        .on_right_press(Message::CellRightPressed(grid, idx))
        .on_enter(Message::CellEntered(grid, idx))
        .on_exit(Message::CellExited(grid, idx))
        .into()
}

/// ホバー中セルのツールチップ文字列（登録アイテムのみ。無ければ None）
pub fn tooltip_text(app: &App, grid: GridRef, idx: usize) -> Option<String> {
    let cell = app.cells(grid)?.get(idx)?.as_ref()?;
    if let GridCell::Launcher(item) = cell {
        let mut tip = item.label.clone();
        tip.push('\n');
        tip.push_str(&item.path);
        if let Some(args) = &item.args {
            if !args.trim().is_empty() {
                tip.push_str(&format!("\n引数: {}", args));
            }
        }
        if let Some(n) = item.launch_count {
            tip.push_str(&format!("\n起動回数: {}", n));
        }
        Some(tip)
    } else {
        None
    }
}

fn launcher_content<'a>(
    app: &'a App,
    item: &'a LauncherItem,
    w: f32,
    h: f32,
    list: bool,
    show_labels: bool,
    label_size: f32,
) -> Element<'a, Message> {
    let ui = &app.ui;
    let icon_px = if list { 20.0 } else { 32.0 };
    let icon_el: Element<'a, Message> = if let Some(icon) = app.icons.get(&item.id) {
        crate::app::icon_element(icon, icon_px)
    } else {
        let emoji = match item.item_type.as_str() {
            "folder" => "📁",
            "url" => "🌐",
            "document" => "📄",
            _ => "⚙",
        };
        text(emoji.to_string()).size(icon_px * 0.75).into()
    };

    let base = item_layout(
        icon_el,
        &item.label,
        ui.text_primary,
        w,
        h,
        list,
        show_labels,
        label_size,
    );

    // パス無効の警告オーバーレイ
    if app.invalid_paths.contains(&item.id) {
        stack![
            base,
            container(text("⚠").size(12.0).color(ui.warning))
                .align_x(Alignment::End)
                .align_y(Alignment::Start)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(2)
        ]
        .into()
    } else {
        base
    }
}

/// アイコン+ラベルの配置（グリッド=縦積み、リスト=横並び）
#[allow(clippy::too_many_arguments)]
fn item_layout<'a>(
    icon: Element<'a, Message>,
    label: &str,
    label_color: iced::Color,
    w: f32,
    _h: f32,
    list: bool,
    show_labels: bool,
    label_size: f32,
) -> Element<'a, Message> {
    if list {
        let mut r = row![icon]
            .spacing(6)
            .align_y(Alignment::Center)
            .padding([0, 6]);
        r = r.push(
            text(label.to_string())
                .size(label_size + 1.0)
                .color(label_color)
                .wrapping(text::Wrapping::None),
        );
        container(r)
            .align_y(Alignment::Center)
            .width(Length::Fixed(w))
            .height(Length::Fill)
            .clip(true)
            .into()
    } else {
        let mut c = column![icon].spacing(2).align_x(Alignment::Center);
        if show_labels {
            c = c.push(
                text(label.to_string())
                    .size(label_size)
                    .color(label_color)
                    .align_x(Alignment::Center)
                    .wrapping(text::Wrapping::None),
            );
        }
        container(c)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .clip(true)
            .into()
    }
}
