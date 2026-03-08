/* ============================================================
   positionWindowAtCursor - マルチモニター対応カーソル位置配置
   カーソルがあるモニターの作業領域内にウィンドウを配置する。

   useGroupPopupWindow / useFolderBrowserWindow で共通利用。
   ============================================================ */
import { invoke } from "@tauri-apps/api/core";
import type { OpenWindowOverrides } from "../hooks/useChildWindow";

/**
 * カーソル位置を基準にウィンドウを配置する OpenWindowOverrides を返す。
 * モニター作業領域内にクランプし、はみ出し防止する。
 * 取得失敗時は center: true にフォールバック。
 */
export async function positionWindowAtCursor(
  width: number,
  height: number,
): Promise<OpenWindowOverrides> {
  try {
    const info = await invoke<{
      cursor_x: number; cursor_y: number;
      monitor_x: number; monitor_y: number;
      monitor_w: number; monitor_h: number;
    }>("get_cursor_monitor_info");

    const scale = window.devicePixelRatio || 1;
    const cursorX = info.cursor_x / scale;
    const cursorY = info.cursor_y / scale;
    const monX = info.monitor_x / scale;
    const monY = info.monitor_y / scale;
    const monW = info.monitor_w / scale;
    const monH = info.monitor_h / scale;

    const x = Math.max(monX, Math.min(cursorX, monX + monW - width));
    const y = Math.max(monY, Math.min(cursorY, monY + monH - height));

    return { width, height, x: Math.round(x), y: Math.round(y), center: false };
  } catch {
    return { width, height, center: true };
  }
}
