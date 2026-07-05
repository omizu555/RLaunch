# RLaunch — CLaunch風ボタン型ランチャー（iced / Rust）

## 現状

- **本体は iced（Rust GUI）製。リポジトリルートが実装**（`Cargo.toml` / `src/` / `assets/` / `tests/`）。
  旧 Tauri v2 + React 実装は 2026-07-05 に削除し、iced 版をルートへ昇格済み。
- 目標: CLaunch（老舗Windowsランチャー）のクローンに近い使用感 + 旧版機能のパリティ。
- **ウィジェット機能は廃止**（ユーザー決定 2026-07-04、ずっと後で再検討）。旧データの WidgetItem は
  ロード/セーブで壊さないよう型を維持し、UI では無効セルとして表示する。
- GUI は iced のみ。同伴クレートは `.claude/skills/iced-dev/SKILL.md` の承認済みリスト
  （2026-07-04 ユーザー承認）に従い、リスト外はユーザーに許可を求める。
- 描画は tiny-skia（ソフトウェア）。**影(shadow)は使わない**（tiny-skia で残像化するため。
  詳細は iced-dev スキル）。

## ビルド・実行

```powershell
cargo build --release   # 成果物: target/release/rlaunch.exe（単一バイナリ）
cargo run               # 開発実行（常駐型。トレイに常駐しウィンドウは初回表示）
```
検証手順は `.claude/skills/rlaunch-verify/SKILL.md`。デバッグ用に `RLAUNCH_START_TAB=<n>` で
起動時タブを指定できる（特定タブ依存の不具合の再現に便利）。

## 開発スキル（作業前に必ず該当スキルを読む）

| スキル | 用途 |
|---|---|
| `clanch-spec` | 本家 CLaunch の挙動仕様・模倣ロードマップ（gap-analysis） |
| `rlaunch-legacy` | 旧版のデータモデル・データ互換仕様（launcher-data.json スキーマ）・移植時の判断記録 |
| `iced-dev` | iced 0.14 の設計方針・承認クレート・統合パターン・落とし穴（tiny-skia残像対策など） |
| `rlaunch-verify` | ビルド・実行・動作検証の手順 |
| `skill-evolve` | セッションの学びをスキルへ反映（自己進化） |

## 学び記録プロトコル（自己進化）

スキルの記述と実際の食い違い・非自明な問題の解決・CLaunch の新事実を発見したら、
`.claude/skills/_learnings/pending.md` にエントリを追記する。Stop フックが未処理の学びを
検知して skill-evolve での取り込みを促す（フック実体: `.claude/hooks/`、設定: `.claude/settings.json`）。
フックは **pwsh (PowerShell 7+)** 前提。cargo エラーの頻発集計は SessionStart で自動リセットされる。

## 注意

- 既存ユーザーデータ `%APPDATA%/com.rlaunch.app/launcher-data.json`（旧 Tauri 版と共通）との
  互換を壊さないこと。未知フィールドはラウンドトリップで温存する。
