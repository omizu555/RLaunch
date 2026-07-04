# Stop フック: 未処理の学びがあるままターンを終えようとしたら、skill-evolve での取り込みを促してブロックする
# 要 pwsh 7+。手動テスト時は stdin をパイプで与えること
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8

    # stdin は UTF-8 生ストリームで読む（[Console]::In はコンソール無しプロセスで CP932 になり壊れる）
    $raw = ''
    if ([Console]::IsInputRedirected) {
        $raw = [System.IO.StreamReader]::new([Console]::OpenStandardInput(), [System.Text.UTF8Encoding]::new($false)).ReadToEnd()
    }
    # 入力が空・不正のときは邪魔をしない（ConvertFrom-Json は空文字で例外を投げず $null を返す点に注意）
    if ([string]::IsNullOrWhiteSpace($raw)) { exit 0 }
    try { $input_json = $raw | ConvertFrom-Json } catch { exit 0 }
    if ($null -eq $input_json) { exit 0 }

    # stop_hook_active=true は既にブロック経由で継続中 → 二重ブロックせず停止を許可（無限ループ防止）
    if ($input_json.stop_hook_active -eq $true) { exit 0 }

    $root = (Resolve-Path (Join-Path $PSScriptRoot '..' '..')).Path
    $pending = Join-Path $root '.claude/skills/_learnings/pending.md'
    if (-not (Test-Path $pending)) { exit 0 }

    # ファイルがロック等で読めないときはゲートを素通し（フェイルオープン）
    try {
        $entries = @(Select-String -Path $pending -Pattern '^##\s*\[\d{4}-\d{2}-\d{2}\]' -Encoding utf8)
    } catch { exit 0 }
    if ($entries.Count -eq 0) { exit 0 }

    $reason = "未処理の学びが $($entries.Count) 件 .claude/skills/_learnings/pending.md にあります。" +
              "終了する前に skill-evolve スキル（.claude/skills/skill-evolve/SKILL.md）の手順どおり各スキルへ反映し、" +
              "処理済みエントリを archive.md へ移動して pending.md を空にしてください。" +
              "反映する価値がないと判断したエントリも、その旨を付記して archive.md へ移動すること。"

    @{ decision = 'block'; reason = $reason } | ConvertTo-Json -Compress
} catch {
    # 想定外の失敗ではブロックしない
}
exit 0
