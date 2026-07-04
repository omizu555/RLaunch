# 現行RLaunch 機能インベントリ（実装コード全数調査版 2026-07）

移植パリティチェックリストを兼ねる。「移植」列: ✅=そのまま移植 / 🔁=iced流に再設計 /
🔧=未配線を直して移植 / ❓=設計判断が必要（ユーザー確認）

## 1. 常駐・呼び出し・ウィンドウ

| 機能 | 実装仕様 | 移植 |
|---|---|---|
| トレイ常駐 | 左クリック=トグル、右クリックメニュー（表示/非表示・設定・終了）。─/✕は hide のみ、終了はトレイのみ | ✅ ※「設定」は main 表示になるTODOバグ→修正 |
| グローバルホットキー | Ctrl+Space デフォルト、設定で変更可。表示位置 cursor/center/remember | ✅ |
| デスクトップダブルクリック表示 | WH_MOUSE_LL フックで Progman/WorkerW 上のダブルクリック検出→カーソル中心トグル表示（desktop_hook.rs、専用スレッド GetMessageW ポンプ） | ✅ ロジック流用 |
| 自動非表示 | フォーカス喪失 300ms 後 hide。D&D中/ウィンドウドラッグ中(2秒安全弁)/子ウィンドウ表示中はフラグで抑制 | ✅ ※CLaunch式「カーソルアウトで非表示」への強化は gap-analysis 参照 |
| ピン留め | 📌で自動非表示・起動後hide・ホットキーhide・Escape を全抑止 | ✅ |
| 多重起動防止 | single-instance プラグイン、2つ目は既存 main を show+focus | ✅ |
| ウィンドウサイズ自動計算 | width=cellSize×cols+6×(cols-1)+20+2、height+=36+36+28。リスト時: 行高32+gap2×ceil(スロット/listColumns) | ✅ 定数ごと移植 |
| フレームレス構成 | decorations:false, alwaysOnTop, skipTaskbar, resizable:false, transparent, 初期非表示。カスタムタイトルバー（☰/タイトル=ドラッグ領域/📌/─/✕）。appTitle 変更可・空欄で非表示 | ✅ |
| 位置記憶 | window-state プラグイン任せ（windowPosition=remember の実体） | 🔁 自前で位置保存 |
| マルチモニター | GetCursorPos+MonitorFromPoint+GetMonitorInfoW の rcWork にクランプ（get_cursor_monitor_info） | ✅ ロジック流用 |

## 2. タブ

| 機能 | 実装仕様 | 移植 |
|---|---|---|
| CRUD | ＋追加 / ダブルクリックでインライン改名 / 右クリック→削除（アイテム数付き confirm、最後の1つは不可、アクティブ削除で左隣へ） | ✅ |
| D&D並び替え | ポインタベース 5px 閾値、order 再採番 | ✅ |
| 複製 | ディープコピー+新UUID、「XXX のコピー」 | ✅ |
| タブ設定ダイアログ | 名前・列1-20×行1-10・表示モード（全体設定使用チェック）・リスト列数1-4。**リサイズは行列座標でリマップ保持**（remapGridItems） | ✅ |
| ホイール切替 | タブバー上ホイールで前後（ループなし） | ✅ |
| D&D中ホバー切替 | アイテムドラッグ中タブに0.5sホバーで切替（タブまたぎ移動） | ✅ |

## 3. グリッド・アイテム

