//! アイテム登録パイプライン（D&D / ダイアログ / URL）。
//! 旧版 src/utils/fileRegistration.ts の移植: 種別判定 → lnk解決 → アイコン抽出 → LauncherItem。
//!
//! lnk 解決（リンク切れで数秒ブロックし得る）とアイコン抽出は UI スレッドを固めるため、
//! `start_register` → バックグラウンド構築 → `Message::ItemsBuilt` → `finish_register`
//! の非同期2段構成にしている。

use crate::app::{App, GridRef, Message};
use crate::model::data::{now_iso8601, GridCell, LauncherItem};
use crate::platform::{icon, lnk};
use iced::Task;
use std::path::{Path, PathBuf};

/// ブロッキング処理を専用スレッドで実行して await 可能にする。
/// スレッドが panic した場合は None。
pub fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> impl std::future::Future<Output = Option<T>> + Send {
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    async move { rx.await.ok() }
}

/// 拡張子から種別判定（旧版と同ルール: exe/msi 系=executable, lnk=shortcut, url=url, dir=folder）
fn detect_type(path: &Path) -> &'static str {
    if path.is_dir() {
        return "folder";
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("exe") | Some("msi") | Some("bat") | Some("cmd") | Some("com") | Some("scr") => {
            "executable"
        }
        Some("lnk") => "shortcut",
        Some("url") => "url",
        _ => "document",
    }
}

