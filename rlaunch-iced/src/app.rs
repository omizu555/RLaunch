//! アプリ本体（iced daemon）の State / Message 定義と骨格。
//! update の実処理は app_update.rs、ウィンドウ操作は app_windows.rs、view は ui/ 配下。

use crate::external::{self, ExternalEvent, HotkeyRegistry};
use crate::model::data::{GridCell, LauncherData, LauncherItem};
use crate::model::store;
use crate::model::theme::{self, ThemeInfo, UiTheme};
use iced::widget::{image, svg};
use iced::{keyboard, window, Element, Length, Point, Size, Subscription, Task};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

/// レイアウト定数（旧版 src/constants.ts と同値）
pub mod layout {
    pub const GRID_GAP: f32 = 6.0;
    pub const GRID_PADDING: f32 = 20.0;
    pub const BORDER_EXTRA: f32 = 2.0;
    pub const TITLEBAR_HEIGHT: f32 = 36.0;
    pub const TABBAR_HEIGHT: f32 = 36.0;
    pub const STATUSBAR_HEIGHT: f32 = 28.0;
    pub const LIST_ROW_HEIGHT: f32 = 32.0;
    pub const LIST_GAP: f32 = 2.0;
    /// グループポップアップのヘッダー高さ
    pub const POPUP_HEADER_HEIGHT: f32 = 32.0;
    /// セルサイズの段階（Ctrl+ホイール）
    pub const CELL_SIZES: [u32; 9] = [40, 48, 56, 64, 72, 80, 96, 112, 120];
}

/// グリッドの参照先（メインタブ or タブ内グループ）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridRef {
    Tab(usize),
    Group { tab: usize, cell: usize },
}

/// セル間ドラッグの状態
#[derive(Debug, Clone)]
pub enum DragState {
    Idle,
    /// 押下直後（5px 動くまではクリック扱い）
    Pressed {
        grid: GridRef,
        index: usize,
        start: Point,
    },
    Dragging {
        grid: GridRef,
        index: usize,
        /// 現在ホバー中のドロップ先
        over: Option<(GridRef, usize)>,
        /// タブホバー切替（タブ index とホバー開始時刻）
        tab_hover: Option<(usize, Instant)>,
    },
}

