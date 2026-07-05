---
name: rlaunch-verify
description: RLaunch（iced版）のビルド・実行・動作検証の手順。コード変更後の検証、アプリの起動確認、リリース前チェック、「動くか確認して」と言われたときに使用する。ウィンドウ表示の機械的検証（Win32 P/Invoke）手順も含む。
---

# rlaunch-verify — ビルド・実行・検証手順

ソースはリポジトリルート（`Cargo.toml` / `src/` / `assets/` / `tests/`）。cargo コマンドは
ルートで実行する（2026-07-05 に iced 版をルートへ昇格、旧 Tauri 実装は削除済み）。

### 1. 静的検証（コード変更のたびに必ず）

```powershell
cargo fmt --all -- --check      # フォーマット
cargo clippy --all-targets -- -D warnings   # リント（警告をエラー扱い）
cargo test                      # ユニットテスト
```

- clippy の警告は握りつぶさず直す。`#[allow]` を足すときは理由コメント必須。
- データモデル（serde 構造体）を触ったら、既存 launcher-data.json の読み込み互換テスト
  （tests/ 配下のフィクスチャ）を必ず通すこと。

### 2. 実行して動作確認（機能変更のたびに）

```powershell
cargo run   # デバッグ実行。常駐型なのでバックグラウンド実行で起動し、プロセス存在を確認する
```

- 常駐アプリなので起動直後はウィンドウが出ないことが正常（トレイ常駐）。
  プロセス確認: `Get-Process -Name rlaunch*`
- 終了させるとき: `Stop-Process -Name rlaunch* -Confirm:$false`（トレイメニュー相当の
  終了経路が未実装の間の暫定手段）
- **注意**: グローバルホットキー（Ctrl+Space等）を奪うため、検証後は必ずプロセスを終了する。
  多重起動防止があるので、前のプロセスが残っていると新しいビルドが起動しない。

### 3. 手動検証チェックリスト（変更箇所に応じて）

| 領域 | 確認項目 |
|---|---|
| 常駐 | トレイアイコン表示、左クリックでトグル、右クリックメニュー、終了でプロセス消滅 |
| ホットキー | Ctrl+Space トグル、カーソルのあるモニターに表示、変更後の再登録 |
| 表示/非表示 | フォーカス喪失で消える、ピン留めで消えない、Escape で消える |
| グリッド | セルサイズ・行列数変更でウィンドウサイズ追従、タブ切替 |
| D&D | エクスプローラーから exe/lnk/フォルダをドロップ→正しいセルに登録・アイコン表示 |
| 起動 | クリック起動、引数付き、管理者起動（UAC ダイアログ）、URL、フォルダ |
| データ | 再起動して配置・設定が保持される、旧 launcher-data.json が読める |
| テーマ | 切替の即時反映、透過テーマ |
| DPI | スケール150%のモニターと100%のモニター間で位置・サイズが正しい（環境があれば） |

- 自動化の選択肢: iced 0.14 はヘッドレステスト（iced_test）に対応。UIロジックの回帰は
  ヘッドレステストへの切り出しを検討（要調査・導入したら本スキルを更新）。

### 4. リリースビルド

```powershell
cargo build --release
# 成果物: target/release/*.exe（単一バイナリ。配布方式が決まったらここを更新）
```

### 5. ウィンドウ表示の機械的検証（スクショに頼らない）

常駐＋マルチウィンドウ＋マルチモニタなので、表示/非表示・位置・残像は Win32 P/Invoke で
機械的に確認できる（`iced-dev` スキルにコード断片あり）:
- `EnumWindows`+`GetWindowThreadProcessId`+`IsWindowVisible`+`GetWindowRect` で対象ウィンドウの
  表示状態と矩形を取得（マルチモニタで見失わない）
- `SetCursorPos`+`mouse_event` で右クリック/移動を再現し、`CopyFromScreen` でウィンドウ領域を
  キャプチャ → メニュー/ツールチップの残像や表示を目視
- カーソルをプライマリ中央に置いてから起動すると、モニタ間 DPI 差の座標ズレを避けられる
- テスト後は必ず `Stop-Process -Name rlaunch -Force`。プロセスが残ると多重起動防止で次が起動しない

## 落とし穴 / Learnings

<!-- skill-evolve がここに学びを追記する -->
- 検証中にアプリを起動したまま次のビルドをすると、Windows では exe がロックされて
  リンクエラー（os error 5 / 32）になる。ビルド前にプロセス終了を確認すること。
- rust-analyzer が `target/` や `src/` を掴んでいると git mv / rm / cargo clean が
  Permission denied になる。大きなファイル操作の前に `Stop-Process -Name rust-analyzer`
  （VSCode が自動で再起動する）。
