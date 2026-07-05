//! Message ごとの update 実処理。

use crate::app::{
    compute_main_size, layout, App, CtxAction, CtxMenu, CtxTarget, DragState, FormMsg, GridRef,
    GroupForm, ItemForm, Message, Overlay, SearchState, SettingsMsg, TabForm,
};
use crate::app_registration::blocking;
use crate::app_windows::resize_group_cells;
use crate::external::{ExternalEvent, HotkeyAction};
use crate::model::data::{now_iso8601, GridCell, GroupItem, Tab};
use crate::model::store;
use crate::platform::launch::{self, WindowState};
use iced::{keyboard, window, Point, Task};
use std::time::{Duration, Instant};

const DRAG_THRESHOLD: f32 = 5.0;
const AUTO_HIDE_DELAY: Duration = Duration::from_millis(300);
const POPUP_GUARD: Duration = Duration::from_millis(600);
const CURSOR_OUT_DELAY: Duration = Duration::from_millis(500);
const TAB_HOVER_SWITCH: Duration = Duration::from_millis(500);
const DROP_FLUSH_DELAY: Duration = Duration::from_millis(150);
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// ツールチップを表示するまでのホバー静止時間
const TOOLTIP_DELAY: Duration = Duration::from_millis(500);

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Noop => Task::none(),
            Message::External(ev) => self.on_external(ev),
            Message::IcedEvent(id, ev) => self.on_iced_event(id, ev),
            Message::ScaleFactor(s) => {
                if s > 0.1 {
                    self.scale = s;
                }
                // 初回表示は scale 取得後に行う（高DPIでの初回位置ずれ防止）
                if !self.initial_shown {
                    self.initial_shown = true;
                    return self.show_main_task();
                }
                Task::none()
            }
            Message::Tick => self.on_tick(),

            // ---------------- タイトルバー ----------------
            Message::TitlebarDrag => {
                // ユーザーによるドラッグ移動中のみ自動非表示を抑制する
                // （表示時の programmatic な move_to では抑制しない）
                self.titlebar_dragging = true;
                self.suppress_hide_until = Some(Instant::now() + Duration::from_secs(2));
                window::drag(self.main_id).map(|_: ()| Message::Noop)
            }
            Message::PinToggle => {
                self.pinned = !self.pinned;
                Task::none()
            }
            Message::HidePressed => self.hide_main_task(),
            Message::OpenSettings => self.open_settings_task(),

            // ---------------- タブ ----------------
            Message::TabClicked(t) => self.switch_tab(t),
            Message::TabPressed(t) => {
                self.ctx_menu = None;
                if matches!(self.overlay, Overlay::None) {
                    self.tab_drag = Some(crate::app::TabDrag {
                        index: t,
                        start: self.cursor,
                        dragging: false,
                    });
                }
                Task::none()
            }
            Message::TabReleased(t) => {
                let drag = self.tab_drag.take();
                match drag {
                    Some(d) if d.dragging => {
                        // 並び替え確定
                        self.save();
                        Task::none()
                    }
                    _ => {
                        // クリック（ダブルクリックでタブ設定を開く）
                        let now = Instant::now();
                        if let Some((lt, at)) = self.last_tab_click {
                            if lt == t && now.duration_since(at) <= DOUBLE_CLICK {
                                self.last_tab_click = None;
                                return self.on_tab_ctx(CtxAction::TabSettings, t);
                            }
                        }
                        self.last_tab_click = Some((t, now));
                        self.switch_tab(t)
                    }
                }
            }
            Message::TabAdd => {
                let s = &self.data.settings;
                let tab = Tab::new("新規タブ", s.default_grid_columns, s.default_grid_rows);
                self.data.tabs.push(tab);
                let t = self.data.tabs.len() - 1;
                self.save();
                self.switch_tab(t)
            }
            Message::TabRightClicked(t) => {
                self.ctx_menu = Some(CtxMenu {
                    at: self.cursor,
                    target: CtxTarget::Tab(t),
                });
                Task::none()
            }
            Message::TabEntered(t) => {
                self.hovered_tab = Some(t);
                if let DragState::Dragging { tab_hover, .. } = &mut self.drag {
                    if t != self.active_tab {
                        *tab_hover = Some((t, Instant::now()));
                    }
                }
                // タブ D&D 並び替え（ドラッグ中に他タブへ入ったらライブで入れ替え）
                if let Some(d) = &mut self.tab_drag {
                    if d.dragging && d.index != t && t < self.data.tabs.len() {
                        let from = d.index;
                        let moving = self.data.tabs.remove(from);
                        self.data.tabs.insert(t, moving);
                        // アクティブタブの追従
                        if self.active_tab == from {
                            self.active_tab = t;
                        } else if from < self.active_tab && t >= self.active_tab {
                            self.active_tab -= 1;
                        } else if from > self.active_tab && t <= self.active_tab {
                            self.active_tab += 1;
                        }
                        d.index = t;
                    }
                }
                Task::none()
            }
            Message::TabExited(t) => {
                if self.hovered_tab == Some(t) {
                    self.hovered_tab = None;
                }
                if let DragState::Dragging { tab_hover, .. } = &mut self.drag {
                    if tab_hover.map(|(ht, _)| ht) == Some(t) {
                        *tab_hover = None;
                    }
                }
                Task::none()
            }
            Message::TabbarEntered => {
                self.tabbar_hovered = true;
                Task::none()
            }
            Message::TabbarExited => {
                self.tabbar_hovered = false;
                Task::none()
            }

            // ---------------- グリッド ----------------
            Message::CellPressed(grid, idx) => {
                self.ctx_menu = None;
                if !matches!(self.overlay, Overlay::None) {
                    return Task::none();
                }
                self.drag = DragState::Pressed {
                    grid,
                    index: idx,
                    start: self.cursor_for(grid),
                };
                Task::none()
            }
            Message::CellReleased(grid, idx) => self.on_cell_released(grid, idx),
            Message::CellRightPressed(grid, idx) => {
                if !matches!(self.overlay, Overlay::None) {
                    return Task::none();
                }
                self.drag = DragState::Idle;
                self.ctx_menu = Some(CtxMenu {
                    at: self.cursor_for(grid),
                    target: CtxTarget::Cell(grid, idx),
                });
                Task::none()
            }
            Message::CellEntered(grid, idx) => {
                self.hovered_cell = Some((grid, idx));
                if let DragState::Dragging { over, .. } = &mut self.drag {
                    *over = Some((grid, idx));
                }
                Task::none()
            }
            Message::CellExited(grid, idx) => {
                if self.hovered_cell == Some((grid, idx)) {
                    self.hovered_cell = None;
                }
                if let DragState::Dragging { over, .. } = &mut self.drag {
                    if *over == Some((grid, idx)) {
                        *over = None;
                    }
                }
                Task::none()
            }
            Message::RootReleased => {
                self.drag = DragState::Idle;
                self.titlebar_dragging = false;
                if let Some(d) = self.tab_drag.take() {
                    if d.dragging {
                        self.save();
                    }
                }
                Task::none()
            }
            Message::CursorMoved(p) => {
                self.cursor = p;
                // カーソルが動いたらツールチップを隠し、静止タイマーを再スタート
                self.last_cursor_move = Some(Instant::now());
                self.tooltip_shown = false;
                self.maybe_start_drag(p, false);
                if let Some(d) = &mut self.tab_drag {
                    if !d.dragging {
                        let dist = ((p.x - d.start.x).powi(2) + (p.y - d.start.y).powi(2)).sqrt();
                        if dist > DRAG_THRESHOLD {
                            d.dragging = true;
                        }
                    }
                }
                Task::none()
            }
            Message::PopupCursorMoved(p) => {
                self.popup_cursor = p;
                self.last_cursor_move = Some(Instant::now());
                self.tooltip_shown = false;
                self.maybe_start_drag(p, true);
                Task::none()
            }

            // ---------------- コンテキストメニュー / オーバーレイ ----------------
            Message::Ctx(target, action) => self.on_ctx_action(target, action),
            Message::CloseCtxMenu => {
                self.ctx_menu = None;
                Task::none()
            }
            Message::Form(msg) => self.on_form(msg),
            Message::OverlayCancel => {
                self.overlay = Overlay::None;
                Task::none()
            }

            // ---------------- 検索 ----------------
            Message::SearchInput(q) => {
                if let Some(s) = &mut self.search {
                    s.query = q;
                    s.selected = 0;
                }
                Task::none()
            }
            Message::SearchNav(delta) => {
                // 表示件数（先頭8件）に合わせてクランプ（見えない項目の選択防止）
                let len = self.search_hits().len().min(8);
                if let Some(s) = &mut self.search {
                    if len > 0 {
                        let cur = s.selected as i32 + delta;
                        s.selected = cur.clamp(0, len as i32 - 1) as usize;
                    }
                }
                Task::none()
            }
            Message::SearchSubmit => {
                let switch_tab = self.modifiers.control();
                self.search_launch(None, switch_tab)
            }
            Message::SearchLaunch { switch_tab } => self.search_launch(None, switch_tab),
            Message::SearchClicked(i) => self.search_launch(Some(i), false),
            Message::SearchClose => {
                self.search = None;
                Task::none()
            }

            // ---------------- 設定 ----------------
            Message::Settings(msg) => self.on_settings(msg),

            // ---------------- グループポップアップ ----------------
            Message::PopupHeaderDrag => {
                if let Some(p) = &self.popup {
                    self.titlebar_dragging = true;
                    self.suppress_hide_until = Some(Instant::now() + POPUP_GUARD);
                    window::drag(p.id).map(|_: ()| Message::Noop)
                } else {
                    Task::none()
                }
            }
            Message::PopupClose => self.close_popup_task(),

            // ---------------- フォルダブラウザ ----------------
            Message::FolderEntryClicked(i) => self.on_folder_entry(i),
            Message::FolderUp => {
                if let Some(f) = &mut self.folder {
                    if let Some(prev) = f.history.pop() {
                        f.current = prev;
                    } else if let Some(parent) = f.current.parent() {
                        f.current = parent.to_path_buf();
                    }
                    f.entries = crate::app_windows::read_dir_entries(&f.current);
                }
                Task::none()
            }
            Message::FolderOpenExplorer => {
                if let Some(f) = &self.folder {
                    let path = f.current.to_string_lossy().into_owned();
                    if let Err(e) = launch::shell_open(&path, None, None, WindowState::Normal) {
                        self.show_toast(e);
                    }
                }
                self.close_folder_task()
            }
            Message::FolderClose => self.close_folder_task(),

            // ---------------- 非同期結果 ----------------
            Message::FilesPicked(grid, idx, paths) => {
                self.file_dialog_open = false;
                if let Some(paths) = paths {
                    if !paths.is_empty() {
                        return self.start_register(grid, idx, paths);
                    }
                }
                Task::none()
            }
            Message::FolderPicked(grid, idx, path) => {
                self.file_dialog_open = false;
                if let Some(path) = path {
                    return self.start_register(grid, idx, vec![path]);
                }
                Task::none()
            }
            Message::ItemsBuilt(grid, idx, items) => {
                self.finish_register(grid, idx, items);
                Task::none()
            }
            Message::ImagePicked(b64) => {
                self.file_dialog_open = false;
                if let Some(b64) = b64 {
                    match &mut self.overlay {
                        Overlay::ItemEdit { form, .. } => {
                            form.icon_override = Some(b64);
                            self.show_toast("画像を設定しました（保存で反映されます）");
                        }
                        Overlay::GroupEdit { form, .. } => {
                            form.icon_override = Some(b64);
                            self.show_toast("画像を設定しました（保存で反映されます）");
                        }
                        _ => {}
                    }
                }
                Task::none()
            }
            Message::ExportPathPicked(Some(path)) => {
                self.file_dialog_open = false;
                match store::export_to(&self.data, &path) {
                    Ok(()) => self.show_toast("エクスポートしました"),
                    Err(e) => self.show_toast(format!("エクスポート失敗: {}", e)),
                }
                Task::none()
            }
            Message::ExportPathPicked(None) => {
                self.file_dialog_open = false;
                Task::none()
            }
            Message::ImportPathPicked(Some(path)) => {
                self.file_dialog_open = false;
                self.overlay = Overlay::ImportChoice { path };
                Task::none()
            }
            Message::ImportPathPicked(None) => {
                self.file_dialog_open = false;
                Task::none()
            }
        }
    }

    fn cursor_for(&self, grid: GridRef) -> Point {
        match grid {
            GridRef::Tab(_) => self.cursor,
            GridRef::Group { .. } => {
                // ポップアップが開いているグループはポップアップ座標、
                // （メイングリッド上のグループセルはドラッグ元にならないので不問）
                if self.popup_grid() == Some(grid) {
                    self.popup_cursor
                } else {
                    self.cursor
                }
            }
        }
    }

    fn maybe_start_drag(&mut self, p: Point, in_popup: bool) {
        if let DragState::Pressed { grid, index, start } = self.drag {
            let is_popup_grid = self.popup_grid() == Some(grid);
            if is_popup_grid != in_popup {
                return;
            }
            let dist = ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt();
            if dist > DRAG_THRESHOLD {
                let occupied = self
                    .cells(grid)
                    .and_then(|c| c.get(index))
                    .map(|c| c.is_some())
                    .unwrap_or(false);
                if occupied {
                    self.drag = DragState::Dragging {
                        grid,
                        index,
                        over: None,
                        tab_hover: None,
                    };
                } else {
                    self.drag = DragState::Idle;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // 外部イベント
    // ------------------------------------------------------------------

    fn on_external(&mut self, ev: ExternalEvent) -> Task<Message> {
        match ev {
            ExternalEvent::Hotkey(id) => {
                let action = self
                    .hotkeys
                    .as_ref()
                    .and_then(|r| r.bindings.get(&id).cloned());
                match action {
                    Some(HotkeyAction::ToggleMain) => self.toggle_main_task(),
                    Some(HotkeyAction::LaunchItem(item_id)) => self.launch_item_by_id(&item_id),
                    None => Task::none(),
                }
            }
            ExternalEvent::TrayToggle | ExternalEvent::TrayMenuToggle => self.toggle_main_task(),
            ExternalEvent::TrayMenuSettings => self.open_settings_task(),
            ExternalEvent::TrayMenuQuit => {
                self.save();
                iced::exit()
            }
            ExternalEvent::DesktopDoubleClick => {
                if self.main_visible {
                    if self.pinned {
                        Task::none()
                    } else {
                        self.hide_main_task()
                    }
                } else {
                    // クリックした場所に出す（旧版挙動: 表示位置設定に関わらずカーソル基準）
                    self.show_main_at_cursor_task()
                }
            }
            ExternalEvent::ShowRequest => self.show_main_task(),
        }
    }

    // ------------------------------------------------------------------
    // iced イベント（ウィンドウ・キーボード・ホイール）
    // ------------------------------------------------------------------

    fn on_iced_event(&mut self, id: window::Id, ev: iced::Event) -> Task<Message> {
        use iced::Event as E;
        match ev {
            E::Window(wev) => self.on_window_event(id, wev),
            E::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
                self.modifiers = m;
                Task::none()
            }
            E::Keyboard(keyboard::Event::KeyPressed { key, .. }) => self.on_key(id, key),
            E::Mouse(iced::mouse::Event::WheelScrolled { delta }) => self.on_wheel(id, delta),
            E::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                // ウィンドウ外リリースでもドラッグ状態を確実に解除する。
                // セル上のリリースは mouse_area の CellReleased が先に処理し
                // drag は既に Idle になっているため、ここは残留分の掃除のみ。
                self.drag = DragState::Idle;
                self.titlebar_dragging = false;
                if let Some(d) = self.tab_drag.take() {
                    if d.dragging {
                        self.save();
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn on_window_event(&mut self, id: window::Id, ev: window::Event) -> Task<Message> {
        use window::Event as W;
        match ev {
            W::Opened { position, .. } => {
                if id == self.main_id {
                    self.main_pos = position;
                    return window::scale_factor(id).map(Message::ScaleFactor);
                } else if self.popup.as_ref().map(|p| p.id) == Some(id) {
                    self.popup_pos = position;
                }
                Task::none()
            }
            W::Moved(p) => {
                if id == self.main_id {
                    self.main_pos = Some(p);
                    if self.data.settings.window_position == "remember" {
                        self.data.settings.window_x = Some(p.x as i32);
                        self.data.settings.window_y = Some(p.y as i32);
                    }
                    // ユーザーがタイトルバーでドラッグ中のみ、移動に伴う
                    // フォーカス揺れで消えないよう抑制を延長する。
                    // （表示時の move_to でも Moved は来るが、そこで抑制すると
                    // 表示直後にデスクトップをクリックしても消えなくなる）
                    if self.titlebar_dragging {
                        self.suppress_hide_until = Some(Instant::now() + Duration::from_secs(2));
                    }
                } else if self.popup.as_ref().map(|p| p.id) == Some(id) {
                    self.popup_pos = Some(p);
                    self.popup_unfocused_since = None;
                }
                Task::none()
            }
            W::Rescaled(s) => {
                if id == self.main_id && s > 0.1 {
                    self.scale = s;
                }
                Task::none()
            }
            W::Focused => {
                if id == self.main_id {
                    self.main_focused = true;
                    self.unfocused_since = None;
                    // メインがフォーカスを得たらポップアップを閉じる（旧版挙動）
                    return self.close_popup_task();
                } else if self.popup.as_ref().map(|p| p.id) == Some(id) {
                    self.popup_unfocused_since = None;
                }
                Task::none()
            }
            W::Unfocused => {
                if id == self.main_id {
                    self.main_focused = false;
                    if self.main_visible {
                        self.unfocused_since = Some(Instant::now());
                    }
                } else if self.popup.as_ref().map(|p| p.id) == Some(id) {
                    self.popup_unfocused_since = Some(Instant::now());
                }
                Task::none()
            }
            W::CloseRequested => {
                if id == self.main_id {
                    self.hide_main_task()
                } else if Some(id) == self.settings_id {
                    self.settings_id = None;
                    self.save();
                    window::close(id).map(|_: window::Id| Message::Noop)
                } else if self.popup.as_ref().map(|p| p.id) == Some(id) {
                    self.close_popup_task()
                } else if self.folder.as_ref().map(|f| f.id) == Some(id) {
                    self.close_folder_task()
                } else {
                    window::close(id).map(|_: window::Id| Message::Noop)
                }
            }
            W::Closed => {
                if Some(id) == self.settings_id {
                    self.settings_id = None;
                    // 設定を閉じた後、メインが非フォーカスなら自動非表示を再アーム
                    if !self.main_focused && self.main_visible {
                        self.unfocused_since = Some(Instant::now());
                    }
                }
                if self.folder.is_none() && self.popup.is_none() {
                    // ポップアップ消滅後の取り残しタイマーを掃除（永久Tick防止の保険）
                    self.popup_unfocused_since = None;
                }
                if self.popup.as_ref().map(|p| p.id) == Some(id) {
                    self.popup = None;
                    self.popup_pos = None;
                    // ポップアップが閉じた後メインが非フォーカスなら自動非表示タイマー開始
                    if !self.main_focused && self.main_visible {
                        self.unfocused_since = Some(Instant::now());
                    }
                }
                if self.folder.as_ref().map(|f| f.id) == Some(id) {
                    self.folder = None;
                    if !self.main_focused && self.main_visible {
                        self.unfocused_since = Some(Instant::now());
                    }
                }
                Task::none()
            }
            W::FileHovered(_) => {
                // ドロップを受けるのはメインとグループポップアップのみ。
                // モーダル表示中は受けない（編集対象がシフトして別アイテムを上書きする事故防止）
                let accepts = (id == self.main_id || self.popup.as_ref().map(|p| p.id) == Some(id))
                    && matches!(self.overlay, Overlay::None);
                if accepts {
                    self.file_hovering = true;
                    self.file_hover_window = Some(id);
                }
                Task::none()
            }
            W::FilesHoveredLeft => {
                self.file_hovering = false;
                self.file_hover_window = None;
                self.drop_highlight = None;
                Task::none()
            }
            W::FileDropped(path) => {
                let accepts = (id == self.main_id || self.popup.as_ref().map(|p| p.id) == Some(id))
                    && matches!(self.overlay, Overlay::None);
                if accepts {
                    self.pending_drops.push((id, path));
                    self.last_drop_at = Some(Instant::now());
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn on_key(&mut self, id: window::Id, key: keyboard::Key) -> Task<Message> {
        use keyboard::key::Named;
        use keyboard::Key;

        // 設定ウィンドウ: Escape で閉じる
        if Some(id) == self.settings_id {
            if key == Key::Named(Named::Escape) {
                self.settings_id = None;
                self.save();
                return window::close(id).map(|_: window::Id| Message::Noop);
            }
            return Task::none();
        }
        // ポップアップ: Escape で閉じる
        if self.popup.as_ref().map(|p| p.id) == Some(id) {
            if key == Key::Named(Named::Escape) {
                return self.close_popup_task();
            }
            return Task::none();
        }
        if self.folder.as_ref().map(|f| f.id) == Some(id) {
            match key {
                Key::Named(Named::Escape) => return self.close_folder_task(),
                Key::Named(Named::Backspace) => return self.update(Message::FolderUp),
                _ => return Task::none(),
            }
        }
        if id != self.main_id {
            return Task::none();
        }

        // メインウィンドウのキー処理（優先度: メニュー > オーバーレイ > 検索 > グリッド）
        if key == Key::Named(Named::Escape) {
            if self.ctx_menu.is_some() {
                self.ctx_menu = None;
            } else if !matches!(self.overlay, Overlay::None) {
                self.overlay = Overlay::None;
            } else if self.search.is_some() {
                self.search = None;
            } else if !self.pinned {
                return self.hide_main_task();
            }
            return Task::none();
        }

        // Ctrl+F: 検索（モーダル表示中は開かない）
        if self.modifiers.control() && matches!(self.overlay, Overlay::None) {
            if let Key::Character(c) = &key {
                if c.as_str().eq_ignore_ascii_case("f") {
                    self.search = Some(SearchState::default());
                    return iced::widget::operation::focus(iced::advanced::widget::Id::new(
                        "search-input",
                    ));
                }
            }
        }

        // 検索中のナビゲーション
        if self.search.is_some() {
            return match key {
                Key::Named(Named::ArrowUp) => self.update(Message::SearchNav(-1)),
                Key::Named(Named::ArrowDown) => self.update(Message::SearchNav(1)),
                Key::Named(Named::Enter) => self.update(Message::SearchLaunch {
                    switch_tab: self.modifiers.control(),
                }),
                _ => Task::none(),
            };
        }

        if !matches!(self.overlay, Overlay::None) {
            // オーバーレイ表示中: Enter で保存（Ctrl+Enter 相当も含む）
            if key == Key::Named(Named::Enter) && self.modifiers.control() {
                return self.on_form(FormMsg::Save);
            }
            return Task::none();
        }

        // グリッドキーボードナビゲーション
        let Some(tab) = self.data.tabs.get(self.active_tab) else {
            return Task::none();
        };
        let cols = tab.grid_columns as i32;
        let total = tab.items.len() as i32;
        if total == 0 {
            return Task::none();
        }
        let cur = self.focused_cell.map(|c| c as i32).unwrap_or(-1);
        let next = match key {
            Key::Named(Named::ArrowRight) => Some((cur + 1).rem_euclid(total)),
            Key::Named(Named::ArrowLeft) => Some((cur - 1).rem_euclid(total)),
            Key::Named(Named::ArrowDown) => Some((cur + cols).rem_euclid(total)),
            Key::Named(Named::ArrowUp) => Some((cur - cols).rem_euclid(total)),
            Key::Named(Named::Enter) => {
                if let Some(idx) = self.focused_cell {
                    return self.activate_cell(GridRef::Tab(self.active_tab), idx);
                }
                None
            }
            Key::Named(Named::Delete) => {
                if let Some(idx) = self.focused_cell {
                    let grid = GridRef::Tab(self.active_tab);
                    self.clear_cell(grid, idx);
                }
                None
            }
            _ => None,
        };
        if let Some(n) = next {
            self.focused_cell = Some(n as usize);
        }
        Task::none()
    }

    fn on_wheel(&mut self, id: window::Id, delta: iced::mouse::ScrollDelta) -> Task<Message> {
        if id != self.main_id || !matches!(self.overlay, Overlay::None) {
            return Task::none();
        }
        let dy = match delta {
            iced::mouse::ScrollDelta::Lines { y, .. } => y,
            iced::mouse::ScrollDelta::Pixels { y, .. } => y,
        };
        if dy == 0.0 {
            return Task::none();
        }
        let up = dy > 0.0;

        // Ctrl+ホイール: セルサイズ段階変更
        if self.modifiers.control() {
            let sizes = layout::CELL_SIZES;
            let cur = self.data.settings.cell_size;
            let idx = sizes
                .iter()
                .position(|&s| s >= cur)
                .unwrap_or(sizes.len() - 1);
            let next = if up {
                (idx + 1).min(sizes.len() - 1)
            } else {
                idx.saturating_sub(1)
            };
            if sizes[next] != cur {
                self.data.settings.cell_size = sizes[next];
                self.save();
                return self.resize_main_task();
            }
            return Task::none();
        }

        // タブバー上のホイール: タブ切替（上=前、下=次）
        if self.tabbar_hovered || self.hovered_tab.is_some() {
            let len = self.data.tabs.len();
            if len > 1 {
                let cur = self.active_tab as i32;
                let next = if up { cur - 1 } else { cur + 1 };
                let next = next.clamp(0, len as i32 - 1) as usize;
                if next != self.active_tab {
                    return self.switch_tab(next);
                }
            }
        }
        Task::none()
    }

    // ------------------------------------------------------------------
    // Tick（100ms、必要時のみ購読）
    // ------------------------------------------------------------------

    fn on_tick(&mut self) -> Task<Message> {
        let now = Instant::now();
        let mut tasks: Vec<Task<Message>> = Vec::new();

        // ツールチップ静止表示: カーソルが一定時間止まっていたら表示
        if !self.tooltip_shown && self.hovered_cell.is_some() {
            if let Some(t) = self.last_cursor_move {
                if now >= t + TOOLTIP_DELAY {
                    self.tooltip_shown = true;
                }
            }
        }

        // トースト期限
        if let Some((_, until)) = &self.toast {
            if now >= *until {
                self.toast = None;
            }
        }

        let suppressed = self.suppress_hide_until.map(|t| now < t).unwrap_or(false);

        // 自動非表示（フォーカス喪失）。
        // ブロック中はタイマーを消費せず保持し、ブロック解除後に改めて評価する
        // （消費すると設定ウィンドウ等を閉じた後に永久に消えなくなる）。
        if let Some(since) = self.unfocused_since {
            if now >= since + AUTO_HIDE_DELAY {
                if self.main_focused || !self.main_visible {
                    // フォーカス復帰済み/既に非表示ならタイマー破棄
                    self.unfocused_since = None;
                } else {
                    let blocked = !self.data.settings.auto_hide
                        || self.pinned
                        || suppressed
                        || self.file_hovering
                        || self.file_dialog_open
                        || self.settings_id.is_some()
                        || self.popup.is_some()
                        || self.folder.is_some()
                        || matches!(self.drag, DragState::Dragging { .. });
                    if !blocked {
                        self.unfocused_since = None;
                        tasks.push(self.hide_main_task());
                    }
                }
            }
        }

        // ポップアップの自動クローズ（フォーカス喪失）。
        // ポップアップ発のモーダル（編集/確認等）表示中とダイアログ表示中は閉じない。
        if let Some(since) = self.popup_unfocused_since {
            let guard_ok = self
                .popup
                .as_ref()
                .map(|p| now >= p.opened_at + POPUP_GUARD)
                .unwrap_or(false);
            let overlay_open = !matches!(self.overlay, Overlay::None);
            if now >= since + AUTO_HIDE_DELAY
                && guard_ok
                && !suppressed
                && !self.file_hovering
                && !self.file_dialog_open
                && !overlay_open
            {
                tasks.push(self.close_popup_task());
            }
        }

        // カーソルアウトで非表示（CLaunch 風、オプション）
        if self.main_visible
            && self.data.settings.hide_on_cursor_out
            && !self.pinned
            && !suppressed
            && !self.file_hovering
            && self.popup.is_none()
            && self.folder.is_none()
            && self.settings_id.is_none()
            && matches!(self.overlay, Overlay::None)
            && self.ctx_menu.is_none()
        {
            if let (Some((cx, cy)), Some(pos)) =
                (crate::platform::monitor::cursor_pos(), self.main_pos)
            {
                let scale = self.scale.max(0.5);
                let size = compute_main_size(&self.data, self.active_tab);
                let margin = 12.0;
                let x0 = (pos.x - margin) * scale;
                let y0 = (pos.y - margin) * scale;
                let x1 = (pos.x + size.width + margin) * scale;
                let y1 = (pos.y + size.height + margin) * scale;
                let inside = (cx as f32) >= x0
                    && (cx as f32) <= x1
                    && (cy as f32) >= y0
                    && (cy as f32) <= y1;
                if inside {
                    // 一度ウィンドウに入って初めて「外に出たら消す」を発動可能にする。
                    // （表示位置=center 等でカーソルから離れた場所に出た直後、
                    // 触れる前に消えてしまうのを防ぐ）
                    self.cursor_out_armed = true;
                    self.cursor_out_since = None;
                } else if self.cursor_out_armed {
                    let since = *self.cursor_out_since.get_or_insert(now);
                    if now >= since + CURSOR_OUT_DELAY {
                        self.cursor_out_since = None;
                        tasks.push(self.hide_main_task());
                    }
                }
            }
        }

        // ドラッグ中のタブホバー切替
        if let DragState::Dragging {
            tab_hover: Some((t, since)),
            ..
        } = self.drag
        {
            if now >= since + TAB_HOVER_SWITCH && t != self.active_tab {
                if let DragState::Dragging {
                    tab_hover, over, ..
                } = &mut self.drag
                {
                    *tab_hover = None;
                    *over = None;
                }
                tasks.push(self.switch_tab(t));
            }
        }

        // 外部ファイルホバー中のハイライト追従
        if self.file_hovering {
            if let Some(win) = self.file_hover_window {
                self.drop_highlight = self.hit_test_drop(win);
            }
        }

        // ドロップのフラッシュ（複数ファイルをまとめて登録）
        if !self.pending_drops.is_empty() && matches!(self.overlay, Overlay::None) {
            let flush = self
                .last_drop_at
                .map(|t| now >= t + DROP_FLUSH_DELAY)
                .unwrap_or(true);
            if flush {
                let drops = std::mem::take(&mut self.pending_drops);
                let win = drops[0].0;
                let paths: Vec<_> = drops.into_iter().map(|(_, p)| p).collect();
                let target = self.drop_highlight.or_else(|| self.hit_test_drop(win));
                self.file_hovering = false;
                self.file_hover_window = None;
                self.drop_highlight = None;
                if let Some((grid, idx)) = target {
                    tasks.push(self.start_register(grid, idx, paths));
                } else {
                    // セル外へのドロップ: 先頭の空きセルへ
                    let grid = if self.popup.as_ref().map(|p| p.id) == Some(win) {
                        self.popup_grid().unwrap_or(GridRef::Tab(self.active_tab))
                    } else {
                        GridRef::Tab(self.active_tab)
                    };
                    if let Some(idx) = self.find_empty_from(grid, 0) {
                        tasks.push(self.start_register(grid, idx, paths));
                    } else {
                        self.show_toast("空きスロットがありません");
                    }
                }
            }
        }

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    // ------------------------------------------------------------------
    // セル操作
    // ------------------------------------------------------------------

    fn on_cell_released(&mut self, grid: GridRef, idx: usize) -> Task<Message> {
        let drag = std::mem::replace(&mut self.drag, DragState::Idle);
        match drag {
            DragState::Dragging {
                grid: src_grid,
                index: src_idx,
                ..
            } => {
                self.perform_drop(src_grid, src_idx, grid, idx);
                Task::none()
            }
            DragState::Pressed {
                grid: pg,
                index: pi,
                ..
            } if pg == grid && pi == idx => self.on_cell_clicked(grid, idx),
            _ => Task::none(),
        }
    }

    fn on_cell_clicked(&mut self, grid: GridRef, idx: usize) -> Task<Message> {
        let occupied = self
            .cells(grid)
            .and_then(|c| c.get(idx))
            .map(|c| c.is_some())
            .unwrap_or(false);
        if occupied {
            self.last_empty_click = None;
            return self.activate_cell(grid, idx);
        }
        // 空セル: ダブルクリックでファイル選択
        let now = Instant::now();
        if let Some((lg, li, at)) = self.last_empty_click {
            if lg == grid && li == idx && now.duration_since(at) <= DOUBLE_CLICK {
                self.last_empty_click = None;
                return pick_files_task(grid, idx);
            }
        }
        self.last_empty_click = Some((grid, idx, now));
        Task::none()
    }

    /// セルの内容を実行（起動 / グループを開く）
    pub fn activate_cell(&mut self, grid: GridRef, idx: usize) -> Task<Message> {
        let Some(cells) = self.cells(grid) else {
            return Task::none();
        };
        match cells.get(idx).and_then(|c| c.as_ref()) {
            Some(GridCell::Launcher(_)) => self.launch_cell(grid, idx),
            Some(GridCell::Group(_)) => {
                if let GridRef::Tab(tab) = grid {
                    self.open_group_popup_task(tab, idx)
                } else {
                    Task::none()
                }
            }
            Some(GridCell::Widget(_)) => {
                self.show_toast("ウィジェット機能は廃止されました（右クリックで解除できます）");
                Task::none()
            }
            None => Task::none(),
        }
    }

    /// 起動処理（統計更新・起動後の非表示込み）
    fn launch_cell(&mut self, grid: GridRef, idx: usize) -> Task<Message> {
        let Some(cells) = self.cells(grid) else {
            return Task::none();
        };
        let Some(GridCell::Launcher(item)) = cells.get(idx).and_then(|c| c.as_ref()) else {
            return Task::none();
        };
        let item = item.clone();

        // フォルダの browse モード → 内蔵ブラウザ
        if item.is_folder() && item.folder_action.as_deref() == Some("browse") {
            return self.open_folder_popup_task(std::path::PathBuf::from(&item.path));
        }

        let result = if item.run_as == Some(true) {
            launch::shell_runas(&item.path, item.args.as_deref())
        } else {
            launch::shell_open(
                &item.path,
                item.args.as_deref(),
                item.working_dir.as_deref(),
                WindowState::from_setting(item.window_state.as_deref()),
            )
        };

        match result {
            Ok(()) => {
                // 起動統計
                if let Some(cells) = self.cells_mut(grid) {
                    if let Some(Some(GridCell::Launcher(it))) = cells.get_mut(idx) {
                        it.launch_count = Some(it.launch_count.unwrap_or(0) + 1);
                        it.last_launched_at = Some(now_iso8601());
                    }
                }
                self.save();
                let mut tasks = vec![];
                if let GridRef::Group { .. } = grid {
                    tasks.push(self.close_popup_task());
                }
                if self.data.settings.hide_on_launch && !self.pinned {
                    tasks.push(self.hide_main_task());
                }
                Task::batch(tasks)
            }
            Err(e) => {
                self.show_toast(format!("起動失敗: {}", e));
                Task::none()
            }
        }
    }

    /// ID からアイテムを探して直接起動（アイテム個別ホットキー用）
    fn launch_item_by_id(&mut self, item_id: &str) -> Task<Message> {
        for (ti, tab) in self.data.tabs.iter().enumerate() {
            for (ci, cell) in tab.items.iter().enumerate() {
                match cell {
                    Some(GridCell::Launcher(it)) if it.id == item_id => {
                        return self.launch_cell(GridRef::Tab(ti), ci);
                    }
                    Some(GridCell::Group(g)) => {
                        for (gi, sub) in g.items.iter().enumerate() {
                            if let Some(GridCell::Launcher(it)) = sub {
                                if it.id == item_id {
                                    return self
                                        .launch_cell(GridRef::Group { tab: ti, cell: ci }, gi);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Task::none()
    }

    /// ドロップ実行: グループへの吸い込み / スワップ / タブまたぎ移動
    fn perform_drop(&mut self, sg: GridRef, si: usize, tg: GridRef, ti: usize) {
        if sg == tg && si == ti {
            return;
        }
        // タブまたぎ移動（ドラッグ中のタブホバー切替後のドロップ）
        if sg != tg {
            if let (GridRef::Tab(_), GridRef::Tab(_)) = (sg, tg) {
                let Some(moving) = self
                    .cells_mut(sg)
                    .and_then(|cells| cells.get_mut(si).and_then(|c| c.take()))
                else {
                    return;
                };
                // ターゲットが占有ならスワップ（相手を元セルへ）、空なら移動
                let displaced = self
                    .cells_mut(tg)
                    .and_then(|cells| cells.get_mut(ti))
                    .map(|slot| slot.replace(moving));
                if let Some(Some(displaced)) = displaced {
                    if let Some(cells) = self.cells_mut(sg) {
                        if let Some(slot) = cells.get_mut(si) {
                            *slot = Some(displaced);
                        }
                    }
                }
                self.save();
                self.recheck_invalid_paths();
                return;
            }
            // グループ⇔メイン等はウィンドウが異なるため不成立
            return;
        }
        let Some(cells) = self.cells(sg) else { return };
        let src_is_group = matches!(
            cells.get(si).and_then(|c| c.as_ref()),
            Some(GridCell::Group(_))
        );
        let tgt_is_group = matches!(
            cells.get(ti).and_then(|c| c.as_ref()),
            Some(GridCell::Group(_))
        );

        if tgt_is_group && !src_is_group {
            // グループへ吸い込み（先頭の空きスロットへ）
            let Some(cells) = self.cells_mut(sg) else {
                return;
            };
            let Some(moving) = cells.get_mut(si).and_then(|c| c.take()) else {
                return;
            };
            let mut placed = false;
            if let Some(Some(GridCell::Group(g))) = cells.get_mut(ti) {
                if let Some(slot) = g.items.iter_mut().find(|s| s.is_none()) {
                    *slot = Some(moving);
                    placed = true;
                    g.updated_at = now_iso8601();
                } else {
                    // 空きなし → 元に戻す
                    let moved_back = moving;
                    cells[si] = Some(moved_back);
                }
            } else {
                cells[si] = Some(moving);
            }
            if placed {
                self.show_toast("グループへ移動しました");
            } else {
                self.show_toast("グループに空きがありません");
            }
        } else {
            // スワップ
            if let Some(cells) = self.cells_mut(sg) {
                if si < cells.len() && ti < cells.len() {
                    cells.swap(si, ti);
                }
            }
        }
        self.save();
    }

    /// セルをクリア（アイコンキャッシュも削除）
    pub fn clear_cell(&mut self, grid: GridRef, idx: usize) {
        let removed = self
            .cells_mut(grid)
            .and_then(|cells| cells.get_mut(idx).and_then(|c| c.take()));
        if let Some(cell) = removed {
            let id = match &cell {
                GridCell::Launcher(i) => i.id.clone(),
                GridCell::Group(g) => g.id.clone(),
                GridCell::Widget(w) => w.id.clone(),
            };
            self.icons.remove(&id);
            self.save();
            self.rebind_hotkeys();
            self.show_toast("登録を解除しました");
        }
    }

    pub fn switch_tab(&mut self, t: usize) -> Task<Message> {
        if t >= self.data.tabs.len() {
            return Task::none();
        }
        self.active_tab = t;
        self.focused_cell = None;
        self.hovered_cell = None;
        self.ctx_menu = None;
        self.recheck_invalid_paths();
        Task::batch([self.close_popup_task(), self.resize_main_task()])
    }

    pub fn rebind_hotkeys(&mut self) {
        let wanted = crate::app::wanted_hotkeys(&self.data);
        if let Some(reg) = self.hotkeys.as_mut() {
            let failures = reg.rebind(&wanted);
            for (spec, err) in failures {
                self.show_toast(format!("ホットキー {} を登録できません: {}", spec, err));
            }
        }
    }

    fn search_launch(&mut self, index: Option<usize>, switch_tab: bool) -> Task<Message> {
        let hits = self.search_hits();
        let Some(sel) = index.or_else(|| self.search.as_ref().map(|s| s.selected)) else {
            return Task::none();
        };
        let Some(hit) = hits.get(sel) else {
            return Task::none();
        };
        let (tab, cell) = (hit.tab, hit.cell);
        self.search = None;
        if switch_tab {
            let t = self.switch_tab(tab);
            return Task::batch([t]);
        }
        self.launch_cell(GridRef::Tab(tab), cell)
    }

    // ------------------------------------------------------------------
    // コンテキストメニューのアクション
    // ------------------------------------------------------------------

    fn on_ctx_action(&mut self, target: CtxTarget, action: CtxAction) -> Task<Message> {
        self.ctx_menu = None;
        match target {
            CtxTarget::Cell(grid, idx) => self.on_cell_ctx(action, grid, idx),
            CtxTarget::Tab(t) => self.on_tab_ctx(action, t),
        }
    }

    fn on_cell_ctx(&mut self, action: CtxAction, grid: GridRef, idx: usize) -> Task<Message> {
        match action {
            CtxAction::Launch => self.activate_cell(grid, idx),
            CtxAction::RunAsAdmin => {
                let item = match self
                    .cells(grid)
                    .and_then(|c| c.get(idx))
                    .and_then(|c| c.as_ref())
                {
                    Some(GridCell::Launcher(i)) => Some((i.path.clone(), i.args.clone())),
                    _ => None,
                };
                if let Some((path, args)) = item {
                    if let Err(e) = launch::shell_runas(&path, args.as_deref()) {
                        self.show_toast(format!("管理者起動失敗: {}", e));
                    }
                }
                Task::none()
            }
            CtxAction::OpenLocation => {
                if let Some(GridCell::Launcher(item)) = self
                    .cells(grid)
                    .and_then(|c| c.get(idx))
                    .and_then(|c| c.as_ref())
                {
                    let path = item.path.clone();
                    if let Err(e) = launch::open_file_location(&path) {
                        self.show_toast(e);
                    }
                }
                Task::none()
            }
            CtxAction::BrowseFolder => {
                if let Some(GridCell::Launcher(item)) = self
                    .cells(grid)
                    .and_then(|c| c.get(idx))
                    .and_then(|c| c.as_ref())
                {
                    let path = std::path::PathBuf::from(&item.path);
                    return self.open_folder_popup_task(path);
                }
                Task::none()
            }
            CtxAction::ToggleFolderAction => {
                if let Some(cells) = self.cells_mut(grid) {
                    if let Some(Some(GridCell::Launcher(item))) = cells.get_mut(idx) {
                        let cur = item.folder_action.as_deref().unwrap_or("open");
                        let next = if cur == "browse" { "open" } else { "browse" };
                        item.folder_action = Some(next.to_string());
                        let msg = if next == "browse" {
                            "クリックで内蔵ブラウザを開くようにしました"
                        } else {
                            "クリックでエクスプローラーを開くようにしました"
                        };
                        self.save();
                        self.show_toast(msg);
                    }
                }
                Task::none()
            }
            CtxAction::EditItem => {
                if let Some(GridCell::Launcher(item)) = self
                    .cells(grid)
                    .and_then(|c| c.get(idx))
                    .and_then(|c| c.as_ref())
                {
                    let stats = format!(
                        "起動回数: {} / 最終起動: {}",
                        item.launch_count.unwrap_or(0),
                        item.last_launched_at.as_deref().unwrap_or("—")
                    );
                    self.overlay = Overlay::ItemEdit {
                        grid,
                        index: idx,
                        form: ItemForm {
                            label: item.label.clone(),
                            path: item.path.clone(),
                            args: item.args.clone().unwrap_or_default(),
                            working_dir: item.working_dir.clone().unwrap_or_default(),
                            hotkey: item.hotkey.clone().unwrap_or_default(),
                            run_as: item.run_as.unwrap_or(false),
                            window_state: item
                                .window_state
                                .clone()
                                .unwrap_or_else(|| "normal".into()),
                            stats,
                            icon_override: None,
                        },
                    };
                }
                Task::none()
            }
            CtxAction::RemoveItem => {
                let label = match self
                    .cells(grid)
                    .and_then(|c| c.get(idx))
                    .and_then(|c| c.as_ref())
                {
                    Some(GridCell::Launcher(i)) => i.label.clone(),
                    Some(GridCell::Group(g)) => g.label.clone(),
                    Some(GridCell::Widget(_)) => "ウィジェット".into(),
                    None => return Task::none(),
                };
                self.overlay = Overlay::ConfirmClear {
                    grid,
                    index: idx,
                    label,
                };
                Task::none()
            }
            CtxAction::RegisterFile => {
                self.file_dialog_open = true;
                pick_files_task(grid, idx)
            }
            CtxAction::RegisterFolder => {
                self.file_dialog_open = true;
                pick_folder_task(grid, idx)
            }
            CtxAction::RegisterUrl => {
                self.overlay = Overlay::UrlPrompt {
                    grid,
                    index: idx,
                    url: String::new(),
                };
                Task::none()
            }
            CtxAction::CreateGroup => {
                self.overlay = Overlay::GroupEdit {
                    grid,
                    index: idx,
                    existing: false,
                    form: GroupForm {
                        label: "新規グループ".into(),
                        icon: "📂".into(),
                        icon_color: None,
                        cols: 4,
                        rows: 2,
                        view_mode: None,
                        list_columns: None,
                        icon_override: None,
                    },
                };
                Task::none()
            }
            CtxAction::OpenGroup => self.activate_cell(grid, idx),
            CtxAction::EditGroup => {
                if let Some(GridCell::Group(g)) = self
                    .cells(grid)
                    .and_then(|c| c.get(idx))
                    .and_then(|c| c.as_ref())
                {
                    self.overlay = Overlay::GroupEdit {
                        grid,
                        index: idx,
                        existing: true,
                        form: GroupForm {
                            label: g.label.clone(),
                            icon: g.icon.clone().unwrap_or_else(|| "📂".into()),
                            icon_color: g.icon_color.clone(),
                            cols: g.grid_columns,
                            rows: g.grid_rows,
                            view_mode: g.view_mode.clone(),
                            list_columns: g.list_columns,
                            icon_override: None,
                        },
                    };
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn on_tab_ctx(&mut self, action: CtxAction, t: usize) -> Task<Message> {
        match action {
            CtxAction::TabSettings => {
                if let Some(tab) = self.data.tabs.get(t) {
                    self.overlay = Overlay::TabSettings {
                        tab: t,
                        form: TabForm {
                            label: tab.label.clone(),
                            cols: tab.grid_columns,
                            rows: tab.grid_rows,
                            view_mode: tab.view_mode.clone(),
                            list_columns: tab
                                .list_columns
                                .unwrap_or(self.data.settings.list_columns),
                        },
                    };
                }
                Task::none()
            }
            CtxAction::TabDuplicate => {
                if let Some(tab) = self.data.tabs.get(t) {
                    let mut copy = tab.clone();
                    copy.id = uuid::Uuid::new_v4().to_string();
                    copy.label = format!("{} のコピー", tab.label);
                    // アイテムに新 UUID を割り当て、アイコンキャッシュを引き継ぐ
                    let mut renew = |cell: &mut GridCell| {
                        let (old, new) = match cell {
                            GridCell::Launcher(i) => {
                                let old = i.id.clone();
                                i.id = uuid::Uuid::new_v4().to_string();
                                // ホットキーはグローバル資源なので複製しない
                                // （複製すると以後の rebind が毎回失敗トーストを出す）
                                i.hotkey = None;
                                (old, i.id.clone())
                            }
                            GridCell::Group(g) => {
                                let old = g.id.clone();
                                g.id = uuid::Uuid::new_v4().to_string();
                                (old, g.id.clone())
                            }
                            GridCell::Widget(w) => {
                                let old = w.id.clone();
                                w.id = uuid::Uuid::new_v4().to_string();
                                (old, w.id.clone())
                            }
                        };
                        if let Some(h) = self.icons.get(&old).cloned() {
                            self.icons.insert(new, h);
                        }
                    };
                    for cell in copy.items.iter_mut().flatten() {
                        renew(cell);
                        if let GridCell::Group(g) = cell {
                            for sub in g.items.iter_mut().flatten() {
                                renew(sub);
                            }
                        }
                    }
                    self.data.tabs.insert(t + 1, copy);
                    self.save();
                    self.show_toast("タブを複製しました");
                }
                Task::none()
            }
            CtxAction::TabDelete => {
                if self.data.tabs.len() <= 1 {
                    self.show_toast("最後のタブは削除できません");
                    return Task::none();
                }
                self.overlay = Overlay::ConfirmTabDelete { tab: t };
                Task::none()
            }
            _ => Task::none(),
        }
    }

    // ------------------------------------------------------------------
    // フォーム
    // ------------------------------------------------------------------

    fn on_form(&mut self, msg: FormMsg) -> Task<Message> {
        // フィールド更新
        match (&mut self.overlay, &msg) {
            (Overlay::ItemEdit { form, .. }, m) => match m {
                FormMsg::ItemLabel(v) => form.label = v.clone(),
                FormMsg::ItemPath(v) => form.path = v.clone(),
                FormMsg::ItemArgs(v) => form.args = v.clone(),
                FormMsg::ItemWorkDir(v) => form.working_dir = v.clone(),
                FormMsg::ItemHotkey(v) => form.hotkey = v.clone(),
                FormMsg::ItemRunAs(v) => form.run_as = *v,
                FormMsg::ItemWindowState(v) => form.window_state = v.clone(),
                _ => {}
            },
            (Overlay::GroupEdit { form, .. }, m) => match m {
                FormMsg::GroupLabel(v) => form.label = v.clone(),
                FormMsg::GroupIcon(v) => {
                    form.icon = v.clone();
                    form.icon_override = None; // 絵文字選択で画像指定を打ち消す
                }
                FormMsg::GroupColor(v) => form.icon_color = v.clone(),
                FormMsg::GroupCols(v) => form.cols = *v,
                FormMsg::GroupRows(v) => form.rows = *v,
                FormMsg::GroupViewMode(v) => form.view_mode = v.clone(),
                FormMsg::GroupListCols(v) => form.list_columns = *v,
                _ => {}
            },
            (Overlay::TabSettings { form, .. }, m) => match m {
                FormMsg::TabLabel(v) => form.label = v.clone(),
                FormMsg::TabCols(v) => form.cols = *v,
                FormMsg::TabRows(v) => form.rows = *v,
                FormMsg::TabViewMode(v) => form.view_mode = v.clone(),
                FormMsg::TabListCols(v) => form.list_columns = *v,
                _ => {}
            },
            (Overlay::UrlPrompt { url, .. }, FormMsg::UrlChanged(v)) => *url = v.clone(),
            _ => {}
        }

        match msg {
            FormMsg::Save => self.on_form_save(),
            FormMsg::PickImage => {
                // アイコン用画像を選んで PNG Base64 化（バックグラウンド）
                self.file_dialog_open = true;
                Task::perform(
                    async {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter(
                                "画像",
                                &["png", "jpg", "jpeg", "bmp", "gif", "webp", "ico"],
                            )
                            .pick_file()
                            .await?;
                        let path = file.path().to_path_buf();
                        blocking(move || {
                            let img = image::open(&path).ok()?;
                            let img = img.resize(64, 64, image::imageops::FilterType::Lanczos3);
                            let mut png: Vec<u8> = Vec::new();
                            img.write_to(
                                &mut std::io::Cursor::new(&mut png),
                                image::ImageFormat::Png,
                            )
                            .ok()?;
                            use base64::Engine as _;
                            Some(base64::engine::general_purpose::STANDARD.encode(&png))
                        })
                        .await
                    },
                    |r: Option<Option<String>>| Message::ImagePicked(r.flatten()),
                )
            }
            FormMsg::ConfirmYes => {
                let overlay = std::mem::replace(&mut self.overlay, Overlay::None);
                match overlay {
                    Overlay::ConfirmClear { grid, index, .. } => {
                        self.clear_cell(grid, index);
                        Task::none()
                    }
                    Overlay::ConfirmTabDelete { tab } => {
                        if self.data.tabs.len() > 1 && tab < self.data.tabs.len() {
                            self.data.tabs.remove(tab);
                            if self.active_tab == tab {
                                // アクティブタブ削除時は左隣へ（旧版/Chrome式）
                                self.active_tab = tab.saturating_sub(1);
                            } else if self.active_tab > tab {
                                self.active_tab -= 1;
                            }
                            if self.active_tab >= self.data.tabs.len() {
                                self.active_tab = self.data.tabs.len() - 1;
                            }
                            self.save();
                            self.rebind_hotkeys();
                            self.recheck_invalid_paths();
                            return self.resize_main_task();
                        }
                        Task::none()
                    }
                    other => {
                        self.overlay = other;
                        Task::none()
                    }
                }
            }
            FormMsg::ImportReplace | FormMsg::ImportMerge => {
                let overlay = std::mem::replace(&mut self.overlay, Overlay::None);
                if let Overlay::ImportChoice { path } = overlay {
                    let mode = if matches!(msg, FormMsg::ImportReplace) {
                        store::ImportMode::Replace
                    } else {
                        store::ImportMode::Merge
                    };
                    match store::import_from(&mut self.data, &path, mode) {
                        Ok(()) => {
                            store::normalize(&mut self.data);
                            self.active_tab = 0;
                            let theme_id = self.data.settings.theme.clone();
                            self.apply_theme(&theme_id);
                            self.rebuild_icon_cache();
                            self.recheck_invalid_paths();
                            self.rebind_hotkeys();
                            self.save();
                            self.show_toast("インポートしました");
                            return self.resize_main_task();
                        }
                        Err(e) => self.show_toast(format!("インポート失敗: {}", e)),
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn on_form_save(&mut self) -> Task<Message> {
        let overlay = std::mem::replace(&mut self.overlay, Overlay::None);
        match overlay {
            Overlay::ItemEdit { grid, index, form } => {
                // ホットキーの妥当性チェック（空は解除）
                let hotkey = form.hotkey.trim();
                if !hotkey.is_empty() {
                    if let Err(e) = crate::external::parse_hotkey(hotkey) {
                        self.show_toast(format!("ホットキーが不正です: {}", e));
                        self.overlay = Overlay::ItemEdit { grid, index, form };
                        return Task::none();
                    }
                }
                let mut new_icon: Option<(String, String)> = None;
                if let Some(cells) = self.cells_mut(grid) {
                    if let Some(Some(GridCell::Launcher(item))) = cells.get_mut(index) {
                        let path_changed = item.path != form.path;
                        item.label = form.label.clone();
                        item.path = form.path.clone();
                        item.args = none_if_empty(&form.args);
                        item.working_dir = none_if_empty(&form.working_dir);
                        item.hotkey = none_if_empty(&form.hotkey);
                        item.run_as = if form.run_as { Some(true) } else { None };
                        item.window_state = if form.window_state == "normal" {
                            None
                        } else {
                            Some(form.window_state.clone())
                        };
                        item.updated_at = now_iso8601();
                        if path_changed {
                            if let Ok(b64) =
                                crate::platform::icon::extract_icon_png_base64(&item.path)
                            {
                                item.icon_base64 = Some(b64.clone());
                                new_icon = Some((item.id.clone(), b64));
                            }
                        }
                    }
                }
                // 「画像を選ぶ」によるアイコン差し替え（パス変更の自動抽出より優先）
                if let Some(b64) = &form.icon_override {
                    if let Some(cells) = self.cells_mut(grid) {
                        if let Some(Some(GridCell::Launcher(item))) = cells.get_mut(index) {
                            item.icon_base64 = Some(b64.clone());
                            new_icon = Some((item.id.clone(), b64.clone()));
                        }
                    }
                }
                if let Some((id, b64)) = new_icon {
                    self.cache_icon(&id, &b64);
                }
                self.save();
                self.rebind_hotkeys();
                self.recheck_invalid_paths();
                self.show_toast("保存しました");
                Task::none()
            }
            Overlay::GroupEdit {
                grid,
                index,
                existing,
                form,
            } => {
                let mut icon_cache: Option<(String, String)> = None;
                if existing {
                    if let Some(cells) = self.cells_mut(grid) {
                        if let Some(Some(GridCell::Group(g))) = cells.get_mut(index) {
                            g.label = form.label.clone();
                            g.icon = Some(form.icon.clone());
                            g.icon_color = form.icon_color.clone();
                            g.view_mode = form.view_mode.clone();
                            g.list_columns = form.list_columns;
                            if let Some(b64) = &form.icon_override {
                                g.icon_base64 = Some(b64.clone());
                                icon_cache = Some((g.id.clone(), b64.clone()));
                            }
                            let old_cols = g.grid_columns;
                            let new_cols = form.cols.clamp(1, 8);
                            let new_rows = form.rows.clamp(1, 6);
                            if old_cols != new_cols || g.grid_rows != new_rows {
                                resize_group_cells(&mut g.items, old_cols, new_cols, new_rows);
                                g.grid_columns = new_cols;
                                g.grid_rows = new_rows;
                            }
                            g.updated_at = now_iso8601();
                        }
                    }
                } else {
                    let mut g = GroupItem::new(&form.label);
                    g.icon = Some(form.icon.clone());
                    g.icon_color = form.icon_color.clone();
                    g.view_mode = form.view_mode.clone();
                    g.list_columns = form.list_columns;
                    if let Some(b64) = &form.icon_override {
                        g.icon_base64 = Some(b64.clone());
                        icon_cache = Some((g.id.clone(), b64.clone()));
                    }
                    g.grid_columns = form.cols.clamp(1, 8);
                    g.grid_rows = form.rows.clamp(1, 6);
                    g.items = vec![None; (g.grid_columns * g.grid_rows) as usize];
                    if let Some(cells) = self.cells_mut(grid) {
                        if let Some(slot) = cells.get_mut(index) {
                            if slot.is_none() {
                                *slot = Some(GridCell::Group(g));
                            }
                        }
                    }
                }
                if let Some((id, b64)) = icon_cache {
                    self.cache_icon(&id, &b64);
                }
                self.save();
                self.show_toast("グループを保存しました");
                Task::none()
            }
            Overlay::TabSettings { tab, form } => {
                if let Some(t) = self.data.tabs.get_mut(tab) {
                    t.label = form.label.clone();
                    let new_cols = form.cols.clamp(1, 20);
                    let new_rows = form.rows.clamp(1, 10);
                    if t.grid_columns != new_cols || t.grid_rows != new_rows {
                        t.resize_grid(new_cols, new_rows);
                    }
                    t.view_mode = form.view_mode.clone();
                    t.list_columns = Some(form.list_columns.clamp(1, 4));
                }
                self.save();
                self.resize_main_task()
            }
            Overlay::UrlPrompt { grid, index, url } => {
                self.register_url(grid, index, &url);
                Task::none()
            }
            other => {
                self.overlay = other;
                Task::none()
            }
        }
    }

    // ------------------------------------------------------------------
    // 設定ウィンドウ
    // ------------------------------------------------------------------

    fn on_settings(&mut self, msg: SettingsMsg) -> Task<Message> {
        match msg {
            SettingsMsg::ThemeSelected(id) => {
                self.apply_theme(&id);
                self.save();
                Task::none()
            }
            SettingsMsg::CellSize(v) => {
                self.data.settings.cell_size = v.clamp(40, 120);
                self.save();
                self.resize_main_task()
            }
            // 列/行の全体設定は「新規タブの既定値」のみ。既存タブは変更しない
            // （旧版の全タブ一括リサイズは縮小時にアイテムが消える危険があるため廃止。
            // 各タブのサイズはタブ右クリック→タブ設定で個別に変更する）
            SettingsMsg::GridCols(v) => {
                self.data.settings.default_grid_columns = v.clamp(1, 20);
                self.save();
                Task::none()
            }
            SettingsMsg::GridRows(v) => {
                self.data.settings.default_grid_rows = v.clamp(1, 10);
                self.save();
                Task::none()
            }
            SettingsMsg::ShowLabels(b) => {
                self.data.settings.show_labels = b;
                self.save();
                Task::none()
            }
            SettingsMsg::LabelFontSize(v) => {
                self.data.settings.label_font_size = v.clamp(8, 16);
                self.save();
                Task::none()
            }
            SettingsMsg::HotkeyInput(v) => {
                self.settings_hotkey_draft = v;
                Task::none()
            }
            SettingsMsg::HotkeyApply => {
                let draft = self.settings_hotkey_draft.trim().to_string();
                match crate::external::parse_hotkey(&draft) {
                    Ok(_) => {
                        self.data.settings.hotkey = draft;
                        self.rebind_hotkeys();
                        self.save();
                        self.show_toast("ホットキーを変更しました");
                    }
                    Err(e) => self.show_toast(format!("ホットキーが不正です: {}", e)),
                }
                Task::none()
            }
            SettingsMsg::WindowPos(v) => {
                self.data.settings.window_position = v;
                self.save();
                Task::none()
            }
            SettingsMsg::AutoHide(b) => {
                self.data.settings.auto_hide = b;
                self.save();
                Task::none()
            }
            SettingsMsg::HideOnLaunch(b) => {
                self.data.settings.hide_on_launch = b;
                self.save();
                Task::none()
            }
            SettingsMsg::HideOnCursorOut(b) => {
                self.data.settings.hide_on_cursor_out = b;
                self.save();
                Task::none()
            }
            SettingsMsg::AutoStart(b) => {
                match apply_autostart(b) {
                    Ok(()) => {
                        self.data.settings.auto_start = b;
                        self.save();
                        self.show_toast(if b {
                            "自動起動を有効にしました"
                        } else {
                            "自動起動を無効にしました"
                        });
                    }
                    Err(e) => self.show_toast(format!("自動起動の設定失敗: {}", e)),
                }
                Task::none()
            }
            SettingsMsg::AppTitle(v) => {
                self.data.settings.app_title = v;
                if let Some(tray) = &self.tray {
                    let _ = tray.set_tooltip(Some(&self.data.settings.app_title));
                }
                self.save();
                Task::none()
            }
            SettingsMsg::ViewMode(v) => {
                self.data.settings.view_mode = v;
                self.save();
                self.resize_main_task()
            }
            SettingsMsg::ListColumns(v) => {
                self.data.settings.list_columns = v.clamp(1, 4);
                self.save();
                self.resize_main_task()
            }
            SettingsMsg::OpenThemesDir => {
                let dir = store::themes_dir();
                let _ = std::fs::create_dir_all(&dir);
                if let Err(e) =
                    launch::shell_open(&dir.to_string_lossy(), None, None, WindowState::Normal)
                {
                    self.show_toast(e);
                }
                Task::none()
            }
            SettingsMsg::OpenDataDir => {
                let dir = store::data_dir();
                if let Err(e) =
                    launch::shell_open(&dir.to_string_lossy(), None, None, WindowState::Normal)
                {
                    self.show_toast(e);
                }
                Task::none()
            }
            SettingsMsg::FontFamily(name) => {
                self.data.settings.font_family = name;
                self.save();
                self.show_toast("フォントは再起動後に反映されます");
                Task::none()
            }
            SettingsMsg::CopyData => {
                match serde_json::to_string_pretty(&self.data) {
                    Ok(json) => {
                        self.show_toast("データをクリップボードへコピーしました");
                        return iced::clipboard::write(json);
                    }
                    Err(e) => self.show_toast(format!("コピー失敗: {}", e)),
                }
                Task::none()
            }
            SettingsMsg::Export => {
                self.file_dialog_open = true;
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_file_name("launcher-data.json")
                            .add_filter("JSON", &["json"])
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ExportPathPicked,
                )
            }
            SettingsMsg::Import => {
                self.file_dialog_open = true;
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ImportPathPicked,
                )
            }
        }
    }

    fn on_folder_entry(&mut self, i: usize) -> Task<Message> {
        let Some(f) = &self.folder else {
            return Task::none();
        };
        let Some(entry) = f.entries.get(i).cloned() else {
            return Task::none();
        };
        if entry.is_dir {
            if let Some(f) = &mut self.folder {
                f.history.push(f.current.clone());
                f.current = entry.path.clone();
                f.entries = crate::app_windows::read_dir_entries(&entry.path);
            }
            Task::none()
        } else {
            let path = entry.path.to_string_lossy().into_owned();
            match launch::shell_open(&path, None, None, WindowState::Normal) {
                Ok(()) => {
                    let mut tasks = vec![self.close_folder_task()];
                    if self.data.settings.hide_on_launch && !self.pinned {
                        tasks.push(self.hide_main_task());
                    }
                    Task::batch(tasks)
                }
                Err(e) => {
                    self.show_toast(format!("起動失敗: {}", e));
                    Task::none()
                }
            }
        }
    }
}

fn none_if_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// ファイル選択ダイアログ（非同期）
fn pick_files_task(grid: GridRef, idx: usize) -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .add_filter("実行ファイル", &["exe", "bat", "cmd", "ps1", "msi"])
                .add_filter("ショートカット", &["lnk", "url"])
                .add_filter("すべてのファイル", &["*"])
                .pick_files()
                .await
                .map(|hs| hs.into_iter().map(|h| h.path().to_path_buf()).collect())
        },
        move |paths| Message::FilesPicked(grid, idx, paths),
    )
}

fn pick_folder_task(grid: GridRef, idx: usize) -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |path| Message::FolderPicked(grid, idx, path),
    )
}

/// 自動起動（HKCU Run キー）
fn apply_autostart(enable: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let auto = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("RLaunch")
        .set_app_path(&exe.to_string_lossy())
        .build()
        .map_err(|e| e.to_string())?;
    if enable {
        auto.enable().map_err(|e| e.to_string())
    } else {
        // 未登録状態での disable はエラーになり得るので無視してよい
        match auto.disable() {
            Ok(()) => Ok(()),
            Err(_) if !auto.is_enabled().unwrap_or(false) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
