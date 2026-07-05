---
name: rlaunch-legacy
description: 旧Tauri版RLaunchの機能インベントリ・データモデル・データ互換仕様のリファレンス（実装コード調査に基づく正確版）。iced版で「旧仕様はどうだったか」「launcher-data.jsonのスキーマ」「既存データとの互換性」を確認するときに使用する。旧実装は削除済みで本スキルが記録の正本。
---

# rlaunch-legacy — 旧Tauri版の機能仕様とデータ互換

このスキルは旧 Tauri + React 版 RLaunch の記録。**旧実装コードと docs は 2026-07-05 に
削除済み**（iced 版がルートへ昇格）。旧版の挙動・データスキーマを知る必要が出たら本スキルが正本。
本家 CLaunch の挙動は `clanch-spec`、iced 実装は `iced-dev` を参照。

主用途は **launcher-data.json のデータ互換**（現 iced 版も同じファイルを読む）と、
移植時の設計判断の記録。本スキルは旧実装コードの全数調査に基づく。

## 参照ファイル

- [features.md](features.md) — 機能全数インベントリ・データモデル・（旧）Rustコマンド一覧・
  未配線バグと移植時の判断

## 移植の大方針

1. **データモデルとファイル互換を維持**: 保存先は `%APPDATA%/com.rlaunch.app/launcher-data.json`
   （トップレベル `{ settings: AppSettings, tabs: Tab[] }`）。iced 版は同スキーマを serde 構造体で
   再定義し、**既存ファイルをそのまま読み込める**ようにする（アイコンは Base64 PNG 文字列で互換）。
   テーマ `themes/*.json`・アイコンライブラリ `icons/` も同形式を継続。
2. **Rust ロジックの流用は選別**: sysinfo・カーソル/モニター計算・デスクトップフック
   （WH_MOUSE_LL）は流用可。ただし**アイコン抽出と lnk 解決は現行実装が PowerShell 子プロセス
   依存**（docs の「ExtractIconExW」「lnk クレート」記載は誤り）なので、iced 版では
   Win32 ネイティブ（SHGetFileInfoW / IShellLinkW COM）に置き換える（`iced-dev` §5）。
3. **フロントエンド(React)は全書き換え**: 7つの WebviewWindow 構成を iced daemon の
   マルチウィンドウへ再構成。emit/listen の ready→init ハンドシェイクは不要になる
   （単一プロセス内 State 共有で済む）。
4. **ウィジェットの JS プラグインは移植不可**: `new Function()` は WebView 前提。
   iced 版はビルトイン相当を `canvas` ウィジェットで Rust 実装し、プラグイン機構の再設計
   （rhai/WASM/廃止）は**ユーザーに確認**してから決める。
5. **未配線機能は「直して移植」**: 現行版には UI だけあって機能しない項目が複数ある
   （features.md §8）。iced 版では仕様通りに実装するか、削るかを明示的に決める。

## 中核データモデル（実装準拠）

- `AppSettings`: hotkey("Ctrl+Space"), defaultGridColumns(8)/Rows(4), cellSize(64, 実UI 40-120),
  showLabels, labelFontSize?, theme("dark"), hideOnLaunch(true), windowPosition("cursor" が実デフォルト),
  appTitle("RLaunch"), viewMode("grid"|"list"), listColumns(1-4)
  ※ autoStart/autoHide/windowEffect/windowOpacity/windowX/windowY は型にあるが実質未使用（§8参照）
- `Tab`: id, label, order, gridColumns, gridRows, items(長さ=cols×rows), viewMode?, listColumns?
- `GridCell` = `LauncherItem` | `WidgetItem` | `GroupItem` | null
- `LauncherItem`: path, args?, workingDir?, iconBase64?, iconPath?, libraryIcon?,
  type(executable|shortcut|folder|url|document), runAs?, windowState?, hotkey?,
  folderAction?("open"|"browse"), launchCount?, lastLaunchedAt?, createdAt, updatedAt
- `GroupItem`: label, icon?(絵文字), iconColor?, iconBase64?, libraryIcon?, items,
  gridColumns(2-8), gridRows(1-6), viewMode?, listColumns? — ネスト不可・ウィジェット格納不可
- `WidgetItem`: widgetType, config, updateInterval, colSpan?(1-4), rowSpan?(1-4)

## 落とし穴 / Learnings

<!-- skill-evolve がここに学びを追記する -->
- docs/アプリ仕様書.md の「.settings.dat」「com.mylauncher.app」「ビルトインテーマ Paper White 等」
  「サンプルウィジェット whitenoise/compass 等」は旧記述。実装の正: launcher-data.json /
  com.rlaunch.app / ビルトインテーマは6種（dark・light・classic・flat-white・flat-dark・mono）/
  プラグインウィジェットは features.md §5 の20種。
- 現行のセル間 D&D はポインタイベント自作（WebView2 で HTML5 Drag API がネイティブD&Dと競合
  するため）。iced では制約自体が消えるが、5px 閾値・ゴースト表示・スワップ方式という
  「挙動仕様」は維持する。
