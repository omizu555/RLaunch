---
name: iced-dev
description: iced（Rust GUIフレームワーク）でWindows常駐型ランチャーを開発するためのガイド。icedのコードを書く・設計する・依存クレートを選ぶ・ビルドエラーやウィンドウ挙動の問題を調べるときに使用する。バージョン規約、daemon構成、ホットキー/トレイ/D&D統合の定石と落とし穴を含む。
---

# iced-dev — iced開発ガイド（RLaunch再構築用）

## バージョン規約（2026-07時点）

- **iced 0.14 系に固定**する（`iced = "0.14"`）。0.14.0 は 2025-12-07 リリースの最新安定版。
- master（0.15開発中）のAPIは流動的。docs.rs は必ず **0.14 のバージョン指定ページ**で読む。
- iced は 1.0 未満でメジャーごとに破壊的変更がある（0.12→0.13でトレイトから関数ベースへ全面刷新、
  0.13→0.14でも boot 引数追加・スタイル署名変更）。Web上のサンプルコードは 0.12/0.13 時代のものが
  多いので、**コピペ前に 0.14 の docs.rs で API の現存を確認**すること。
- features: `iced = { version = "0.14", features = ["image", "canvas", "tokio", "advanced"] }` を
  基本とする（image=アイコン表示に必須、canvas=ウィジェット描画に必須〔デフォルト無効〕、
  tokio=非同期Task、advanced=低レイヤが必要になったら）。

## アーキテクチャの基本方針

- **`iced::application` ではなく `iced::daemon` を使う**。理由: application は全ウィンドウを
  閉じるとプロセス終了するが、daemon はウィンドウ0枚でも常駐し続ける。トレイ常駐ランチャーは
  「ウィンドウを閉じてもプロセスは生きる」が必須要件。終了は `iced::exit` の Task で行う。
- Elm アーキテクチャ: `State` + `Message`(enum) + `update(&mut State, Message) -> Task<Message>`
  + `view(&State, window::Id) -> Element<Message>`。daemon では view/title が `window::Id` を
  受け取るので、Id→ウィンドウ種別のマップを State に持ち出し分ける。
- 外部イベント（ホットキー/トレイ/タイマー）は `Subscription` で Message 化する。
  コールバックベースのクレートは channel → `iced::stream::channel` で橋渡しする（詳細は patterns.md）。
- ウィンドウの表示/非表示は `window::set_mode(id, Mode::Hidden / Mode::Windowed)` でトグルする
  （close すると再生成コストがかかる。daemon なら close→open でも動くが hide 方式を基本とする）。

## 承認済み依存クレート（2026-07-04 ユーザー承認済み）

| crate | ver | 用途 | 備考 |
|---|---|---|---|
| iced | 0.14 | GUI本体 | ユーザー承認済み |
| global-hotkey | 0.8 | グローバルホットキー | tauri-apps製。**mainスレッドでManager生成必須** |
| tray-icon | 0.24 | システムトレイ | tauri-apps製。**drop するとトレイから消える→State保持** |
| auto-launch | 0.6 | 自動起動（HKCU Run） | 管理者権限不要 |
| windows | 0.6x | Win32 API 補完 | カーソル位置/モニター/アイコン抽出/ツールウィンドウ化 |
| sysinfo | 最新 | CPU/メモリ情報 | 現行版から流用 |
| serde / serde_json | 最新 | 設定・データ永続化 | |
| rfd | 最新 | ファイル選択ダイアログ | 現行のPowerShell子プロセス方式を置換 |
| image / png | 最新 | アイコンPNG変換 | |

上記以外のクレートを追加したくなったら**ユーザーに許可を求める**（単一インスタンス用の
`single-instance`、lnk解決の `lnk`、スクリプトエンジン等は未承認）。

## 三大落とし穴（先に読め）

1. **FileDropped にドロップ座標が無い**: iced の `window::Event::FileDropped(PathBuf)` は座標を
   持たない（OLEドラッグ中は WM_MOUSEMOVE も来ない）。セル特定は受信時に Win32 `GetCursorPos`
   → ウィンドウ論理座標へ変換してヒットテストする。→ patterns.md §4
2. **座標は論理px**: iced の座標は scale factor 適用後の論理座標。Win32 から得る物理座標
   （カーソル/モニター）は必ず `window::scale_factor(id)` 等で換算する。DPI が異なる
   マルチモニターで未換算だと位置ズレする。
3. **Win32系クレートのスレッド要件**: global-hotkey / tray-icon は Windows では
   「イベントループが回るスレッド（=mainスレッド）」で生成する。`daemon().run()` の前に
   main で生成し、インスタンスを drop させないこと。

## 参照ファイル

- [patterns.md](patterns.md) — 統合パターン詳解（daemon雛形、ホットキー/トレイのSubscription橋渡し、
  D&Dヒットテスト、アイコン抽出、モニター配置、テーマシステム、単一インスタンス）
- 参考実装: Frostbyte Terminal（ホットキー常駐、最重要）、sniffnet（テーマ/本格Windowsアプリ）、
  Halloy（マルチウィンドウ）、Onagre（ランチャーUI）→ URL は patterns.md 末尾

## 検証

ビルド・実行・動作確認の手順は `rlaunch-verify` スキルに従う。

## 落とし穴 / Learnings

<!-- skill-evolve がここに学びを追記する。iced のAPI記述が実際と違ったら必ず訂正すること -->
- （要検証: 本スキルの API 名は 2026-07 の docs.rs 調査に基づく。初回実装時にコンパイルで確認し、
  相違があればここを訂正する）
