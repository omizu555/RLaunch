//! launcher-data.json の読み書き・バックアップ。
//! 保存先は旧 Tauri 版と同一の %APPDATA%/com.rlaunch.app/（既存ユーザーデータをそのまま使う）。

use super::data::{LauncherData, Tab};
use std::fs;
use std::path::PathBuf;

pub const DATA_FILE: &str = "launcher-data.json";

/// %APPDATA%/com.rlaunch.app/
pub fn data_dir() -> PathBuf {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    appdata.join("com.rlaunch.app")
}

pub fn data_file_path() -> PathBuf {
    data_dir().join(DATA_FILE)
}

pub fn themes_dir() -> PathBuf {
    data_dir().join("themes")
}

pub fn icons_dir() -> PathBuf {
    data_dir().join("icons")
}

/// ロード結果
pub struct LoadOutcome {
    pub data: LauncherData,
    pub warning: Option<String>,
    /// 読み取り自体に失敗（ロック等）した場合 true。
    /// この状態で save すると正常なファイルをデフォルトで上書きし全消失するため、
    /// 呼び出し側は save を禁止すること。
    pub save_disabled: bool,
    /// ロード時点のファイル更新時刻（外部変更検出用）
    pub mtime: Option<std::time::SystemTime>,
}

fn file_mtime(path: &std::path::Path) -> Option<std::time::SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// 起動時ロード。ファイル破損時は退避してデフォルトで開始（上書き喪失を防ぐ）。
/// 読み取り不能（ロック等）時は save_disabled=true を返す。
pub fn load() -> LoadOutcome {
    let path = data_file_path();
    if !path.exists() {
        return LoadOutcome {
            data: default_data(),
            warning: None,
            save_disabled: false,
            mtime: None,
        };
    }
    match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<LauncherData>(&text) {
            Ok(mut data) => {
                normalize(&mut data);
                // 正常にパースできたファイルのみバックアップローテーションする
                // （破損ファイルで正常世代を押し出さないため）
                rotate_backups();
                LoadOutcome {
                    data,
                    warning: None,
                    save_disabled: false,
                    mtime: file_mtime(&path),
                }
            }
            Err(e) => {
                // 破損ファイルを退避してから初期データで開始
                let broken = data_dir().join(format!(
                    "launcher-data.broken-{}.json",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                ));
                let _ = fs::copy(&path, &broken);
                LoadOutcome {
                    data: default_data(),
                    warning: Some(format!(
                        "データファイルの読み込みに失敗したため {} に退避しました: {}",
                        broken.display(),
                        e
                    )),
                    save_disabled: false,
                    mtime: file_mtime(&path),
                }
            }
        },
        Err(e) => LoadOutcome {
            data: default_data(),
            warning: Some(format!(
                "データファイルを読めませんでした（保存を無効化しています。再起動してください）: {}",
                e
            )),
            save_disabled: true,
            mtime: None,
        },
    }
}

/// 保存（テンポラリ書き込み→リネームで原子的に）。
/// expected_mtime と実ファイルの更新時刻が異なる場合（旧版アプリ等の外部プロセスが
/// 書き換えた場合）は、上書き前に conflict ファイルへ退避して警告を返す。
/// 戻り値: (新しい mtime, 競合警告)
pub fn save(
    data: &LauncherData,
    expected_mtime: Option<std::time::SystemTime>,
) -> Result<(std::time::SystemTime, Option<String>), String> {
    let dir = data_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("データフォルダを作成できません: {}", e))?;
    let path = data_file_path();

    // 外部変更検出（後勝ち上書きによるサイレントなデータ喪失を防ぐ）
    let mut conflict_note = None;
    if let (Some(expected), Some(actual)) = (expected_mtime, file_mtime(&path)) {
        if actual != expected {
            let conflict = dir.join(format!(
                "launcher-data.conflict-{}.json",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            ));
            let _ = fs::copy(&path, &conflict);
            conflict_note = Some(format!(
                "他のプロセス（旧版RLaunch等）がデータを変更していたため {} に退避してから上書きしました",
                conflict.display()
            ));
        }
    }

    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("シリアライズ失敗: {}", e))?;
    let tmp = dir.join(format!("{}.tmp", DATA_FILE));
    fs::write(&tmp, json).map_err(|e| format!("一時ファイル書き込み失敗: {}", e))?;
    fs::rename(&tmp, &path).map_err(|e| format!("保存失敗: {}", e))?;
    let mtime = file_mtime(&path).unwrap_or_else(std::time::SystemTime::now);
    Ok((mtime, conflict_note))
}

