//! ウィンドウ操作: メインの表示/非表示・配置、設定/グループポップアップ/フォルダブラウザの開閉。

use crate::app::{
    compute_main_size, layout, App, DirEntry, FolderPopup, GridRef, GroupPopup, Message,
};
use crate::model::data::GridCell;
use crate::platform::monitor;
use crate::ui::grid::GridParams;
use iced::{window, Point, Size, Task};
use std::path::{Path, PathBuf};
use std::time::Instant;

impl App {
    /// メインウィンドウを表示（位置決め → 表示 → フォーカス）
    pub fn show_main_task(&mut self) -> Task<Message> {
        self.show_main_positioned(false)
    }

    /// カーソル基準で表示（デスクトップダブルクリック用。表示位置設定を無視 = 旧版挙動）
    pub fn show_main_at_cursor_task(&mut self) -> Task<Message> {
        self.show_main_positioned(true)
    }

    fn show_main_positioned(&mut self, force_cursor: bool) -> Task<Message> {
        let size = compute_main_size(&self.data, self.active_tab);
        let scale = self.scale.max(0.5);
        let mut tasks = vec![window::resize(self.main_id, size)];

        let mode = if force_cursor {
            "cursor"
        } else {
            self.data.settings.window_position.as_str()
        };
        let target: Option<Point> = match mode {
            "cursor" => monitor::cursor_monitor().map(|m| {
                let w = (size.width * scale) as i32;
                let h = (size.height * scale) as i32;
                let x = m.cursor_x - w / 2;
                let y = m.cursor_y - h / 2;
                let (x, y) = monitor::clamp_to_work_area(&m, x, y, w, h);
                Point::new(x as f32 / scale, y as f32 / scale)
            }),
            "center" => monitor::cursor_monitor().map(|m| {
                let w = (size.width * scale) as i32;
                let h = (size.height * scale) as i32;
                let x = m.work_x + (m.work_w - w) / 2;
                let y = m.work_y + (m.work_h - h) / 2;
                Point::new(x as f32 / scale, y as f32 / scale)
            }),
            // remember: window-state 相当（保存座標があれば復元、無ければそのまま）
            _ => match (self.data.settings.window_x, self.data.settings.window_y) {
                (Some(x), Some(y)) => Some(Point::new(x as f32, y as f32)),
                _ => None,
            },
        };
        if let Some(p) = target {
            tasks.push(window::move_to(self.main_id, p));
        }
        tasks.push(window::set_mode(self.main_id, window::Mode::Windowed));
        tasks.push(window::gain_focus(self.main_id));

        self.main_visible = true;
        self.unfocused_since = None;
        self.cursor_out_since = None;
        self.cursor_out_armed = false;
        Task::batch(tasks).map(|_: window::Id| Message::Noop)
    }

    /// メインウィンドウを非表示（ポップアップ類も閉じる）
    pub fn hide_main_task(&mut self) -> Task<Message> {
        self.main_visible = false;
        self.unfocused_since = None;
        self.cursor_out_since = None;
        self.ctx_menu = None;
        self.search = None;
        self.focused_cell = None;
        self.drag = crate::app::DragState::Idle;
        let mut tasks = vec![window::set_mode(self.main_id, window::Mode::Hidden)];
        self.popup_unfocused_since = None;
        if let Some(popup) = self.popup.take() {
            tasks.push(window::close(popup.id));
        }
        if let Some(folder) = self.folder.take() {
            tasks.push(window::close(folder.id));
        }
        if self.data.settings.window_position == "remember" {
            self.save();
        }
        Task::batch(tasks).map(|_: window::Id| Message::Noop)
    }

    pub fn toggle_main_task(&mut self) -> Task<Message> {
        if self.main_visible {
            if self.pinned {
                // ピン留め中はホットキーで消さない（本家挙動）
                window::gain_focus(self.main_id).map(|_: ()| Message::Noop)
            } else {
                self.hide_main_task()
            }
        } else {
            self.show_main_task()
        }
    }

    /// アクティブタブ構成に合わせてメインをリサイズ
    pub fn resize_main_task(&self) -> Task<Message> {
        let size = compute_main_size(&self.data, self.active_tab);
        window::resize(self.main_id, size).map(|_: ()| Message::Noop)
    }