/// コンテキストメニュー
#[derive(Debug, Clone)]
pub struct CtxMenu {
    pub at: Point,
    pub target: CtxTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtxTarget {
    Cell(GridRef, usize),
    Tab(usize),
}

/// コンテキストメニューからのアクション
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtxAction {
    Launch,
    RunAsAdmin,
    OpenLocation,
    BrowseFolder,
    ToggleFolderAction,
    EditItem,
    RemoveItem,
    RegisterFile,
    RegisterFolder,
    RegisterUrl,
    CreateGroup,
    OpenGroup,
    EditGroup,
    TabSettings,
    TabDuplicate,
    TabDelete,
}

/// モーダルオーバーレイ
#[derive(Debug, Clone)]
pub enum Overlay {
    None,
    ItemEdit {
        grid: GridRef,
        index: usize,
        form: ItemForm,
    },
    GroupEdit {
        grid: GridRef,
        index: usize,
        /// None = 新規作成
        existing: bool,
        form: GroupForm,
    },
    TabSettings {
        tab: usize,
        form: TabForm,
    },
    UrlPrompt {
        grid: GridRef,
        index: usize,
        url: String,
    },
    ConfirmClear {
        grid: GridRef,
        index: usize,
        label: String,
    },
    ConfirmTabDelete {
        tab: usize,
    },
    ImportChoice {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ItemForm {
    pub label: String,
    pub path: String,
    pub args: String,
    pub working_dir: String,
    pub hotkey: String,
    pub run_as: bool,
    pub window_state: String,
    pub stats: String,
    /// 「画像を選ぶ」で設定した新アイコン（PNG Base64、保存時に反映）
    pub icon_override: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GroupForm {
    pub label: String,
    pub icon: String,
    pub icon_color: Option<String>,
    pub cols: u32,
    pub rows: u32,
    /// None = 親タブ/全体設定を使用
    pub view_mode: Option<String>,
    pub list_columns: Option<u32>,
    /// 「画像を選ぶ」で設定した新アイコン（PNG Base64、保存時に反映）
    pub icon_override: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TabForm {
    pub label: String,
    pub cols: u32,
    pub rows: u32,
    /// None = 全体設定を使用
    pub view_mode: Option<String>,
    pub list_columns: u32,
}

/// 検索状態
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub selected: usize,
}

/// 検索結果1件
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub tab: usize,
    pub cell: usize,
    pub label: String,
    pub path: String,
    pub tab_label: String,
}

/// グループポップアップウィンドウ
#[derive(Debug, Clone)]
pub struct GroupPopup {
    pub id: window::Id,
    pub tab: usize,
    pub cell: usize,
    /// 開いた直後のフォーカス喪失ガード
    pub opened_at: Instant,
}

/// フォルダブラウザポップアップ
#[derive(Debug, Clone)]
pub struct FolderPopup {
    pub id: window::Id,
    pub current: PathBuf,
    pub entries: Vec<DirEntry>,
    pub history: Vec<PathBuf>,
    pub opened_at: Instant,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

pub struct App {
    pub data: LauncherData,
    pub themes: Vec<ThemeInfo>,
    pub ui: UiTheme,

    // ウィンドウ
    pub main_id: window::Id,
    pub main_visible: bool,
    pub scale: f32,
    /// メインウィンドウの位置（論理スクリーン座標、Moved/Opened で追跡）
    pub main_pos: Option<Point>,
    pub settings_id: Option<window::Id>,
    pub settings_hotkey_draft: String,
    pub popup: Option<GroupPopup>,
    pub popup_pos: Option<Point>,
    pub folder: Option<FolderPopup>,

    // 表示状態
    pub active_tab: usize,
    pub pinned: bool,
    pub focused_cell: Option<usize>,
    pub hovered_cell: Option<(GridRef, usize)>,
    pub hovered_tab: Option<usize>,
    pub tabbar_hovered: bool,

    // ドラッグ＆ドロップ
    pub drag: DragState,
    pub file_hovering: bool,
    pub file_hover_window: Option<window::Id>,
    pub drop_highlight: Option<(GridRef, usize)>,
    pub pending_drops: Vec<(window::Id, PathBuf)>,
    pub last_drop_at: Option<Instant>,
    pub last_empty_click: Option<(GridRef, usize, Instant)>,

    // オーバーレイ
    pub ctx_menu: Option<CtxMenu>,
    pub overlay: Overlay,
    pub search: Option<SearchState>,
    pub toast: Option<(String, Instant)>,

    // 入力
    pub cursor: Point,
    pub popup_cursor: Point,
    pub modifiers: keyboard::Modifiers,

    // 自動非表示
    pub main_focused: bool,
    pub unfocused_since: Option<Instant>,
    pub popup_unfocused_since: Option<Instant>,
    pub suppress_hide_until: Option<Instant>,
    pub cursor_out_since: Option<Instant>,

    // キャッシュ
    pub icons: HashMap<String, IconData>,
    pub invalid_paths: HashSet<String>,

    // 常駐部品（drop すると消えるので保持し続ける）
    pub tray: Option<tray_icon::TrayIcon>,
    pub hotkeys: Option<HotkeyRegistry>,

    // その他
    /// 初回表示済みか（scale factor 取得後に初回表示するためのフラグ）
    pub initial_shown: bool,
    /// rfd ダイアログ表示中（自動非表示を抑止）
    pub file_dialog_open: bool,
    /// タブの D&D 並び替え状態
    pub tab_drag: Option<TabDrag>,
    /// タブのダブルクリック検出用
    pub last_tab_click: Option<(usize, Instant)>,
    /// launcher-data.json のロード/保存時点の更新時刻（外部変更検出用）
    pub data_mtime: Option<std::time::SystemTime>,
    /// 読み取り失敗時に true（データ全消失を防ぐため保存を禁止）
    pub save_disabled: bool,
    /// タイトルバー/ポップアップヘッダーのドラッグ移動中
    /// （この間だけウィンドウ移動による自動非表示抑制を行う。
    /// 表示時の programmatic な move_to では抑制しない）
    pub titlebar_dragging: bool,
    /// カーソルアウト非表示のアーミング（表示後、カーソルが一度ウィンドウに
    /// 入ってから「外に出たら消す」を有効化する）
    pub cursor_out_armed: bool,
}

/// タブの D&D 並び替え状態
#[derive(Debug, Clone)]
pub struct TabDrag {
    pub index: usize,
    pub start: Point,
    pub dragging: bool,
}

/// アイコンキャッシュの要素。ラスター(PNG等)とベクター(SVG)を区別する
/// （iced の image ウィジェットはラスター専用で、SVG を渡すと描画時に panic するため）。
#[derive(Debug, Clone)]
pub enum IconData {
    Raster(image::Handle),
    Vector(svg::Handle),
}

/// Base64（data URL 可）を検証してアイコン化する。
/// - SVG は svg::Handle に（旧版アイコンライブラリ由来）
/// - ラスターは実デコードして幅高さ>0 を確認し from_rgba に（不正画像での panic を防ぐ）
/// - どちらでもデコードできなければ None（呼び出し側は絵文字にフォールバック）
pub fn decode_icon(b64: &str) -> Option<IconData> {
    use base64::Engine as _;
    let (svg_hint, raw) = if let Some(rest) = b64.strip_prefix("data:") {
        let is_svg = rest.starts_with("image/svg");
        let data = rest.split_once(',').map(|(_, d)| d).unwrap_or(rest);
        (is_svg, data)
    } else {
        (false, b64)
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw.trim())
        .ok()?;
    if bytes.is_empty() {
        return None;
    }
    // SVG 判定: data URL のヒント、または先頭が <svg / <?xml
    let looks_svg = svg_hint || {
        let head = &bytes[..bytes.len().min(64)];
        let s = String::from_utf8_lossy(head);
        let s = s.trim_start();
        s.starts_with("<svg") || s.starts_with("<?xml")
    };
    if looks_svg {
        Some(IconData::Vector(svg::Handle::from_memory(bytes)))
    } else {
        // ラスターは image crate で実デコードして妥当性を確認してから from_rgba。
        // （from_bytes は遅延デコードで、不正な場合に tiny-skia の描画時 panic を招く）
        // ※ ここでの `image` は iced::widget::image なので、crate は `::image` で参照する
        let img = ::image::load_from_memory(&bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        if w == 0 || h == 0 {
            return None;
        }
        Some(IconData::Raster(image::Handle::from_rgba(
            w,
            h,
            rgba.into_raw(),
        )))
    }
}

/// アイコンを指定サイズの Element として描画する
pub fn icon_element<'a>(icon: &IconData, size: f32) -> Element<'a, Message> {
    match icon {
        IconData::Raster(h) => image(h.clone())
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into(),
        IconData::Vector(h) => svg(h.clone())
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into(),
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // システム
    External(ExternalEvent),
    IcedEvent(window::Id, iced::Event),
    ScaleFactor(f32),
    Tick,

    // タイトルバー
    TitlebarDrag,
    PinToggle,
    HidePressed,
    OpenSettings,

    // タブ
    TabClicked(usize),
    TabPressed(usize),
    TabReleased(usize),
    TabAdd,
    TabRightClicked(usize),
    TabEntered(usize),
    TabExited(usize),
    TabbarEntered,
    TabbarExited,

    // グリッド
    CellPressed(GridRef, usize),
    CellReleased(GridRef, usize),
    CellRightPressed(GridRef, usize),
    CellEntered(GridRef, usize),
    CellExited(GridRef, usize),
    RootReleased,
    CursorMoved(Point),
    PopupCursorMoved(Point),

    // コンテキストメニュー / オーバーレイ
    // ターゲットをメッセージに含める（ルートの CloseCtxMenu と同フレームで
    // 処理順が入れ替わっても動作するように、self.ctx_menu に依存しない）
    Ctx(CtxTarget, CtxAction),
    CloseCtxMenu,
    Form(FormMsg),
    OverlayCancel,

    // 検索
    SearchInput(String),
    SearchNav(i32),
    /// text_input の on_submit 用（switch_tab は update 側で modifiers から決定）
    SearchSubmit,
    SearchLaunch {
        switch_tab: bool,
    },
    SearchClicked(usize),
    SearchClose,

    // 設定ウィンドウ
    Settings(SettingsMsg),

    // グループポップアップ
    PopupHeaderDrag,
    PopupClose,

    // フォルダブラウザ
    FolderEntryClicked(usize),
    FolderUp,
    FolderOpenExplorer,
    FolderClose,

    // 非同期結果
    FilesPicked(GridRef, usize, Option<Vec<PathBuf>>),
    FolderPicked(GridRef, usize, Option<PathBuf>),
    /// バックグラウンドで構築済みのアイテム群（lnk解決・アイコン抽出済み）を配置する
    ItemsBuilt(GridRef, usize, Vec<LauncherItem>),
    /// アイコン用画像ファイルの選択結果（PNG Base64 化済み）
    ImagePicked(Option<String>),
    ExportPathPicked(Option<PathBuf>),
    ImportPathPicked(Option<PathBuf>),

    Noop,
}

/// オーバーレイフォームの入力
#[derive(Debug, Clone)]
pub enum FormMsg {
    ItemLabel(String),
    ItemPath(String),
    ItemArgs(String),
    ItemWorkDir(String),
    ItemHotkey(String),
    ItemRunAs(bool),
    ItemWindowState(String),
    GroupLabel(String),
    GroupIcon(String),
    GroupColor(Option<String>),
    GroupCols(u32),
    GroupRows(u32),
    GroupViewMode(Option<String>),
    GroupListCols(Option<u32>),
    /// アイコン用画像ファイルを選ぶ（アイテム/グループ編集共通）
    PickImage,
    TabLabel(String),
    TabCols(u32),
    TabRows(u32),
    TabViewMode(Option<String>),
    TabListCols(u32),
    UrlChanged(String),
    ConfirmYes,
    ImportReplace,
    ImportMerge,
    Save,
}

/// 設定ウィンドウの操作
#[derive(Debug, Clone)]
pub enum SettingsMsg {
    ThemeSelected(String),
    CellSize(u32),
    GridCols(u32),
    GridRows(u32),
    ShowLabels(bool),
    LabelFontSize(u32),
    HotkeyInput(String),
    HotkeyApply,
    WindowPos(String),
    AutoHide(bool),
    HideOnLaunch(bool),
    HideOnCursorOut(bool),
    AutoStart(bool),
    AppTitle(String),
    ViewMode(String),
    ListColumns(u32),
    OpenThemesDir,
    OpenDataDir,
    Export,
    Import,
    /// launcher-data.json 全文をクリップボードへ
    CopyData,
    /// UIフォント変更（None = 既定の Yu Gothic UI。再起動後に反映）
    FontFamily(Option<String>),
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let store::LoadOutcome {
            data,
            warning: load_warning,
            save_disabled,
            mtime: data_mtime,
        } = store::load();
        // デバッグ用: 起動時に開くタブを指定できる（RLAUNCH_START_TAB=1 等）
        let start_tab = std::env::var("RLAUNCH_START_TAB")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&t| t < data.tabs.len())
            .unwrap_or(0);
        let themes = theme::load_all(&store::themes_dir());
        let ui = theme::find(&themes, &data.settings.theme)
            .map(UiTheme::from_info)
            .unwrap_or_default();

        // トレイ・ホットキー・デスクトップフック・表示要求リスナー（main スレッド上）
        let tray = match crate::tray::build_tray(&data.settings.app_title) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("トレイ初期化失敗: {}", e);
                None
            }
        };
        external::wire_hotkey_events();
        let mut hotkeys = HotkeyRegistry::new()
            .map_err(|e| eprintln!("ホットキー初期化失敗: {}", e))
            .ok();
        let mut hotkey_failures = Vec::new();
        if let Some(reg) = hotkeys.as_mut() {
            hotkey_failures = reg.rebind(&wanted_hotkeys(&data));
        }
        if let Err(e) = crate::platform::desktop_hook::spawn_desktop_double_click_hook(|| {
            external::send(ExternalEvent::DesktopDoubleClick)
        }) {
            eprintln!("デスクトップフック失敗: {}", e);
        }
        if let Err(e) = crate::platform::single::spawn_show_listener(|| {
            external::send(ExternalEvent::ShowRequest)
        }) {
            eprintln!("表示要求リスナー失敗: {}", e);
        }

        let size = compute_main_size(&data, 0);
        // 透過テーマ（不透明度<1）のときだけ透過ウィンドウにする。
        // tiny-skia は透過ウィンドウでオーバーレイの残像が出るため、
        // 不透明テーマでは透過を無効にして残像を防ぐ。
        let want_transparent = ui.window_opacity < 1.0;
        let (id, open_task) = window::open(window::Settings {
            size,
            visible: false,
            resizable: false,
            decorations: false,
            transparent: want_transparent,
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

        let mut app = Self {
            data,
            themes,
            ui,
            main_id: id,
            main_visible: false,
            scale: 1.0,
            main_pos: None,
            settings_id: None,
            settings_hotkey_draft: String::new(),
            popup: None,
            popup_pos: None,
            folder: None,
            active_tab: start_tab,
            pinned: false,
            focused_cell: None,
            hovered_cell: None,
            hovered_tab: None,
            tabbar_hovered: false,
            drag: DragState::Idle,
            file_hovering: false,
            file_hover_window: None,
            drop_highlight: None,
            pending_drops: Vec::new(),
            last_drop_at: None,
            last_empty_click: None,
            ctx_menu: None,
            overlay: Overlay::None,
            search: None,
            toast: None,
            cursor: Point::ORIGIN,
            popup_cursor: Point::ORIGIN,
            modifiers: keyboard::Modifiers::default(),
            main_focused: false,
            unfocused_since: None,
            popup_unfocused_since: None,
            suppress_hide_until: None,
            cursor_out_since: None,
            icons: HashMap::new(),
            invalid_paths: HashSet::new(),
            tray,
            hotkeys,
            initial_shown: false,
            file_dialog_open: false,
            tab_drag: None,
            last_tab_click: None,
            data_mtime,
            save_disabled,
            titlebar_dragging: false,
            cursor_out_armed: false,
        };
        app.rebuild_icon_cache();
        app.recheck_invalid_paths();
        if let Some(w) = load_warning {
            app.show_toast(w);
        }
        for (spec, err) in hotkey_failures {
            app.show_toast(format!("ホットキー {} を登録できません: {}", spec, err));
        }

        // 初回表示は scale factor 取得後（ScaleFactor ハンドラ）に行う。
        // scale=1.0 のまま位置決めすると高DPI環境で初回位置がずれるため。
        let scale_task = window::scale_factor(id).map(Message::ScaleFactor);
        (
            app,
            Task::batch([open_task.map(|_| Message::Noop), scale_task]),
        )
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if window_id == self.main_id {
            crate::ui::main_window::view(self)
        } else if Some(window_id) == self.settings_id {
            crate::ui::settings_window::view(self)
        } else if self.popup.as_ref().map(|p| p.id) == Some(window_id) {
            crate::ui::group_popup::view(self)
        } else if self.folder.as_ref().map(|f| f.id) == Some(window_id) {
            crate::ui::folder_popup::view(self)
        } else {
            iced::widget::text("").into()
        }
    }

    pub fn theme(&self, _window: window::Id) -> iced::Theme {
        self.ui.to_iced_theme()
    }

    pub fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.settings_id {
            format!("{} 設定", self.data.settings.app_title)
        } else {
            self.data.settings.app_title.clone()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            external::subscription().map(Message::External),
            iced::event::listen_with(filter_event),
        ];
        if self.tick_needed() {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick),
            );
        }
        Subscription::batch(subs)
    }

