use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FileInfo {
    pub file_name: String,
    pub file_type: String,
    pub extension: String,
    pub full_path: String,
    pub exists: bool,
    pub is_dir: bool,
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, String> {
    let p = Path::new(&path);
    let exists = p.exists();
    let is_dir = p.is_dir();
    let file_name = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            p.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });
    let extension = p
        .extension()
        .map(|s| s.to_string_lossy().to_string().to_lowercase())
        .unwrap_or_default();

    let file_type = if is_dir {
        "folder"
    } else {
        match extension.as_str() {
            "exe" | "msi" => "executable",
            "lnk" => "shortcut",
            "url" => "url",
            _ => "document",
        }
    };

    let full_path = p
        .canonicalize()
        .map(|cp| cp.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.clone());
    // Remove \\?\ prefix from Windows extended-length paths
    let full_path = full_path
        .strip_prefix(r"\\?\")
        .unwrap_or(&full_path)
        .to_string();

    Ok(FileInfo {
        file_name,
        file_type: file_type.to_string(),
        extension,
        full_path,
        exists,
        is_dir,
    })
}
