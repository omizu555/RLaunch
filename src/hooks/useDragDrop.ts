/* ============================================================
   useDragDrop - Tauri D&D イベントフック
   OS からのファイルドラッグ&ドロップを検知し、
   落下位置のグリッドセルインデックスを特定する
   ============================================================ */
import { useEffect } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { setDragDropActive } from "./useAutoHide";
import { getCellIndexFromPosition } from "../utils/domHelpers";
import { useStableRef } from "./useStableRef";

interface Position {
  x: number;
  y: number;
}

/**
 * Tauri のネイティブ D&D イベントをリスンするフック
 * @param onDrop ファイルがドロップされた時のコールバック (filePaths, cellIndex)
 * @param onHover ドラッグ中のホバー位置が変わった時のコールバック (cellIndex | null)
 */
export function useDragDrop(
  onDrop: (filePaths: string[], cellIndex: number | null) => void,
  onHover: (cellIndex: number | null) => void
) {
  const onDropRef = useStableRef(onDrop);
  const onHoverRef = useStableRef(onHover);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      const appWindow = getCurrentWebviewWindow();
      unlisten = await appWindow.onDragDropEvent((event) => {
        const payload = event.payload;

        if (payload.type === "enter") {
          // D&D開始 → 自動非表示を抑制
          setDragDropActive(true);
          const pos = "position" in payload ? (payload as unknown as { position: Position }).position : null;
          const cellIndex = pos ? getCellIndexFromPosition(pos) : null;
          onHoverRef.current(cellIndex);
        } else if (payload.type === "over") {
          const pos = "position" in payload ? (payload as unknown as { position: Position }).position : null;
          const cellIndex = pos ? getCellIndexFromPosition(pos) : null;
          onHoverRef.current(cellIndex);
        } else if (payload.type === "drop") {
          const dropPayload = payload as unknown as { paths: string[]; position: Position };
          const cellIndex = getCellIndexFromPosition(dropPayload.position);
          onDropRef.current(dropPayload.paths, cellIndex);
          onHoverRef.current(null);
          // D&D完了 → 自動非表示を再有効化
          setTimeout(() => setDragDropActive(false), 300);
        } else if (payload.type === "leave") {
          onHoverRef.current(null);
          // D&D離脱 → 自動非表示を再有効化
          setTimeout(() => setDragDropActive(false), 300);
        }
      });
    })();

    return () => {
      unlisten?.();
    };
  }, []);
}