| 機能 | 実装仕様 | 移植 |
|---|---|---|
| グリッド表示 | 空セルも表示。ステータスバー「タブ名 — N アイテム / M スロット」 | ✅ |
| リスト表示 | 同一エンジンで行高32px横長セル。全体→タブ→グループの3層フォールバック | ✅ |
| セルサイズ | 設定40-120px(step4) + Ctrl+ホイール段階変更 [40,48,56,64,72,80,96,112,120] | ✅ |
| キーボードナビ | 矢印（ラップアラウンド）/Enter起動/Delete解除（確認なし）。タブ切替でリセット | ✅ |
| セル間D&D | 5px閾値、swap方式、カーソル追従ゴースト（アイコン+ラベル半透明）、justDragged でクリック誤発火防止 | ✅ 挙動仕様維持 |
| グループへドロップ | 非グループアイテムをグループセルへ→先頭空きへ移動（満杯なら通知） | ✅ |
| OSファイルD&D | enter/over/drop/leave。座標→セル特定・ハイライト。占有セルへは挿入シフト（末尾溢れ消失）、空セルへは配置。複数一括は右方向へ順次+結果通知 | ✅ iced-dev §4 の方式で |
| 登録パイプライン | get_file_info→種別判定(exe/msi=executable, lnk=shortcut, url=url, dir=folder, 他=document)→lnk解決→アイコン抽出→LauncherItem | ✅ |
| ダイアログ登録 | 空セル右クリック/ダブルクリック→ファイル選択（フィルタ: 実行/ショートカット/全部）・フォルダ選択 | ✅ rfd で |
| URL登録 | prompt入力、hostname を自動ラベル化 | ✅ |
| 起動 | ShellExecuteW "open"（コンソールフラッシュ回避のため Command 不使用）。起動後 hideOnLaunch | ✅ |
| 管理者起動 | ShellExecuteW "runas"（右クリックメニューから） | ✅ |
| 場所を開く | explorer.exe /select, | ✅ |
| 編集ダイアログ | ラベル/パス/引数/作業Dir/ウィンドウ状態/管理者チェック/個別ホットキー/アイコン（ライブラリ選択）。起動統計表示。Ctrl+Enter保存 | ✅ |
| アイコンライブラリ | AppData/icons/ の画像を data URL 列挙。初回にデフォルトSVG46種書き出し | ✅ |
| 個別ホットキー | 非表示のままアイテム直接起動。全タブ走査で登録、トグルキーと同一はスキップ | ✅ |
| 起動統計 | launchCount++/lastLaunchedAt。リッチツールチップ（パス/引数/統計） | ✅ |
| パス有効性チェック | タブ表示時に fs.exists、無効は ⚠ オーバーレイ | ✅ |
| フォルダ動作切替 | folderAction: open(Explorer)/browse(内蔵ブラウザ)。右クリックでトグル | ✅ |
| コンテキストメニュー | セル種別ごと出し分け、画面外クランプ | ✅ |
| 検索 Ctrl+F | 全タブのラベル+パス部分一致。↑↓/Enter起動/Ctrl+Enterタブ移動 | ✅ |
| トースト通知 | 下部3秒 | ✅ |

## 4. グループ / フォルダブラウザ

- **グループ編集**（独立ウィンドウ）: 名前/絵文字16種+カラー8色+ライブラリ画像/2-8列×1-6行
  （ミニプレビュー）/表示モード・リスト列数（親継承 or 個別）。リサイズは既存アイテム保持。
- **グループポップアップ**（独立・reusable）: クリックでカーソル位置基準（作業領域クランプ）に
  ミニランチャー表示。ヘッダードラッグ移動可。フォーカス喪失で自動クローズ（移動後600msガード）、
  メインがフォーカス取得でも閉じ、閉じた後メイン非フォーカスならメインも hide。
  中身はメイングリッド同等（D&D登録・並び替え・起動・編集・解除）。ウィジェット/ネスト不可。
  フォルダ browse のみ親へ委譲。
- **フォルダブラウザ**（独立・reusable、500×460）: list_directory で一覧（フォルダ先頭+名前順、
  拡張子別絵文字12カテゴリ、サイズ表示）。クリックで潜る/起動して閉じる、⬆/Backspace で戻る、
  Explorerで開く。
- 移植: 🔁 iced のマルチウィンドウ（or オーバーレイ）で同等機能。reusable/ハンドシェイクの
  複雑さは単一プロセスで消える。

## 5. ウィジェット（❓ 方式の設計判断が必要）

- Canvas 2D、`draw(ctx, w, h, config, data)`。data = { now, systemInfo?(1500msキャッシュ),
  clicked?, invoke? }。DPRスケーリングは呼び出し側処理。エラー時 ⚠ フォールバック描画。
- ビルトイン6種（TS直import）: analog-clock / digital-clock / countdown-timer / cpu-monitor /
  memory-monitor / date-calendar