    fn tick_needed(&self) -> bool {
        self.toast.is_some()
            || self.unfocused_since.is_some()
            || self.popup_unfocused_since.is_some()
            || self.file_hovering
            || !self.pending_drops.is_empty()
            || matches!(self.drag, DragState::Dragging { .. })
            || (self.main_visible && self.data.settings.hide_on_cursor_out && !self.pinned)
    }

    // ------------------------------------------------------------------
    // 共通ヘルパー
    // ------------------------------------------------------------------

    pub fn cells(&self, grid: GridRef) -> Option<&Vec<Option<GridCell>>> {
        match grid {
            GridRef::Tab(t) => self.data.tabs.get(t).map(|t| &t.items),
            GridRef::Group { tab, cell } => {
                match self.data.tabs.get(tab)?.items.get(cell)?.as_ref()? {
                    GridCell::Group(g) => Some(&g.items),
                    _ => None,
                }
            }
        }
    }

    pub fn cells_mut(&mut self, grid: GridRef) -> Option<&mut Vec<Option<GridCell>>> {
        match grid {
            GridRef::Tab(t) => self.data.tabs.get_mut(t).map(|t| &mut t.items),
            GridRef::Group { tab, cell } => {
                match self.data.tabs.get_mut(tab)?.items.get_mut(cell)?.as_mut()? {
                    GridCell::Group(g) => Some(&mut g.items),
                    _ => None,
                }
            }
        }
    }