    /// 設定ウィンドウを開く（既に開いていればフォーカス）
    pub fn open_settings_task(&mut self) -> Task<Message> {
        if let Some(id) = self.settings_id {
            return window::gain_focus(id).map(|_: ()| Message::Noop);
        }
        let (id, open) = window::open(window::Settings {
            size: Size::new(500.0, 680.0),
            resizable: false,
            decorations: true,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            platform_specific: window::settings::PlatformSpecific {
                drag_and_drop: false, // 設定画面へのファイルドロップは受けない
                ..Default::default()
            },
            ..Default::default()
        });
        self.settings_id = Some(id);
        self.settings_hotkey_draft = self.data.settings.hotkey.clone();
        open.map(|_| Message::Noop)
    }

    /// グループポップアップを開く（カーソル位置基準・作業領域クランプ）
    pub fn open_group_popup_task(&mut self, tab: usize, cell: usize) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = Vec::new();
        if let Some(prev) = self.popup.take() {
            tasks.push(window::close(prev.id).map(|_: window::Id| Message::Noop));
        }
        let grid = GridRef::Group { tab, cell };
        let Some(params) = GridParams::resolve(self, grid) else {
            return Task::batch(tasks);
        };
        let cell_size = self.data.settings.cell_size as f32;
        let (inner_w, inner_h) = params.inner_size(cell_size);
        let size = Size::new(
            (inner_w + layout::GRID_PADDING + layout::BORDER_EXTRA).max(180.0),
            inner_h + layout::GRID_PADDING + layout::BORDER_EXTRA + layout::POPUP_HEADER_HEIGHT,
        );

        let scale = self.scale.max(0.5);
        let position = monitor::cursor_monitor()
            .map(|m| {
                let w = (size.width * scale) as i32;
                let h = (size.height * scale) as i32;
                // カーソルの右下に出す（本家の感覚に近い）。はみ出すならクランプ。
                let (x, y) =
                    monitor::clamp_to_work_area(&m, m.cursor_x - 20, m.cursor_y - 10, w, h);
                window::Position::Specific(Point::new(x as f32 / scale, y as f32 / scale))
            })
            .unwrap_or(window::Position::Centered);

        let (id, open) = window::open(window::Settings {
            size,
            position,
            visible: true,
            resizable: false,
            decorations: false,
            transparent: self.ui.window_opacity < 1.0,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            platform_specific: window::settings::PlatformSpecific {
                skip_taskbar: true,
                drag_and_drop: true,
                undecorated_shadow: false,
                ..Default::default()
            },
            ..Default::default()
        });
        self.popup = Some(GroupPopup {
            id,
            tab,
            cell,
            opened_at: Instant::now(),
        });
        self.popup_unfocused_since = None;
        tasks.push(open.map(|_| Message::Noop));
        tasks.push(window::gain_focus(id).map(|_: ()| Message::Noop));
        Task::batch(tasks)
    }

    pub fn close_popup_task(&mut self) -> Task<Message> {
        self.popup_unfocused_since = None;
        if let Some(p) = self.popup.take() {
            window::close(p.id).map(|_: window::Id| Message::Noop)
        } else {
            Task::none()
        }
    }

    /// フォルダブラウザを開く
    pub fn open_folder_popup_task(&mut self, path: PathBuf) -> Task<Message> {
        let entries = read_dir_entries(&path);
        let size = Size::new(500.0, 460.0);
        let scale = self.scale.max(0.5);

        if let Some(f) = self.folder.as_mut() {
            // 再利用: 中身だけ差し替え
            f.history.push(f.current.clone());
            f.current = path;
            f.entries = entries;
            return window::gain_focus(f.id).map(|_: ()| Message::Noop);
        }

        let position = monitor::cursor_monitor()
            .map(|m| {
                let w = (size.width * scale) as i32;
                let h = (size.height * scale) as i32;
                let (x, y) =
                    monitor::clamp_to_work_area(&m, m.cursor_x - 20, m.cursor_y - 10, w, h);
                window::Position::Specific(Point::new(x as f32 / scale, y as f32 / scale))
            })
            .unwrap_or(window::Position::Centered);

        let (id, open) = window::open(window::Settings {
            size,
            position,
            visible: true,
            resizable: true,
            decorations: false,
            transparent: self.ui.window_opacity < 1.0,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            platform_specific: window::settings::PlatformSpecific {
                skip_taskbar: true,
                drag_and_drop: false, // フォルダブラウザへのドロップは受けない
                ..Default::default()
            },
            ..Default::default()
        });
        self.folder = Some(FolderPopup {
            id,
            current: path,
            entries,
            history: Vec::new(),
            opened_at: Instant::now(),
        });
        Task::batch([
            open.map(|_| Message::Noop),
            window::gain_focus(id).map(|_: ()| Message::Noop),
        ])
    }

    pub fn close_folder_task(&mut self) -> Task<Message> {
        if let Some(f) = self.folder.take() {
            window::close(f.id).map(|_: window::Id| Message::Noop)
        } else {
            Task::none()
        }
    }

    /// グループポップアップのグリッド参照（開いていれば）
    pub fn popup_grid(&self) -> Option<GridRef> {
        self.popup.as_ref().map(|p| GridRef::Group {
            tab: p.tab,
            cell: p.cell,
        })
    }

    /// 外部ファイルドロップのヒットテスト:
    /// 物理カーソル座標 → 対象ウィンドウのグリッドセル index
    pub fn hit_test_drop(&self, window_id: window::Id) -> Option<(GridRef, usize)> {
        let (cx, cy) = monitor::cursor_pos()?;
        let scale = self.scale.max(0.5);

        let (grid, origin, header_h) = if window_id == self.main_id {
            (
                GridRef::Tab(self.active_tab),
                self.main_pos?,
                layout::TITLEBAR_HEIGHT + layout::TABBAR_HEIGHT,
            )
        } else if let Some(p) = &self.popup {
            if p.id == window_id {
                // ポップアップの位置は開いた後に移動しない前提で cursor 起点を使えないため、
                // メインと同様に Moved を追跡できるまでは popup_pos を使う
                (
                    GridRef::Group {
                        tab: p.tab,
                        cell: p.cell,
                    },
                    self.popup_pos?,
                    layout::POPUP_HEADER_HEIGHT,
                )
            } else {
                return None;
            }
        } else {
            return None;
        };

        let local_x = cx as f32 / scale - origin.x;
        let local_y = cy as f32 / scale - origin.y - header_h;
        let params = GridParams::resolve(self, grid)?;
        let cell_size = self.data.settings.cell_size as f32;
        grid_hit_test(&params, cell_size, local_x, local_y).map(|idx| (grid, idx))
    }
}

