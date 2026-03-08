# MyLauncher アーキテクチャ設計

## 1. 全体構成

```
┌──────────────────────────────────────────────────────────────┐
│  Vite マルチページビルド                                        │
│  ┌──────────┐ ┌──────────┐ ┌─────────────┐ ┌─────────────┐  │
│  │  main    │ │ settings │ │ widget-sel  │ │ widget-set  │  │
│  │ (React)  │ │ (React)  │ │  (React)    │ │  (React)    │  │
│  └────┬─────┘ └────┬─────┘ └──────┬──────┘ └──────┬──────┘  │
│       │            │              │               │          │
│       └──────┬─────┴──────────────┴───────────────┘          │
│              │  emit / listen (Tauri イベント)                 │
│  ┌───────────┴──────────────────────────────────────────┐    │
│  │                 Tauri v2 WebView                     │    │
│  │  ┌──────────────────────────────────────────────┐    │    │
│  │  │  Rust Backend (lib.rs + commands/)            │    │    │
│  │  │  - 9 Plugins  - 17 Commands  - Tray Icon    │    │    │
│  │  └──────────────────────────────────────────────┘    │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. フロントエンド構成

### 2.1 コンポーネントツリー (メインウィンドウ)

```
App.tsx
├── CustomTitleBar     ← ☰設定, 📌ピン, 最小化, 閉じる
├── SearchBar          ← Ctrl+F 検索
├── TabBar             ← タブ管理 (追加/削除/名変更/ホイール切替)
├── LauncherGrid       ← ボタングリッド + ポインタベースD&D
│   └── LauncherButton ← 個別セル（アプリ/ウィジェット/空）
│       └── WidgetRenderer ← Canvas描画コンテナ
├── ContextMenu        ← 右クリックメニュー
└── (通知トースト)      ← インライン実装
```

### 2.2 カスタムフック一覧

| フック | 責務 |
|---|---|
| `useTabManager` | タブ CRUD、グリッドデータ管理、リサイズ、永続化 |
| `useLauncher` | アプリ起動ロジック（通常/管理者/フォルダ/ドキュメント） |
| `useWidgetManager` | ウィジェット配置・設定変更・保存 |
| `useSettingsWindow` | 設定ウィンドウの開閉・通信ライフサイクル |
| `useWidgetSelectWindow` | ウィジェット選択ウィンドウの開閉・通信 |
| `useWidgetSettingsWindow` | ウィジェット設定ウィンドウの開閉・通信 |
| `useHotkey` | グローバルホットキーの登録・更新 |
| `useAutoHide` | フォーカス喪失時の自動非表示 |
| `useDragDrop` | Tauri ネイティブ D&D（OS ファイルドロップ） |
| `useWindowSize` | ウィンドウサイズ自動計算・リサイズ |

### 2.3 ユーティリティ

| モジュール | 責務 |
|---|---|
| `themeLoader.ts` | テーマ初期化・一覧取得・CSS変数動的適用 |
| `widgetLoader.ts` | マニフェスト読み込み・スクリプト動的ロード・キャッシュ |
| `launcherStore.ts` | Tauri Store ラッパー |

---

## 3. ウィンドウ構成

4つの WebviewWindow を使用。各ウィンドウは独立した HTML エントリポイントを持つ。

| ウィンドウ | HTML | メインコンポーネント | 用途 |
|---|---|---|---|
| `main` | `index.html` | `App.tsx` | ランチャー本体 |
| `settings` | `settings.html` | `SettingsWindow.tsx` | アプリ設定 |
| `widget-select` | `widget-select.html` | `WidgetSelectWindow.tsx` | ウィジェット選択 |
| `widget-settings` | `widget-settings.html` | `WidgetSettingsWindow.tsx` | ウィジェット設定 |

### 3.1 ウィンドウ間通信フロー

```
┌───────────┐    emit("settings-current")    ┌──────────────┐
│   main    │ ─────────────────────────────→ │   settings   │
│           │ ←───────────────────────────── │              │
│           │    emit("settings-changed")    └──────────────┘
│           │
│           │    emit("widget-select-init")  ┌──────────────┐
│           │ ─────────────────────────────→ │ widget-select│
│           │ ←───────────────────────────── │              │
│           │    emit("widget-select-result")└──────────────┘
│           │
│           │    emit("widget-settings-init")┌──────────────┐
│           │ ─────────────────────────────→ │widget-settings│
│           │ ←───────────────────────────── │              │
│           │    emit("widget-settings-result")└────────────┘
└───────────┘
```

---

## 4. Rust バックエンド構成

### 4.1 コマンドモジュール

```
src-tauri/src/
├── lib.rs              ← プラグイン登録 (9個), コマンド登録 (17個), setup, トレイ
├── main.rs             ← エントリポイント
└── commands/
    ├── mod.rs           ← pub use
    ├── icon_extractor.rs ← extract_icon (ExtractIconExW → PNG → Base64)
    ├── lnk_resolver.rs  ← resolve_lnk (lnk クレート)
    ├── file_info.rs     ← get_file_info
    ├── app_launcher.rs  ← launch_app, run_as_admin, open_file_location, get_cursor_position
    ├── system_info.rs   ← get_system_info (sysinfo クレート)
    ├── theme_manager.rs ← init_themes, list_themes, get_themes_dir_path
    └── widget_manager.rs← init_widgets, list_widgets, get_widget_script, get_widgets_dir_path