- プラグイン20種（widget_scripts/*.js を include_str! 埋め込み→AppDataへ書き出し→new Function ロード）:
  binary-clock, stopwatch, world-clock, moon-phase, weather-icon, battery-level,
  pomodoro(通知音再生), network-status, year-progress, daylight-info, dice, pulse-animation,
  quick-note, dual-monitor, matrix-rain, daily-quote, color-wheel, breathing-guide,
  fps-counter, particles ※docs の一覧（whitenoise等）は誤り
- configSchema 7型: color/checkbox/select/text/number/datetime/file(音声ファイル選択)。
  共通設定: 更新間隔(16ms-1h)・横/縦スパン(1-4、CSS Grid span で複数セル占有)
- 選択ウィンドウ(400×520)・設定ウィンドウ(380×480、動的フォーム生成)
- 既知バグ: ウィンドウ非表示時も setInterval が回り続ける（ARCHITECTURE.md の記載と実装が乖離）
  → iced 版は Subscription 方式で自然に解決（iced-dev §10）
- **移植方針（決定 2026-07-04）**: ウィジェット機能は一旦廃止（ずっと後で再検討）。
  データ互換のため WidgetItem の型・serde は維持し、ロード/セーブで温存する。
  UI では「🧩（無効）」のような不活性セルとして表示し、解除（セルクリア）だけ可能にする。

## 6. テーマ

- テーマ JSON: `{ id(=ファイル名), label, author, variables: { CSS変数名: 値 } }`。
  変数18種+透過用2種: --bg-primary/--bg-secondary/--bg-button/--bg-button-hover/
  --bg-button-active/--bg-button-empty/--text-primary/--text-secondary/--text-muted/
  --border-color/--accent-color/--accent-hover/--shadow-color/--danger-color/--success-color/
  --warning-color/--border-radius/--border-radius-sm/--window-opacity(0-1)/--window-effect(none|mica|acrylic)
- ビルトイン6種（Rust埋め込み、初回書き出し・上書きしない）: dark(Catppuccin Mocha系)/
  light(Latte系)/classic/flat-white/flat-dark/mono
- サンプル: `resources/sample-themes/*.json` バンドル。list_themes がユーザー themes/ と
  統合列挙（同IDユーザー優先、コピー不要）
- リアルタイムプレビュー: settings-preview イベントで即時反映、キャンセルで巻き戻し
- Mica/Acrylic: --window-effect を Rust set_window_effect（tauri Effects）で適用
- 移植: 🔁 JSON 形式は互換維持し、変数→iced Palette/Extended + 独自 style 関数へマップ
  （iced-dev §7）。Mica/Acrylic の iced 対応は要調査（要検証）

## 7. データ・永続化・Rustコマンド

- 保存先: `%APPDATA%/com.rlaunch.app/launcher-data.json`（Tauri Store, autoSave）。
  トップレベル `{ settings, tabs }`。バックアップ: 起動時 .bak.1→2→3 ローテーション。
- エクスポート/インポート: JSON全文コピー/保存ダイアログ出力/インポート（上書き=全置換 or
  マージ=既存タブ保持+新規追加・設定上書き）
- Tauri コマンド26個の内訳（iced版では通常の関数になる）:
  - file_info: get_file_info / icon_extractor: extract_icon（**PowerShell子プロセス実装**）/
    lnk_resolver: resolve_lnk（**PowerShell+WScript.Shell実装**）→ 両者は Win32 ネイティブに置換
  - app_launcher: launch_app / run_as_admin / open_file_location / get_store_path /
    set_window_effect / hide_webview_window / get_cursor_position / get_cursor_monitor_info /
    list_directory
  - system_info: get_system_info（sysinfo, Mutex<System>）
  - theme_manager 4 / widget_manager 4 / icon_library 3
  - audio: pick_sound_file（PowerShell WinForms ダイアログ→rfd に置換）/ read_sound_file
    （10MB上限、data URL 返却）
- クレート: windows 0.57, sysinfo 0.32, base64 0.22（バージョンは現行 Cargo.toml 進拠）

## 8. 未配線・既知バグ（iced版で直すか削るか明示的に決める）

1. `autoStart`: UI・保存はあるが autostart プラグインの enable()/disable() 呼び出しが無い → 🔧
2. トレイ「設定」メニュー: 設定ウィンドウでなく main を表示するだけ（lib.rs に TODO） → 🔧
3. `autoHide` 設定: 保存されるが App.tsx は true ハードコードで無視、設定UIにも無い → 🔧（設定UI追加）
4. `windowState`(normal/maximized/minimized): 編集UIあり、launch_app は常に SW_SHOWNORMAL → 🔧
5. `runAs` チェック: 保存されるが通常クリック起動で参照されない（管理者起動は右クリックのみ） → 🔧
6. `windowEffect`/`windowOpacity`/`windowX`/`windowY`: 型に残存する死にフィールド → 削除
7. アイコン変更: マニュアルは「カスタム画像選択可」だが実装はライブラリ選択のみ → どちらかに揃える
8. ウィジェットの非表示時描画停止: 記載のみで未実装 → iced Subscription で解決
9. タブ削除の確認有無が経路で不一致（Delete キー=確認なし、メニュー=確認あり） → 統一
10. insertGridCell の末尾溢れは**消失**する仕様 → 維持か要検討

## 9. 将来候補（docs の未実装案、着手前にユーザーと優先度確認）

- UX改善提案書: P-46 タイトルバーカスタマイズ / P-47 セル角丸・間隔設定 / P-49 マルチモニター強化 /
  P-51 フェードアニメーション
- 開発計画書: テーマエディタGUI / クリップボード連携 / ウィンドウ参照登録 / 連続起動モード /
  UWP対応 / i18n
- docs/案/: パイメニュー・マウスジェスチャー等20案（全て構想段階）
- CLaunch 模倣の拡張は `clanch-spec` の gap-analysis.md が正