/// グリッド内ローカル座標（パディング込み領域左上原点）からセル index を求める
pub fn grid_hit_test(params: &GridParams, cell_size: f32, x: f32, y: f32) -> Option<usize> {
    let pad = layout::GRID_PADDING / 2.0;
    let x = x - pad;
    let y = y - pad;
    if x < 0.0 || y < 0.0 {
        return None;
    }
    if params.list_mode {
        let total = params.cols * params.rows;
        let list_cols = params.list_cols;
        let inner_w =
            cell_size * params.cols as f32 + layout::GRID_GAP * (params.cols as f32 - 1.0);
        let cell_w = (inner_w - layout::GRID_GAP * (list_cols as f32 - 1.0)) / list_cols as f32;
        let col = (x / (cell_w + layout::GRID_GAP)) as usize;
        let row = (y / (layout::LIST_ROW_HEIGHT + layout::LIST_GAP)) as usize;
        if col >= list_cols {
            return None;
        }
        let idx = row * list_cols + col;
        if idx < total
            && x - col as f32 * (cell_w + layout::GRID_GAP) <= cell_w
            && y - row as f32 * (layout::LIST_ROW_HEIGHT + layout::LIST_GAP)
                <= layout::LIST_ROW_HEIGHT
        {
            Some(idx)
        } else {
            None
        }
    } else {
        let step = cell_size + layout::GRID_GAP;
        let col = (x / step) as usize;
        let row = (y / step) as usize;
        if col >= params.cols || row >= params.rows {
            return None;
        }
        if x - col as f32 * step <= cell_size && y - row as f32 * step <= cell_size {
            Some(row * params.cols + col)
        } else {
            None
        }
    }
}

/// フォルダ一覧（フォルダ先頭+名前順、アクセス不可はスキップ）
pub fn read_dir_entries(path: &Path) -> Vec<DirEntry> {
    let mut list: Vec<DirEntry> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(path) {
        for e in rd.flatten() {
            let meta = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            list.push(DirEntry {
                name: e.file_name().to_string_lossy().into_owned(),
                path: e.path(),
                is_dir: meta.is_dir(),
                size: meta.len(),
            });
        }
    }
    list.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    list
}

/// グループの中身リサイズ（旧版 App.tsx と同じ線形保持:
/// スロット総数が減らない限りアイテムは消えない。行列リマップはタブのみ）
pub fn resize_group_cells(
    items: &mut Vec<Option<GridCell>>,
    _old_cols: u32,
    new_cols: u32,
    new_rows: u32,
) {
    items.resize((new_cols * new_rows) as usize, None);
}
