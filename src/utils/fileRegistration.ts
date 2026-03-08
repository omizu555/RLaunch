/* ============================================================
   fileRegistration.ts - ファイルパスからランチャーアイテムを生成
   OS D&D ドロップ時のファイル種別判定・LNK解決・アイコン抽出を行う。
   ============================================================ */
import { invoke } from "@tauri-apps/api/core";
import type { LauncherItem, ItemType } from "../types";

/** Rust get_file_info の戻り値型 */
interface FileInfo {
  file_name: string;
  file_type: string;
  extension: string;
  full_path: string;
  exists: boolean;
  is_dir: boolean;
}

/** Rust resolve_lnk の戻り値型 */
interface LnkInfo {
  target_path: string;
  working_dir: string;
  arguments: string;
  icon_location: string;
}

/**
 * ファイルパスから LauncherItem を生成する。
 * .lnk の場合はリンク先解決、アイコン抽出も行う。
 *
 * @returns 成功時は LauncherItem、ファイルが存在しない場合は null
 * @throws 登録処理中のエラー
 */
export async function createLauncherItemFromPath(filePath: string): Promise<LauncherItem | null> {
  const fileInfo = await invoke<FileInfo>("get_file_info", { path: filePath });

  if (!fileInfo.exists) {
    return null;
  }

  let effectivePath = fileInfo.full_path;
  let args: string | undefined;
  let workingDir: string | undefined;
  let itemType = fileInfo.file_type as ItemType;

  // .lnk ショートカットを解決
  if (fileInfo.file_type === "shortcut") {
    try {
      const lnkInfo = await invoke<LnkInfo>("resolve_lnk", { path: filePath });
      if (lnkInfo.target_path) {
        effectivePath = lnkInfo.target_path;
        args = lnkInfo.arguments || undefined;
        workingDir = lnkInfo.working_dir || undefined;
        const resolvedInfo = await invoke<FileInfo>("get_file_info", { path: effectivePath });
        itemType = resolvedInfo.file_type as ItemType;
      }
    } catch (e) {
      console.warn("LNK resolution failed, using original path:", e);
    }
  }

  // アイコン抽出
  let iconBase64: string | undefined;
  try {
    iconBase64 = await invoke<string>("extract_icon", { path: effectivePath });
  } catch {
    console.warn("Icon extraction failed, using default icon");
  }

  return {
    id: crypto.randomUUID(),
    label: fileInfo.file_name,
    path: effectivePath,
    args,
    workingDir,
    iconBase64,
    iconPath: filePath,
    type: itemType,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}
