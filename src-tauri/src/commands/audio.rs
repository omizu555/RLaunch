/* ============================================================
   audio - 音声ファイル関連コマンド
   ウィジェット (ポモドーロ等) の通知音で使用
   ============================================================ */
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

/// PowerShell の stdout バイト列をデコード
/// Windows では CP932 (Shift_JIS) で出力される場合があるため、
/// まず UTF-8 でパースし、失敗したら CP932 としてデコードする
fn decode_powershell_output(raw: &[u8]) -> String {
    // まず UTF-8 を試す
    if let Ok(s) = String::from_utf8(raw.to_vec()) {
        return s.trim().to_string();
    }
    // CP932 (Shift_JIS) としてデコード
    let mut result = String::new();
    let mut i = 0;
    while i < raw.len() {
        let b = raw[i];
        if b <= 0x7F {
            result.push(b as char);
            i += 1;
        } else if i + 1 < raw.len() {
            // 2バイト文字: PowerShell on Windows は OEM コードページで出力
            // cp932_to_char でユニコードに変換
            let lead = b;
            let trail = raw[i + 1];
            if let Some(ch) = cp932_to_unicode(lead, trail) {
                result.push(ch);
            } else {
                result.push('\u{FFFD}');
            }
            i += 2;
        } else {
            result.push('\u{FFFD}');
            i += 1;
        }
    }
    result.trim().to_string()
}

/// CP932 2バイト文字 → Unicode 変換
/// PowerShell stdout の日本語ファイル名をデコードするために使用
fn cp932_to_unicode(lead: u8, trail: u8) -> Option<char> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let bytes = [lead, trail];
    unsafe {
        let mut wbuf = [0u16; 2];
        let ret = windows::Win32::Globalization::MultiByteToWideChar(
            932,
            windows::Win32::Globalization::MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            &bytes,
            Some(&mut wbuf),
        );
        if ret > 0 {
            let os = OsString::from_wide(&wbuf[..ret as usize]);
            os.to_str().and_then(|s| s.chars().next())
        } else {
            None
        }
    }
}

/// ネイティブファイル選択ダイアログで音声ファイルを選ぶ
/// 選択されなかった場合は空文字列を返す
#[tauri::command]
pub async fn pick_sound_file() -> Result<String, String> {
    // -OutputEncoding UTF8 & [Console]::OutputEncoding で UTF-8 を強制
    let script = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.OpenFileDialog
$f.Title = '通知音ファイルを選択'
$f.Filter = '音声ファイル (*.mp3;*.wav;*.ogg;*.flac;*.m4a;*.aac;*.webm)|*.mp3;*.wav;*.ogg;*.flac;*.m4a;*.aac;*.webm|すべてのファイル (*.*)|*.*'
if ($f.ShowDialog() -eq 'OK') { [Console]::Out.Write($f.FileName) }
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("ダイアログ起動エラー: {}", e))?;

    let path = decode_powershell_output(&output.stdout);
    Ok(path)
}

/// 音声ファイルを読み込み `data:audio/...;base64,...` 形式の Data URL を返す
/// 対応形式: mp3, wav, ogg, flac, m4a, aac, wma, webm
#[tauri::command]
pub async fn read_sound_file(path: String) -> Result<String, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Err(format!("ファイルが見つかりません: {}", path));
    }

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime = match ext.as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "m4a" | "aac" => "audio/aac",
        "wma" => "audio/x-ms-wma",
        "webm" => "audio/webm",
        _ => return Err(format!("非対応の音声形式です: .{}", ext)),
    };

    let bytes = fs::read(file_path)
        .map_err(|e| format!("ファイル読み込みエラー: {}", e))?;

    // サイズ制限 (10MB)
    if bytes.len() > 10 * 1024 * 1024 {
        return Err("ファイルサイズが大きすぎます (上限: 10MB)".into());
    }

    let b64 = base64_encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// 簡易 Base64 エンコーダ (外部クレート不要)
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
