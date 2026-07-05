# iced 0.14 統合パターン詳解（Windowsランチャー向け）

API 名は docs.rs の iced 0.14 に基づく（2026-07調査）。実装時にコンパイルで裏を取り、
相違があれば `_learnings/pending.md` に記録して本ファイルを訂正すること。

## 1. daemon 雛形とマルチウィンドウ

```rust
fn main() -> iced::Result {
    // ここ（mainスレッド、run前）で single-instance 判定・GlobalHotKeyManager・TrayIcon を生成
    iced::daemon(App::boot, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)      // |state, window_id| -> Theme  ウィンドウ毎に変えられる
        .title(App::title)      // |state, window_id| -> String
        .run()
}
```

- `window::open(Settings) -> (Id, Task<Id>)` で新規ウィンドウ。boot 時にメインウィンドウを
  `visible: false` で開いておき、表示要求で `set_mode(Windowed)` + `move_to` + `gain_focus`。
- 主要 window API: `close(Id)` / `move_to(Id, Point)` / `set_mode(Id, Mode)` /
  `gain_focus(Id)` / `position(Id)` / `size(Id)` / `monitor_size(Id)` / `scale_factor(Id)`。
- ウィンドウイベント購読: `iced::event::listen_with` は **fnポインタ限定**
  （`fn(iced::Event, Status, window::Id) -> Option<Message>`、環境キャプチャ不可）。第1引数は
  全イベントの enum なので `iced::Event::Window(e) => ...` でマッチして window::Event を取り出す。
  Event: `Opened/Closed/Moved/Resized/Rescaled/Focused/Unfocused/CloseRequested/FileHovered/
  FileDropped/FilesHoveredLeft/RedrawRequested`。
- State には `HashMap<window::Id, WindowKind>`（Main/Settings/GroupPopup/...）を持ち、
  view/update で出し分ける。

### メインウィンドウの Settings（ランチャー標準構成）

```rust
window::Settings {
    size: computed_size,                    // グリッドから自動計算（論理px）
    position: window::Position::Specific(point), // または SpecificWith(|win, mon| ...)
    visible: false,
    resizable: false,
    decorations: false,
    transparent: true,
    level: window::Level::AlwaysOnTop,
    exit_on_close_request: false,
    platform_specific: window::settings::PlatformSpecific {
        skip_taskbar: true,
        drag_and_drop: true,        // false にすると FileDropped が来なくなる。必ず true
        undecorated_shadow: true,   // 注意: 上端に1pxの線が出る（公式ドキュメント記載）
        corner_preference: ..,      // Win11 の DWM 角丸制御（Default/DoNotRound/Round/RoundSmall）
        ..Default::default()
    },
    ..Default::default()
}
```

- Alt+Tab からも消したい場合は iced では不可 → `windows` crate で HWND に
  `WS_EX_TOOLWINDOW` を `SetWindowLongPtrW` で付与。HWND の取得は
  `window::run(id, |w: &dyn window::Window| ...) -> Task<T>`（0.14 で実在確認済み。
  `run_with_handle` は 0.13 時代の旧名で 0.14 には無い）。Window は raw-window-handle の
  `HasWindowHandle`/`HasDisplayHandle` を実装しており `window_handle()` から HWND を取り出せる。
- フレームレスでのウィンドウドラッグ移動（タイトルバー相当）は、タイトルバー領域の押下で
  `window::drag(id)` の Task を返すだけでよい（0.14 に実在確認済み。リサイズは
  `window::drag_resize(id, Direction)`）。

## 2. グローバルホットキー（global-hotkey 0.8）

```rust
// main() で（daemon run 前、mainスレッド）
let manager = GlobalHotKeyManager::new().unwrap();
let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Space);
manager.register(hotkey).unwrap();   // Err = 他アプリと衝突 → UIに出す
// manager と hotkey は drop させない（drop で解除される）→ static / State に move
```

Subscription への橋渡し（コールバック→channel→Message の定石）:

```rust
fn hotkey_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        iced::stream::channel(32, |mut output| async move {
            let (tx, mut rx) = futures::channel::mpsc::unbounded();
            GlobalHotKeyEvent::set_event_handler(Some(move |ev: GlobalHotKeyEvent| {
                let _ = tx.unbounded_send(ev);
            }));
            while let Some(ev) = rx.next().await {
                if ev.state == HotKeyState::Pressed {
                    let _ = output.send(Message::HotkeyPressed(ev.id)).await;
                }
            }
        })
    })
}
```

