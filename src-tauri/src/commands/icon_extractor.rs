use std::os::windows::process::CommandExt;
use std::process::Command;

/// ファイルからアイコンを抽出して Base64 PNG として返す
/// .NET の System.Drawing.Icon.ExtractAssociatedIcon を使用
#[tauri::command]
pub async fn extract_icon(path: String) -> Result<String, String> {
    let escaped = path.replace('\'', "''");
    let script = format!(
        r#"Add-Type -AssemblyName System.Drawing;$i=[Drawing.Icon]::ExtractAssociatedIcon('{}');if($i){{$b=$i.ToBitmap();$m=[IO.MemoryStream]::new();$b.Save($m,[Drawing.Imaging.ImageFormat]::Png);[Convert]::ToBase64String($m.ToArray())}}"#,
        escaped
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("PowerShell error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Icon extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let base64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
    if base64.is_empty() {
        return Err("No icon found".into());
    }

    Ok(base64)
}