    pub fn save(&mut self) {
        if self.save_disabled {
            self.show_toast(
                "データファイルを読み込めていないため保存を停止中です（再起動してください）",
            );
            return;
        }
        crate::model::store::renumber_tabs(&mut self.data);
        match store::save(&self.data, self.data_mtime) {
            Ok((mtime, conflict)) => {
                self.data_mtime = Some(mtime);
                if let Some(note) = conflict {
                    self.show_toast(note);
                }
            }
            Err(e) => self.show_toast(format!("保存に失敗: {}", e)),
        }
    }

    pub fn show_toast(&mut self, text: impl Into<String>) {
        self.toast = Some((
            text.into(),
            Instant::now() + std::time::Duration::from_secs(3),
        ));
    }

    /// テーマを適用（ui 再構築）
    pub fn apply_theme(&mut self, id: &str) {
        if let Some(info) = theme::find(&self.themes, id) {
            self.ui = UiTheme::from_info(info);
            self.data.settings.theme = id.to_string();
        }
    }

    /// アイコンキャッシュ再構築（Base64 → IconData。SVG/壊れ画像も安全に扱う）
    pub fn rebuild_icon_cache(&mut self) {
        self.icons.clear();
        let mut add = |id: &str, b64: &str| {
            if let Some(icon) = decode_icon(b64) {
                self.icons.insert(id.to_string(), icon);
            }
        };
        for tab in &self.data.tabs {
            for cell in tab.items.iter().flatten() {
                match cell {
                    GridCell::Launcher(item) => {
                        if let Some(b64) = &item.icon_base64 {
                            add(&item.id, b64);
                        }
                    }
                    GridCell::Group(g) => {
                        if let Some(b64) = &g.icon_base64 {
                            add(&g.id, b64);
                        }
                        for sub in g.items.iter().flatten() {
                            if let GridCell::Launcher(item) = sub {
                                if let Some(b64) = &item.icon_base64 {
                                    add(&item.id, b64);
                                }
                            }
                        }
                    }
                    GridCell::Widget(_) => {}
                }
            }
        }
    }