- **注意**: `Subscription::run` の引数はキャプチャ無しの fn ポインタ限定（`||...` の
  非キャプチャクロージャは可）。設定値等を渡したくなったら
  `Subscription::run_with(data, fn(&D) -> S)`（D: Hash + 'static）を使う。
- 再登録（ホットキー設定変更）: `manager.unregister(old)` → `register(new)`。
- RegisterHotKey ベースなので衝突時 `register()` が Err を返す。起動時エラーハンドリング必須。
- アイテム個別ホットキーも同じ Manager に多数登録し、`ev.id`（HotKey::id()）で引き当てる。

## 3. システムトレイ（tray-icon 0.24）

- main スレッド（daemon run 前 or boot 内）で `TrayIconBuilder` 生成。
  **TrayIcon インスタンスは drop したらトレイから消える** → State か static に保持。
- メニューは同梱の muda。`TrayIconEvent::set_event_handler` / `MenuEvent::set_event_handler`
  で受け、ホットキーと同様に channel → Subscription で Message 化。
- 左クリックでトグルにする場合 `with_menu_on_left_click(false)` にして
  `TrayIconEvent::Click` を自前処理。
- **既知問題**: application 構成で全ウィンドウを close するとイベントループが止まりトレイが
  無反応になる（discourse.iced.rs #704）。→ daemon + `Mode::Hidden` 方式なら回避できる。
- iced(winit) のイベントループが Windows メッセージをポンプするので追加のメッセージループは不要。

## 4. ファイルD&D登録（座標問題の解法）

1. `platform_specific.drag_and_drop = true`（既定）を維持。
2. `FileHovered(PathBuf)` でハイライト開始… ただし**どのセルかは座標が無いと分からない**。
3. ホバー中のセル特定・ドロップ先特定は Win32 で行う:
   ```text
   GetCursorPos() → 物理スクリーン座標
   - window::position(id)（論理）× scale_factor(id) → 物理ウィンドウ原点
   cursor_physical - origin_physical → 物理ウィンドウ内座標 → ÷ scale factor → 論理座標
   → グリッドレイアウト計算（セルサイズ/gap/padding は自前定数）でセル index 算出
   ```
4. `FileHovered` は連続発火しないので、ホバー中のハイライト追従はタイマー
   （`iced::time::every(50ms)` の Subscription をドラッグ中のみ有効化）で GetCursorPos を
   ポーリングする。
5. 複数ファイルドロップは `FileDropped` が**1ファイルずつ連続で発火**する。短時間バッファリング
   （例: 100ms デバウンス）でまとめてから登録処理へ。

## 5. アイコン抽出（.exe/.lnk → iced image）

- 第一候補: `windows` crate 直叩き。`SHGetFileInfoW(SHGFI_ICON|SHGFI_LARGEICON)` → HICON →
  `GetIconInfo`/`GetDIBits` で RGBA 化。lnk はシェルがリンク解決込みで返すので楽。
  高解像度（48/256px）は `SHCreateItemFromParsingName` → `IShellItemImageFactory::GetImage`。
- `windows-icons` crate（path→RgbaImage）もあるが小規模（約600行）なので採用時はユーザーに確認。
- 表示: `iced::widget::image::Handle::from_rgba(w, h, pixels)` → GPU アトラスにキャッシュされる。
- **COM 注意**: SHGetFileInfoW 系は CoInitializeEx が必要な場合がある。winit も COM/OLE を
  初期化するため、抽出は専用ワーカースレッド（そのスレッドで CoInitializeEx）で行い、
  `Task::perform` で非同期化 → `HashMap<PathBuf, image::Handle>` にキャッシュ。
- HICON/HBITMAP は `DestroyIcon`/`DeleteObject` でリーク防止。
- 既存データ互換: 現行版は Base64 PNG を launcher-data.json に保存している。iced 版でも
  Base64 PNG を保持し `Handle::from_bytes` で読めばデータ互換を保てる。

## 6. カーソルのあるモニターへの表示

iced に全モニター列挙 API は無い。Win32 で:

```text
GetCursorPos → MonitorFromPoint(MONITOR_DEFAULTTONEAREST) → GetMonitorInfoW
→ rcWork（作業領域、物理座標）内にウィンドウ矩形をクランプ
→ scale factor で論理座標へ変換 → window::move_to(id, point)
```

- 現行 Tauri 版の `get_cursor_monitor_info`（app_launcher.rs）と同じロジックが流用できる。
- `Position::SpecificWith(|window_size, monitor_size| point)` でモニター基準の中央寄せも可。
- フォーカス表示: `gain_focus(id)` は Windows のフォアグラウンドロックで効かないことがある。
  効かない場合は `AttachThreadInput` ワークアラウンド（windows crate）を検討。