```

### 4.2 setup 処理

```rust
.setup(|app| {
    // 1. テーマ初期化 (ビルトイン JSON をAppDataにコピー)
    init_themes(app.handle());
    // 2. ウィジェット初期化 (マニフェスト + スクリプトをAppDataにコピー)
    init_widgets(app.handle());
    // 3. トレイアイコン構築 (表示/非表示, 設定, 終了)
    build_tray(app)?;
    Ok(())
})
```

---

## 5. ドラッグ&ドロップ アーキテクチャ

### 5.1 二重D&Dシステム

| 種別 | 実装方式 | フック | 用途 |
|---|---|---|---|
| OS ファイルドロップ | Tauri `onDragDropEvent` | `useDragDrop` | 外部ファイルをグリッドに登録 |
| セル間移動 | ポインタイベント | `LauncherGrid` 内 | セル間でアイテムを並べ替え |

### 5.2 ポインタベース D&D 詳細

```
pointerdown
  → DRAG_THRESHOLD (5px) を超えるまで待機
    → pointermove (document レベル)
       - isDragging = true
       - ソースセル: .dragging クラス
       - elementFromPoint() でホバー先検出
       - ターゲットセル: .drag-over クラス
    → pointerup (document レベル)
       - isDragging = false
       - ソース ↔ ターゲットを swap
       - justDragged ref → クリック誤発火防止
```

HTML5 Drag API は Tauri v2 のネイティブ D&D と競合するため使用不可。

---

## 6. ウィジェットプラグインのロードフロー

```
1. init_widgets() [Rust setup]
   └→ ビルトイン + サンプルの manifest.json / widget.js を AppData にコピー

2. list_widgets [Rust command]
   └→ AppData/widgets/*/manifest.json を走査 → Vec<WidgetManifest>

3. listWidgetManifests() [widgetLoader.ts]
   └→ invoke("list_widgets") → マニフェスト一覧

4. loadPluginDrawFn(widgetId) [widgetLoader.ts]
   └→ invoke("get_widget_script", { widgetId })
   └→ new Function(script + "; return draw;")()   ← ファクトリパターン
   └→ draw 関数をキャッシュ (Map<string, DrawFunction>)

5. WidgetRenderer.tsx
   └→ isBuiltinWidget(type) ? 直接 import : loadPluginDrawFn()
   └→ setInterval で draw(ctx, w, h, config, systemInfo) を繰り返し呼出
   └→ ウィンドウ非表示時は停止
```

---

## 7. テーマ適用フロー

```
1. init_themes() [Rust setup]
   └→ ビルトイン JSON を AppData/themes/ にコピー

2. listThemes() [themeLoader.ts]
   └→ invoke("list_themes") → テーマ一覧

3. applyThemeById(themeId) [themeLoader.ts]
   └→ テーマ一覧からIDで検索
   └→ applyThemeVariables(colors)
       ├→ data-theme-vars 属性から前回の変数名を取得・removeProperty
       ├→ colors の各 key-value を document.documentElement.style.setProperty
       └→ data-theme-vars 属性を更新

4. リアルタイムプレビュー
   └→ settings ウィンドウで選択
   └→ emit("settings-changed") → main で applyThemeById()
```

---

## 8. 状態管理

| データ | 管理方法 | 永続化 |
|---|---|---|
| タブ + グリッドデータ | `useTabManager` (useState + Store) | Tauri Store |
| AppSettings | `useSettingsWindow` (useState + Store) | Tauri Store |
| ウィジェット描画状態 | `WidgetRenderer` (useRef + setInterval) | なし |
| ドラッグ状態 | `LauncherGrid` (useRef) | なし |
| ウィンドウ位置/サイズ | `window-state` プラグイン | 自動 |

---

## 9. CSS 設計

| ファイル | スコープ |
|---|---|
| `App.css` | グローバルリセット、レイアウト基盤、通知 |
| `CustomTitleBar.css` | タイトルバー |
| `TabBar.css` | タブバー |
| `LauncherGrid.css` | グリッド、D&Dフィードバック |
| `ContextMenu.css` | コンテキストメニュー |
| `SearchBar.css` | 検索バー |
| `SettingsWindow.css` | 設定ウィンドウ |
| `WidgetSelectWindow.css` | ウィジェット選択 |
| `WidgetSettingsWindow.css` | ウィジェット設定 |
| `themes/variables.css` | CSS変数ベーステーマ（テーマJSONで動的上書き） |

---

## 10. 既知の技術的制約

1. **WebView2 D&D 制約**: Windows の WebView2 は HTML5 Drag API 使用時に Tauri ネイティブ D&D を無効化するため、セル間D&Dはポインタイベントで実装
2. **`new Function()` セキュリティ**: ウィジェットプラグインの JS 実行に `new Function()` を使用。信頼できるプラグインのみ想定
3. **sysinfo の更新間隔**: CPU 使用率の取得にはクールダウン (FETCH_COOLDOWN: 1500ms) を設定し過負荷を防止
4. **フレームレスウィンドウのドラッグ**: `data-tauri-drag-region` 使用。ドラッグ中に `blur` イベントが発火するため `isWindowDragging` フラグで回避

---

## 11. 旧コード（削除候補）

以下のファイルは独立ウィンドウ化により不要。現在もソースに残存:

| ファイル | 備考 |
|---|---|
| `WidgetSelectDialog.tsx` / `.css` | → `WidgetSelectWindow.tsx` に置換済み |
| `WidgetSettingsDialog.tsx` / `.css` | → `WidgetSettingsWindow.tsx` に置換済み |