    /// 1アイテム分のアイコンをキャッシュに追加
    pub fn cache_icon(&mut self, id: &str, b64: &str) {
        if let Some(icon) = decode_icon(b64) {
            self.icons.insert(id.to_string(), icon);
        }
    }

    /// アクティブタブのパス有効性チェック
    pub fn recheck_invalid_paths(&mut self) {
        self.invalid_paths.clear();
        let Some(tab) = self.data.tabs.get(self.active_tab) else {
            return;
        };
        let mut check = |item: &crate::model::data::LauncherItem| {
            if !item.is_url() && !std::path::Path::new(&item.path).exists() {
                self.invalid_paths.insert(item.id.clone());
            }
        };
        for cell in tab.items.iter().flatten() {
            match cell {
                GridCell::Launcher(item) => check(item),
                GridCell::Group(g) => {
                    for sub in g.items.iter().flatten() {
                        if let GridCell::Launcher(item) = sub {
                            check(item);
                        }
                    }
                }
                GridCell::Widget(_) => {}
            }
        }
    }

    /// 現在のオーバーレイが対象にしているグリッド（表示先ウィンドウの決定に使う）
    pub fn overlay_grid(&self) -> Option<GridRef> {
        match &self.overlay {
            Overlay::ItemEdit { grid, .. }
            | Overlay::GroupEdit { grid, .. }
            | Overlay::UrlPrompt { grid, .. }
            | Overlay::ConfirmClear { grid, .. } => Some(*grid),
            _ => None,
        }
    }

