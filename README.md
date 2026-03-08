# RLaunch

Windows 向けのボタン型デスクトップランチャーです。  
グリッド状に並んだボタンにアプリ・URL・フォルダ・ウィジェットなどを配置し、ホットキー一つで呼び出して素早くアクセスできます。
私は仕事で、CLaunchを使用していますが、ウィジェットとかあればいいのになという思いだけでAIに作成してもらいました。


## 特長

- **ホットキー / デスクトップダブルクリックで即呼び出し** — どこからでも表示／非表示
- **ドラッグ＆ドロップ登録** — エクスプローラーからファイルをドロップするだけ
- **タブで分類管理** — 用途別にタブを分けてアイテムを整理
- **グループ（サブフォルダ）** — 関連するアイテムをまとめてポップアップ表示
- **ウィジェット** — 時計・CPU モニターなどの情報パネルを配置（26 種同梱）
- **豊富なテーマ** — 18 種のビルトインテーマ（透過テーマ含む）＋カスタムテーマ対応
- **マルチモニター対応** — カーソルがあるモニターに表示
- **プラグイン拡張** — テーマ・ウィジェットを自作して追加可能

## 技術スタック

- [Tauri v2](https://v2.tauri.app/) (Rust バックエンド)
- React 19 + TypeScript 5
- Vite 7
- 対象OS: Windows 10 / 11 (64bit)

## 必要な環境

- [Node.js](https://nodejs.org/) 20 以上
- [Rust](https://www.rust-lang.org/tools/install) 1.83 以上
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)
- Windows 10/11 SDK（Visual Studio Build Tools）

## ビルド方法

### 開発モード

```bash
npm install
cargo tauri dev
```

### リリースビルド

```bash
npm install
cargo tauri build
```

ビルド成果物は `src-tauri/target/release/bundle/` に出力されます。  
MSI インストーラーおよび NSIS セットアップが生成されます。

## プロジェクト構成

```
src/             … フロントエンド（React + TypeScript）
src-tauri/       … バックエンド（Rust / Tauri）
widget_scripts/  … ウィジェットプラグインのソーススクリプト
docs/            … ドキュメント
```

## ドキュメント

- [操作マニュアル](docs/操作マニュアル.md)
- [アプリ仕様書](docs/アプリ仕様書.md)
- [アーキテクチャ](docs/ARCHITECTURE.md)
- [テーマの作り方](docs/テーマの作り方.md)
- [ウィジェットの作り方](docs/ウィジェットの作り方.md)

## ライセンス

MIT
