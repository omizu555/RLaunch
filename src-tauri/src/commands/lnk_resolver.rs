use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::process::Command;

#[derive(Serialize)]
pub struct LnkInfo {
    pub target_path: String,
    pub working_dir: String,
    pub arguments: String,
    pub icon_location: String,
}

/// .lnk ショートカットファイルのリンク先パスを解決する
/// PowerShell の WScript.Shell COM を使って確実に解決
#[tauri::command]
pub async fn resolve_lnk(path: String) -> Result<LnkInfo, String> {
    let escaped = path.replace('\'', "''");
    let script = format!(
        r#"$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');@{{t=$s.TargetPath;w=$s.WorkingDirectory;a=$s.Arguments;i=$s.IconLocation}}|ConvertTo-Json -Compress"#,
        escaped
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("PowerShell error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "LNK resolution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(json_str.trim())
        .map_err(|e| format!("JSON parse error: {}", e))?;

    Ok(LnkInfo {
        target_path: v["t"].as_str().unwrap_or("").to_string(),
        working_dir: v["w"].as_str().unwrap_or("").to_string(),
        arguments: v["a"].as_str().unwrap_or("").to_string(),
        icon_location: v["i"].as_str().unwrap_or("").to_string(),
    })
}
