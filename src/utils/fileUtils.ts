/* ============================================================
   fileUtils - ファイル・フォルダ関連ユーティリティ
   FolderBrowserWindow 等で使用する共通関数・型
   ============================================================ */

/** Rust の list_directory が返すエントリ */
export interface DirectoryEntry {
  name: string;
  path: string;
  is_dir: boolean;
  extension: string;
  size: number;
}

/** ファイルサイズを読みやすく整形 */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/** 拡張子に応じた絵文字アイコン */
export function getFileEmoji(ext: string): string {
  const e = ext.toLowerCase();
  if (["exe", "msi", "bat", "cmd", "ps1"].includes(e)) return "⚙";
  if (["lnk"].includes(e)) return "🔗";
  if (["txt", "log", "md", "csv", "json", "xml", "yaml", "yml", "toml", "ini", "cfg"].includes(e)) return "📝";
  if (["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico"].includes(e)) return "🖼";
  if (["mp3", "wav", "ogg", "flac", "m4a", "aac", "wma"].includes(e)) return "🎵";
  if (["mp4", "avi", "mkv", "mov", "wmv", "webm", "flv"].includes(e)) return "🎬";
  if (["zip", "rar", "7z", "tar", "gz", "bz2", "xz"].includes(e)) return "📦";
  if (["pdf"].includes(e)) return "📕";
  if (["doc", "docx", "xls", "xlsx", "ppt", "pptx"].includes(e)) return "📊";
  if (["html", "htm", "css", "js", "ts", "tsx", "jsx"].includes(e)) return "🌐";
  return "📄";
}

/** グリッドサイズから子ウィンドウの px サイズを計算 */
export function calcPopupSize(
  cols: number,
  rows: number,
  cellSize = 64,
  gap = 6,
) {
  const padding = 16;           // grid padding 8px × 2
  const headerFooter = 28 + 28; // header + footer
  const w = cols * cellSize + (cols - 1) * gap + padding + 20; // +20 余白
  const h = rows * cellSize + (rows - 1) * gap + padding + headerFooter + 16;
  return { w: Math.max(w, 200), h: Math.max(h, 160) };
}
