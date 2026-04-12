use std::fs;
use std::path::Path;

fn main() {
    // サンプルテーマリソースをビルド出力にコピー
    let src = Path::new("resources/sample-themes");
    if src.exists() {
        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
        let dst = Path::new("target").join(&profile).join("resources").join("sample-themes");
        fs::create_dir_all(&dst).ok();
        if let Ok(entries) = fs::read_dir(src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let dest = dst.join(entry.file_name());
                    fs::copy(&path, &dest).ok();
                }
            }
        }
        // リソースファイル変更時に再ビルドをトリガー
        println!("cargo:rerun-if-changed=resources/sample-themes");
    }

    tauri_build::build()
}