## 7. テーマ / スキン

- iced 0.14 の `Theme` は enum（Light/Dark/Dracula/Nord/TokyoNight/Catppuccin系など22種）+
  `Custom`。`Theme::custom(name, Palette)` で基本色（background/text/primary/success/danger）、
  `Theme::custom_with_fn` で `palette::Extended`（hover等の派生色）まで制御。
- daemon の `.theme(|state, window_id| ...)` が State のテーマフィールドを返すだけで
  **実行時テーマ切替**が成立する。既存テーマ JSON（CSS変数 key-value）→ Palette/Extended への
  マッパーを書けば現行テーマ資産を継承できる（`rlaunch-legacy` の変数一覧参照）。
- ウィジェット単位のスタイルは関数スタイリング。**status 引数の有無はウィジェットによる**:
  container は `.style(|theme: &Theme| container::Style { background, border: Border { radius, .. }, .. })`
  （status 無し）、button 等のインタラクティブ系は `.style(|theme, status| button::Style { .. })`。
  `theme.extended_palette()` から色を引けば全ウィジェットがテーマ追従。
- 全体の背景透過: `.style(|state, theme| iced::theme::Style { background_color: Color::TRANSPARENT, .. })`
  + `transparent: true`。Mica/Acrylic は iced 標準に無い → HWND 直接操作が必要（要調査・要検証）。

## 8. 単一インスタンス / 自動起動

- 単一インスタンス: `windows` crate で `CreateMutexW` + `GetLastError()==ERROR_ALREADY_EXISTS`
  判定（約20行、追加クレート不要）。既存インスタンスへの「表示せよ」通知は
  `RegisterWindowMessageW` + `PostMessage`（HWND broadcast）か名前付きパイプで送り、
  受信側は Subscription で待つ。
- 自動起動: auto-launch crate（HKCU\...\Run）。exe 移動対策に起動時
  `is_enabled() && パス不一致 → 再enable` を入れる。

## 9. デスクトップダブルクリック検出

現行 Tauri 版の `desktop_hook.rs`（SetWindowsHookExW(WH_MOUSE_LL) + Progman/WorkerW 判定 +
専用スレッドの GetMessageW ポンプ）がそのまま流用できる。通知は mpsc → Subscription で
Message 化に置き換える。エッジファンクション（画面端検出）も同じフックの mousemove 監視で実装可能。

## 10. ウィジェット描画

- iced の `canvas` ウィジェット（`iced::widget::canvas`、`Program` trait 実装）が
  現行 Canvas 2D ウィジェットの移植先。`frame.fill/stroke/text` 等で描画。
- 定期更新は `iced::time::every(Duration)` の Subscription（表示中のみ購読 → 非表示時は
  自動で描画停止になる。現行版の「常時 setInterval」問題も同時に解決）。
- `canvas::Cache` で静的部分をキャッシュし、更新時のみ `clear()`。

## 11. 実装で確認済みの 0.14 API 差分（2026-07-05 の実装セッションで確定）

コンパイル・実動で裏取り済み。Web上の 0.12/0.13 サンプルとの相違点:

- `Space::new()` は**引数なし**。サイズは `.width()/.height()` で指定（`horizontal_space()` 相当は `Space::new().width(Length::Fill)`）
- `checkbox(is_checked: bool)` — ラベルは `.label("...")` ビルダーメソッド
- `Radius` に `From<[f32; 4]>` は無い。角ごとの指定は `Radius { top_left, top_right, bottom_right, bottom_left }` の構造体リテラルで
- `text_input::focus(id)` は削除済み。`iced::widget::operation::focus(iced::advanced::widget::Id::new("id"))` が **Task を直接返す**。汎用 Operation の実行は `iced::advanced::widget::operate(op)`（`iced::task` モジュールには widget() は再エクスポートされていない）
- `iced::stream::channel` のクロージャ引数は型推論できないことがある → `|mut output: futures::channel::mpsc::Sender<T>|` と明示
- `mouse_area` にあるのは on_press / on_release / on_right_press / on_enter / on_exit / on_move（**ダブルクリックは無い** → 自前で Instant 差分判定）
- `mouse_area::on_release` は**カーソルが領域上にある時のみ**発火。ウィンドウ外リリースは `listen_with` で `Mouse(ButtonReleased(Left))` を拾って掃除する（ドラッグ固着防止）
- `text_input` の on_submit は**修飾キーに関係なく** Enter をキャプチャする。Ctrl+Enter 分岐は on_submit を中立メッセージにして update 側で `modifiers.control()` を見る
- クリップボード: `iced::clipboard::write(String) -> Task`
- tokio を直接依存に足さずにブロッキング処理を Task 化するには `futures::channel::oneshot` + `std::thread::spawn` の自前 `blocking()` ヘルパーで足りる（アイコン抽出・lnk解決は必ずこれで包む。update 内で同期実行するとネットワーク lnk で数秒フリーズする）
- `tray-icon` 0.24: ダブルクリック時は `Click{button_state: Up}` が2回届く。トグル用途は **Down** でマッチすること（Upだと2回トグルして元に戻る）
- API の裏取りは docs.rs より `~/.cargo/registry/src/index.crates.io-*/iced*-0.14.*/src/` のローカルソースを grep するのが確実で速い
- **メモリ**: デフォルトの wgpu レンダラーは常駐アプリには重い（実測: リリースビルドで WS 212MB）。
  `default-features = false` + `features = ["tiny-skia", "crisp", "thread-pool", ...]` の
  ソフトウェア描画にすると **WS 25MB / Private 8MB / exe 14.7→8.3MB** まで落ちる。
  ランチャー程度の UI なら描画品質・速度の体感差は無い
