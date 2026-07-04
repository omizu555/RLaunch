# PostToolUse フック: cargo コマンドで同一エラーコードがセッション内で3回発生したら「学び候補」を自動起票する
# （集計は .claude/hooks/state/ に保持し、SessionStart(startup) フックがリセットする）
# 要 pwsh 7+。手動テスト時は stdin をパイプで与えること
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8

    # stdin は UTF-8 生ストリームで読む（[Console]::In はコンソール無しプロセスで CP932 になり
    # 日本語入りの rustc 出力で JSON が壊れる）
    $raw = ''
    if ([Console]::IsInputRedirected) {
        $raw = [System.IO.StreamReader]::new([Console]::OpenStandardInput(), [System.Text.UTF8Encoding]::new($false)).ReadToEnd()
    }
    if ([string]::IsNullOrWhiteSpace($raw)) { exit 0 }
    try { $input_json = $raw | ConvertFrom-Json } catch { exit 0 }
    if ($null -eq $input_json) { exit 0 }

    # cargo のビルド系コマンド以外は対象外（ツールチェーン指定 `cargo +nightly build` も許容）
    $command = [string]$input_json.tool_input.command
    if ($command -notmatch '\bcargo\s+(\+\S+\s+)?(build|check|clippy|test|run)\b') { exit 0 }

    # ツール出力からエラーコードを抽出（tool_response / tool_output どちらのフィールド名でも拾う）
    $outputText = ''
    foreach ($field in @('tool_response', 'tool_output')) {
        $v = $input_json.$field
        if ($null -ne $v) {
            $outputText += if ($v -is [string]) { $v } else { $v | ConvertTo-Json -Compress -Depth 5 }
        }
    }
    $codes = [regex]::Matches($outputText, 'error\[(E\d{4})\]') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique
    if (-not $codes) { exit 0 }

    $root = (Resolve-Path (Join-Path $PSScriptRoot '..' '..')).Path
    $stateDir = Join-Path $PSScriptRoot 'state'
    New-Item -ItemType Directory -Path $stateDir -Force | Out-Null
    $tallyPath = Join-Path $stateDir 'cargo-error-tally.json'
    $pending = Join-Path $root '.claude/skills/_learnings/pending.md'
    $threshold = 3

    # フックは並列実行されうるので、集計の読み書きと起票は名前付き Mutex で排他する
    $mutex = [System.Threading.Mutex]::new($false, 'Local\rlaunch-cargo-error-tally')
    try { $null = $mutex.WaitOne(5000) } catch [System.Threading.AbandonedMutexException] {}

    $filed = @()
    try {
        $tally = @{}
        if (Test-Path $tallyPath) {
            try {
                (Get-Content -Path $tallyPath -Raw -Encoding UTF8 | ConvertFrom-Json).PSObject.Properties |
                    ForEach-Object { $tally[$_.Name] = @{ count = [int]$_.Value.count; suggested = [bool]$_.Value.suggested } }
            } catch { $tally = @{} }
        }

        foreach ($code in $codes) {
            if (-not $tally.ContainsKey($code)) { $tally[$code] = @{ count = 0; suggested = $false } }
            $tally[$code].count++
        }

        # 起票（Add-Content）を先に行い、成功したものだけ suggested=true として永続化する
        # （先に suggested を保存すると、起票失敗時にその学び候補が二度と出なくなる）
        $today = Get-Date -Format 'yyyy-MM-dd'
        foreach ($code in $codes) {
            if ($tally[$code].count -ge $threshold -and -not $tally[$code].suggested) {
                if (Test-Path $pending) {
                    $entry = @"

## [$today] cargo-error-$code-頻発（自動検知）
- 対象スキル: iced-dev
- 発見: エラー $code がこのセッションで${threshold}回以上発生した（PostToolUseフックの自動集計）
- 根拠: .claude/hooks/state/cargo-error-tally.json
- 反映案: 原因と対処パターンが定まったら iced-dev の「落とし穴 / Learnings」へ記録。単なる作業中の一過性エラーだったなら破棄してよい
"@
                    try {
                        Add-Content -Path $pending -Value $entry -Encoding UTF8
                        $tally[$code].suggested = $true
                        $filed += $code
                    } catch {}  # 書けなければ suggested のままにせず次回再挑戦
                }
            }
        }

        try { $tally | ConvertTo-Json -Depth 3 | Set-Content -Path $tallyPath -Encoding UTF8 } catch {}
    } finally {
        try { $mutex.ReleaseMutex() } catch {}
        $mutex.Dispose()
    }

    if ($filed.Count -gt 0) {
        $ctx = "rustc エラー $($filed -join ', ') が${threshold}回以上発生したため、学び候補を _learnings/pending.md に自動起票しました。解決したら対処法を添えて skill-evolve で反映してください。"
        @{ hookSpecificOutput = @{ hookEventName = 'PostToolUse'; additionalContext = $ctx } } | ConvertTo-Json -Compress -Depth 3
    }
} catch {
    # フックの失敗でツール実行フローを妨げない
}
exit 0