/// パスから LauncherItem を構築（lnk 解決・アイコン抽出込み。ブロッキングなので
/// UI スレッドから直接呼ばず `blocking` 経由で使うこと）
pub fn build_item_from_path(path: &Path) -> LauncherItem {
    let now = now_iso8601();
    let item_type = detect_type(path);
    let label = path
        .file_stem()
        .or_else(|| path.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let mut target_path = path.to_string_lossy().into_owned();
    let mut args: Option<String> = None;
    let mut working_dir: Option<String> = None;

    if item_type == "shortcut" {
        if let Ok(info) = lnk::resolve_lnk(&target_path) {
            args = info.arguments;
            working_dir = info.working_dir;
            target_path = info.target_path;
        }
    }

    // アイコンは元のパス（lnk ならシェルが解決込みで返す）から抽出
    let icon_base64 = icon::extract_icon_png_base64(&path.to_string_lossy()).ok();

    LauncherItem {
        id: uuid::Uuid::new_v4().to_string(),
        label,
        path: target_path,
        args,
        working_dir,
        icon_base64,
        icon_path: None,
        library_icon: None,
        item_type: item_type.to_string(),
        run_as: None,
        window_state: None,
        hotkey: None,
        folder_action: None,
        launch_count: None,
        last_launched_at: None,
        created_at: now.clone(),
        updated_at: now,
        extra: Default::default(),
    }
}

/// URL 文字列から LauncherItem を構築
pub fn build_url_item(url: &str) -> LauncherItem {
    let now = now_iso8601();
    let label = url
        .trim()
        .strip_prefix("https://")
        .or_else(|| url.trim().strip_prefix("http://"))
        .unwrap_or(url.trim())
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string();
    LauncherItem {
        id: uuid::Uuid::new_v4().to_string(),
        label,
        path: url.trim().to_string(),
        args: None,
        working_dir: None,
        icon_base64: None,
        icon_path: None,
        library_icon: None,
        item_type: "url".to_string(),
        run_as: None,
        window_state: None,
        hotkey: None,
        folder_action: None,
        launch_count: None,
        last_launched_at: None,
        created_at: now.clone(),
        updated_at: now,
        extra: Default::default(),
    }
}

impl App {
    /// 登録開始（バックグラウンドで lnk 解決・アイコン抽出 → ItemsBuilt で配置）
    pub fn start_register(
        &mut self,
        grid: GridRef,
        idx: usize,
        paths: Vec<PathBuf>,
    ) -> Task<Message> {
        if paths.is_empty() {
            return Task::none();
        }
        if paths.len() > 2 {
            self.show_toast(format!("{} 件を登録しています…", paths.len()));
        }
        Task::perform(
            blocking(move || {
                paths
                    .iter()
                    .map(|p| build_item_from_path(p))
                    .collect::<Vec<_>>()
            }),
            move |items| match items {
                Some(items) => Message::ItemsBuilt(grid, idx, items),
                None => Message::Noop,
            },
        )
    }

    /// 構築済みアイテムの配置（旧版挙動: 先頭は占有セルなら挿入シフト、
    /// 以降は右方向の空きセルへ順次。溢れは通知）
    pub fn finish_register(&mut self, grid: GridRef, idx: usize, items: Vec<LauncherItem>) {
        let total = items.len();
        if total == 0 {
            return;
        }
        let mut registered = 0usize;
        let mut next_search = idx;

        for (i, item) in items.into_iter().enumerate() {
            let icon_b64 = item.icon_base64.clone();
            let id = item.id.clone();

            let placed = if i == 0 {
                self.place_item(grid, idx, item)
            } else {
                match self.find_empty_from(grid, next_search) {
                    Some(slot) => {
                        next_search = slot + 1;
                        self.place_item(grid, slot, item)
                    }
                    None => false,
                }
            };
            if placed {
                registered += 1;
                if let Some(b64) = icon_b64 {
                    self.cache_icon(&id, &b64);
                }
            }
        }

        self.save();
        self.recheck_invalid_paths();
        if registered == total {
            if total == 1 {
                self.show_toast("アイテムを登録しました");
            } else {
                self.show_toast(format!("{} アイテムを登録しました", registered));
            }
        } else {
            self.show_toast(format!(
                "{}/{} アイテムを登録しました（空きスロット不足）",
                registered, total
            ));
        }
    }

    /// 1アイテムを配置。空セル=そのまま、占有セル=挿入シフト（末尾溢れは消失=旧版仕様）。
    /// グリッド範囲外は false。
    pub fn place_item(&mut self, grid: GridRef, idx: usize, item: LauncherItem) -> bool {
        let Some(cells) = self.cells_mut(grid) else {
            return false;
        };
        if idx >= cells.len() {
            return false;
        }
        if cells[idx].is_none() {
            cells[idx] = Some(GridCell::Launcher(item));
            return true;
        }
        // 挿入シフト: idx から最初の空きセルまでを1つ後ろへずらす。空きが無ければ末尾が消える。
        let mut carry = Some(GridCell::Launcher(item));
        for slot in cells.iter_mut().skip(idx) {
            let prev = slot.take();
            *slot = carry;
            carry = prev;
            if carry.is_none() {
                break;
            }
        }
        true
    }

    /// idx 以降で最初の空きセル
    pub fn find_empty_from(&self, grid: GridRef, idx: usize) -> Option<usize> {
        let cells = self.cells(grid)?;
        (idx..cells.len()).find(|&i| cells[i].is_none())
    }

    /// URL をセルへ登録
    pub fn register_url(&mut self, grid: GridRef, idx: usize, url: &str) {
        if url.trim().is_empty() {
            return;
        }
        let item = build_url_item(url);
        if self.place_item(grid, idx, item) {
            self.save();
            self.show_toast("URL を登録しました");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_type_covers_known_extensions() {
        assert_eq!(detect_type(Path::new("C:\\a\\b.exe")), "executable");
        assert_eq!(detect_type(Path::new("C:\\a\\b.lnk")), "shortcut");
        assert_eq!(detect_type(Path::new("C:\\a\\b.url")), "url");
        assert_eq!(detect_type(Path::new("C:\\a\\b.txt")), "document");
        assert_eq!(detect_type(Path::new("C:\\Windows")), "folder");
    }

    #[test]
    fn build_url_item_extracts_hostname() {
        let item = build_url_item("https://www.example.com/path?q=1");
        assert_eq!(item.label, "www.example.com");
        assert_eq!(item.item_type, "url");
    }

    #[test]
    fn blocking_returns_value() {
        let result = futures::executor::block_on(blocking(|| 42));
        assert_eq!(result, Some(42));
    }
}