/// 起動時バックアップ（旧版互換: launcher-data.bak.{1,2,3}.json の3世代ローテーション）
pub fn rotate_backups() {
    let dir = data_dir();
    let current = data_file_path();
    if !current.exists() {
        return;
    }
    let bak = |n: u32| dir.join(format!("launcher-data.bak.{}.json", n));
    let _ = fs::remove_file(bak(3));
    if bak(2).exists() {
        let _ = fs::rename(bak(2), bak(3));
    }
    if bak(1).exists() {
        let _ = fs::rename(bak(1), bak(2));
    }
    let _ = fs::copy(&current, bak(1));
}

/// フォント名だけを軽量に読む（daemon 起動前に default_font を決めるため。
/// 本ロードは boot 側で改めて行う）
pub fn peek_font_family() -> Option<String> {
    let text = fs::read_to_string(data_file_path()).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("settings")?
        .get("fontFamily")?
        .as_str()
        .map(|s| s.to_string())
}

/// エクスポート（保存ダイアログ等から任意パスへ）
pub fn export_to(data: &LauncherData, path: &std::path::Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// インポート結果
pub enum ImportMode {
    /// 全置換
    Replace,
    /// 既存タブ保持 + 新規タブ追加、設定は上書き
    Merge,
}

pub fn import_from(
    current: &mut LauncherData,
    path: &std::path::Path,
    mode: ImportMode,
) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut incoming: LauncherData = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    normalize(&mut incoming);
    match mode {
        ImportMode::Replace => *current = incoming,
        ImportMode::Merge => {
            current.settings = incoming.settings;
            let existing: std::collections::HashSet<String> =
                current.tabs.iter().map(|t| t.id.clone()).collect();
            for tab in incoming.tabs {
                if !existing.contains(&tab.id) {
                    current.tabs.push(tab);
                }
            }
            renumber_tabs(current);
        }
    }
    Ok(())
}

fn default_data() -> LauncherData {
    let mut data = LauncherData::default();
    data.tabs.push(Tab::new("メイン", 8, 4));
    data
}

/// ロード直後の整合性回復（タブ0個、items 長不一致など）
pub fn normalize(data: &mut LauncherData) {
    if data.tabs.is_empty() {
        data.tabs.push(Tab::new(
            "メイン",
            data.settings.default_grid_columns,
            data.settings.default_grid_rows,
        ));
    }
    data.tabs.sort_by_key(|t| t.order);
    for tab in &mut data.tabs {
        tab.normalize();
        for cell in tab.items.iter_mut().flatten() {
            if let super::data::GridCell::Group(g) = cell {
                g.normalize();
            }
        }
    }
    renumber_tabs(data);
    data.settings.cell_size = data.settings.cell_size.clamp(40, 120);
    data.settings.list_columns = data.settings.list_columns.clamp(1, 4);
}

pub fn renumber_tabs(data: &mut LauncherData) {
    for (i, tab) in data.tabs.iter_mut().enumerate() {
        tab.order = i as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_fixes_empty_and_lengths() {
        let mut data = LauncherData::default();
        normalize(&mut data);
        assert_eq!(data.tabs.len(), 1);
        assert_eq!(data.tabs[0].items.len(), 8 * 4);
    }
}
