# SessionStart フック: スキル索引と学び記録プロトコルをコンテキストに注入する
# 要 pwsh 7+。手動テスト時は stdin をパイプで与えること（例: '{"source":"startup"}' | pwsh -File ...）
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8

    # stdin は UTF-8 生ストリームで読む（[Console]::In はコンソール無しプロセスで CP932 になり日本語が壊れる）
    $raw = ''
    if ([Console]::IsInputRedirected) {
        $raw = [System.IO.StreamReader]::new([Console]::OpenStandardInput(), [System.Text.UTF8Encoding]::new($false)).ReadToEnd()
    }
    $source = ''
    try { $j = $raw | ConvertFrom-Json; if ($null -ne $j) { $source = [string]$j.source } } catch {}

    # 新規セッション開始時に cargo エラー集計をリセット（頻発検知をセッション単位にするため）
    if ($source -eq 'startup') {
        $stateDir = Join-Path $PSScriptRoot 'state'
        if (Test-Path $stateDir) { try { Remove-Item -Recurse -Force $stateDir -Confirm:$false } catch {} }
    }

    $root = (Resolve-Path (Join-Path $PSScriptRoot '..' '..')).Path
    $skillsDir = Join-Path $root '.claude/skills'

    $lines = @()
    $lines += '=== RLaunch 開発スキル索引（SessionStartフックによる自動注入） ==='
    $lines += 'このプロジェクトは Tauri 版 RLaunch を iced で再構築中。該当作業の前に必ず対応スキルを読むこと:'

    if (Test-Path $skillsDir) {
        Get-ChildItem -Directory $skillsDir | Where-Object { $_.Name -notlike '_*' } | ForEach-Object {
            $skillMd = Join-Path $_.FullName 'SKILL.md'
            if (Test-Path $skillMd) {
                $desc = ''
                try {
                    foreach ($l in (Get-Content $skillMd -Encoding UTF8 -TotalCount 10)) {
                        if ($l -match '^description:\s*(.+)$') {
                            # description の最初の一文だけを索引に載せる
                            $desc = ($Matches[1] -split '。')[0]
                            break
                        }
                    }
                } catch { return }  # 読めないスキルはスキップ
                $lines += ('- ' + $_.Name + ': ' + $desc)
            }
        }
    }

    $lines += ''
    $lines += '【学び記録プロトコル】スキルの記述と実際の食い違い・ハマった末に解決した非自明な問題・'
    $lines += '本家CLaunchの新事実を発見したら、その場で .claude/skills/_learnings/pending.md に'
    $lines += 'エントリを追記すること（形式は同ファイル冒頭参照）。Stopフックが取り込みを促す。'

    $pending = Join-Path $skillsDir '_learnings/pending.md'
    if (Test-Path $pending) {
        try {
            $count = @(Select-String -Path $pending -Pattern '^##\s*\[\d{4}-\d{2}-\d{2}\]' -Encoding utf8).Count
            if ($count -gt 0) {
                $lines += ''
                $lines += ("⚠ 未処理の学びが ${count} 件あります。手が空いたら skill-evolve スキルで各スキルへ反映してください。")
            }
        } catch {}
    }

    $lines -join "`n"
} catch {
    # フックの失敗でセッションを妨げない（フェイルオープン）
}
exit 0
