# RLaunch — CLaunch風ボタン型ランチャーの Rust 再構築プロジェクト

## 現状と目標

- 現行実装: Tauri v2 + React（`src/`, `src-tauri/`）。動作するが Web 系スタックからの脱却が目標。
- **進行中の作業: iced（Rust GUI）による完全再実装 → `rlaunch-iced/` サブディレクトリで開発**
  （ユーザー決定 2026-07-04）。完成してユーザーテストが通ったら旧実装（src/, src-tauri/ 等）を
  削除し、新実装をリポジトリルートへ昇格させる。
- 目標: 現行機能のパリティ + 本家 CLaunch の模倣強化（クローンに近いもの）。
- **ウィジェット機能は一旦廃止**（ユーザー決定 2026-07-04、ずっと後で再検討）。ただし旧データの
  WidgetItem はロード/セーブで壊さないよう型は維持し、UI では無効セルとして表示する。
- GUI フレームワークは iced のみ使用可。同伴クレートは `.claude/skills/iced-dev/SKILL.md` の
  承認済みリスト（2026-07-04 ユーザー承認）に従い、リスト外はユーザーに許可を求める。

## 開発スキル（作業前に必ず該当スキルを読む）

| スキル | 用途 |
|---|---|
| `clanch-spec` | 本家 CLaunch の挙動仕様・模倣ロードマップ（gap-analysis） |
| `rlaunch-legacy` | 現行 Tauri 版の機能インベントリ・データ互換・未配線バグ一覧 |
| `iced-dev` | iced 0.14 の設計方針・承認クレート・統合パターン・落とし穴 |
| `rlaunch-verify` | ビルド・実行・動作検証の手順 |
| `skill-evolve` | セッションの学びをスキルへ反映（自己進化） |

## 学び記録プロトコル（自己進化）

スキルの記述と実際の食い違い・非自明な問題の解決・CLaunch の新事実を発見したら、
`.claude/skills/_learnings/pending.md` にエントリを追記する。Stop フックが未処理の学びを
検知して skill-evolve での取り込みを促す（フック実体: `.claude/hooks/`、設定: `.claude/settings.json`）。
フックは **pwsh (PowerShell 7+)** 前提。cargo エラーの頻発集計は SessionStart で自動リセットされる。

## 注意

- `docs/` 配下のドキュメントは実装より古い箇所がある。現行仕様の正は rlaunch-legacy スキルと実コード。
- 既存ユーザーデータ `%APPDATA%/com.rlaunch.app/launcher-data.json` との互換を壊さないこと。