- **日本語フォント**: iced の既定はフォールバック任せで日本語が見づらい。
  `daemon(...).default_font(Font::with_name(name))` で明示する（`with_name` は `&'static str`
  要求なので設定値は `Box::leak` する。builder は起動時のみ → 実行時変更は再起動が必要）。
  設定ファイルからフォント名だけ先読みする軽量 peek 関数を store に用意すると綺麗
- **ウィンドウの実表示検証**: `EnumWindows`+`IsWindowVisible`+`GetWindowRect` の P/Invoke を
  PowerShell から叩くと、スクリーンショット無しで表示/非表示・位置を機械的に検証できる
  （マルチモニタで見失わない。SetCursorPos と組み合わせてカーソル連動の E2E も可能）。
  ただしマルチモニタでモニタ間 DPI が異なる環境では GetWindowRect の座標と実描画位置が
  ずれてクリック自動化が外れることがある → 検証はカーソルをプライマリ中央に置いてから
  起動し、プライマリ（正座標・等倍）に出すと安定する
- **image ウィジェットは SVG 不可**（重要・クラッシュ源）: `image::Handle` にラスター以外
  （SVG バイト列）を渡すと、tiny-skia では描画時に `raster.rs "Image should be allocated"`
  で **panic**（wgpu は寛容にスキップするので気付きにくい）。SVG は別の `svg` ウィジェット
  （iced の `svg` feature、内部で resvg。exe +2MB / メモリ増はほぼ無し）で描く。
  base64 アイコンは SVG（`data:image/svg` or 先頭 `<svg`/`<?xml`）とラスターを判別して
  出し分ける。**ラスターも `from_bytes`（遅延デコード）でなく `::image::load_from_memory`
  で実デコード検証してから `from_rgba` にする**と、壊れた画像でも描画時 panic せず
  安全にスキップできる（旧データに SVG や壊れ画像が紛れていても落ちない）
- **tiny-skia + 透過ウィンドウ = オーバーレイ残像**: `transparent: true` のウィンドウで
  tiny-skia を使うと、閉じたオーバーレイ（ダイアログ/ツールチップ/メニュー）の領域が
  再描画されず前フレームが残像として残る。対策は**不透明テーマのときは `transparent: false`**
  にすること（`ui.window_opacity < 1.0` のときだけ透過ウィンドウにする）。透過が本当に要る
  テーマだけ残像を許容する形にすると、大多数の不透明テーマ利用者は残像ゼロになる
- **デバッグ用の起動タブ指定**: `std::env::var("RLAUNCH_START_TAB")` で起動時の active_tab を
  指定できるようにしておくと、特定タブでのみ再現するクラッシュを GUI 操作なしで再現できて便利
  （無害なので残置。今回のタブ3クラッシュ特定に有効だった）

## 参考実装

- https://github.com/Rahn-IT/frostbyte_terminal — ホットキー常駐+最前面出現（同型UI、最重要）
- https://github.com/GyulyVGC/sniffnet — 実行時テーマ切替・本格Windowsアプリ
- https://github.com/squidowl/halloy — マルチウィンドウ実戦例
- https://github.com/onagre-launcher/onagre — ランチャーUI（Linux向けだが構造参考）
- https://github.com/iced-rs/iced/tree/0.14/examples — multi_window / styling / todos
- https://github.com/iced-rs/awesome-iced — その他実例一覧
