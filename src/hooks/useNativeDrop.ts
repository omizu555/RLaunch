/* ============================================================
   useNativeDrop - 子ウィンドウ用ネイティブ D&D フック
   OS からのファイルドラッグ＆ドロップを検知し、
   ドロップ位置のセルインデックスを特定する。
   メインウィンドウ用 useDragDrop との違い:
   - 自動非表示 (useAutoHide) 連携なし
   - よりシンプルなインターフェース
   ============================================================ */
import { useState, useEffect } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCellIndexFromPosition } from "../utils/domHelpers";
import { useStableRef } from "./useStableRef";

interface Position {
  x: number;
  y: number;
}

/**
 * 子ウィンドウ用のネイティブ D&D フック
 * @param onDrop ファイルドロップ時のコールバック (filePaths, cellIndex)
 * @returns hoverIndex - 現在ドラッグホバー中のセルインデックス
 */
export function useNativeDrop(
  onDrop: (filePaths: string[], cellIndex: number) => void,
) {
  const [hoverIndex, setHoverIndex] = useState<number | null>(null);
  const onDropRef = useStableRef(onDrop);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      const win = getCurrentWebviewWindow();
      unlisten = await win.onDragDropEvent((event) => {
        const payload = event.payload;

        if (payload.type === "over" || payload.type === "enter") {
          const pos = "position" in payload
            ? (payload as unknown as { position: Position }).position
            : null;
          setHoverIndex(pos ? getCellIndexFromPosition(pos) : null);
        } else if (payload.type === "drop") {
          setHoverIndex(null);
          const dropPayload = payload as unknown as { paths: string[]; position: Position };
          const cellIndex = getCellIndexFromPosition(dropPayload.position);
          if (cellIndex !== null && dropPayload.paths.length > 0) {
            onDropRef.current(dropPayload.paths, cellIndex);
          }
        } else if (payload.type === "leave") {
          setHoverIndex(null);
        }
      });
    })();

    return () => { unlisten?.(); };
  }, []);

  return hoverIndex;
}