    /// 検索結果を計算
    pub fn search_hits(&self) -> Vec<SearchHit> {
        let Some(search) = &self.search else {
            return Vec::new();
        };
        let q = search.query.to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let mut hits = Vec::new();
        for (ti, tab) in self.data.tabs.iter().enumerate() {
            for (ci, cell) in tab.items.iter().enumerate() {
                if let Some(GridCell::Launcher(item)) = cell {
                    if item.label.to_lowercase().contains(&q)
                        || item.path.to_lowercase().contains(&q)
                    {
                        hits.push(SearchHit {
                            tab: ti,
                            cell: ci,
                            label: item.label.clone(),
                            path: item.path.clone(),
                            tab_label: tab.label.clone(),
                        });
                    }
                }
            }
        }
        hits
    }
}

/// データから登録すべきホットキー一覧を構築
pub fn wanted_hotkeys(data: &LauncherData) -> Vec<(String, external::HotkeyAction)> {
    let mut list = vec![(
        data.settings.hotkey.clone(),
        external::HotkeyAction::ToggleMain,
    )];
    for tab in &data.tabs {
        let mut collect = |cell: &GridCell| {
            if let GridCell::Launcher(item) = cell {
                if let Some(hk) = &item.hotkey {
                    if !hk.trim().is_empty() && *hk != data.settings.hotkey {
                        list.push((
                            hk.clone(),
                            external::HotkeyAction::LaunchItem(item.id.clone()),
                        ));
                    }
                }
            }
        };
        for cell in tab.items.iter().flatten() {
            collect(cell);
            if let GridCell::Group(g) = cell {
                for sub in g.items.iter().flatten() {
                    collect(sub);
                }
            }
        }
    }
    // 正規化後のキー ID で重複除去（同一キーの二重登録は register が失敗し、
    // rebind のたびにエラートーストが出続けるため、ここで先着優先に落とす）
    let mut seen = HashSet::new();
    list.retain(|(spec, _)| match external::parse_hotkey(spec) {
        Ok(hk) => seen.insert(hk.id()),
        Err(_) => true, // パース不能なものは rebind 側でエラー報告させる
    });
    list
}

/// アクティブタブのグリッド構成からメインウィンドウの論理サイズを計算
pub fn compute_main_size(data: &LauncherData, active_tab: usize) -> Size {
    use layout::*;
    let s = &data.settings;
    let tab = data.tabs.get(active_tab).or_else(|| data.tabs.first());
    let cols = tab
        .map(|t| t.grid_columns)
        .unwrap_or(s.default_grid_columns) as f32;
    let rows = tab.map(|t| t.grid_rows).unwrap_or(s.default_grid_rows) as f32;
    let cell = s.cell_size as f32;
    let view_mode = tab
        .and_then(|t| t.view_mode.as_deref())
        .unwrap_or(&s.view_mode);

    let width = cell * cols + GRID_GAP * (cols - 1.0) + GRID_PADDING + BORDER_EXTRA;
    let grid_h = if view_mode == "list" {
        let list_cols = tab
            .and_then(|t| t.list_columns)
            .unwrap_or(s.list_columns)
            .clamp(1, 4) as f32;
        let total = cols * rows;
        let list_rows = (total / list_cols).ceil();
        LIST_ROW_HEIGHT * list_rows + LIST_GAP * (list_rows - 1.0)
    } else {
        cell * rows + GRID_GAP * (rows - 1.0)
    };
    let height =
        grid_h + GRID_PADDING + BORDER_EXTRA + TITLEBAR_HEIGHT + TABBAR_HEIGHT + STATUSBAR_HEIGHT;
    Size::new(width, height)
}

/// listen_with のフィルタ（fn ポインタ、キャプチャ不可）
fn filter_event(
    event: iced::Event,
    status: iced::event::Status,
    id: window::Id,
) -> Option<Message> {
    use iced::Event as E;
    match &event {
        E::Window(_) => Some(Message::IcedEvent(id, event)),
        E::Keyboard(keyboard::Event::ModifiersChanged(_)) => Some(Message::IcedEvent(id, event)),
        E::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
            // Escape はウィジェットに吸われても届ける（検索バーを一発で閉じるため）。
            // それ以外はテキスト入力等が受け取ったキーをグリッド操作に回さない。
            let is_escape = matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape));
            if is_escape || status == iced::event::Status::Ignored {
                Some(Message::IcedEvent(id, event))
            } else {
                None
            }
        }
        E::Mouse(iced::mouse::Event::WheelScrolled { .. }) => Some(Message::IcedEvent(id, event)),
        // ウィンドウ外リリースでドラッグが固着しないよう、左ボタンリリースは常に転送
        E::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::IcedEvent(id, event))
        }
        _ => None,
    }
}
