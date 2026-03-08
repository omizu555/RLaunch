/* ============================================================
   useAutoHide - フォーカス喪失時の自動非表示
   D&D中やピン留め中は自動非表示を抑制する
   ============================================================ */
import { useEffect } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

// D&Dアクティブ中かどうかのグローバルフラグ
// （useDragDrop から更新される）
export let isDragDropActive = false;
export function setDragDropActive(active: boolean) {
  isDragDropActive = active;
}

// 設定ウィンドウが開いているかどうかのグローバルフラグ
// （useSettingsWindow から更新される）
export let isSettingsWindowOpen = false;
export function setSettingsWindowOpen(open: boolean) {
  isSettingsWindowOpen = open;
}

// ウィンドウドラッグ中かどうかのグローバルフラグ
// （CustomTitleBar から更新される）
export let isWindowDragging = false;
let dragResetTimer: ReturnType<typeof setTimeout> | null = null;
export function setWindowDragging(active: boolean) {
  if (dragResetTimer) {
    clearTimeout(dragResetTimer);
    dragResetTimer = null;
  }
  isWindowDragging = active;
  if (active) {
    // 安全弁: 2秒後に自動リセット（mouseup を取りこぼした場合）
    dragResetTimer = setTimeout(() => {
      isWindowDragging = false;
      dragResetTimer = null;
    }, 2000);
  }
}

// グループポップアップが開いているかどうかのグローバルフラグ
export let isGroupPopupOpen = false;
export function setGroupPopupOpen(open: boolean) {
  isGroupPopupOpen = open;
}

// フォルダブラウザが開いているかどうかのグローバルフラグ
export let isFolderBrowserOpen = false;
export function setFolderBrowserOpen(open: boolean) {
  isFolderBrowserOpen = open;
}

export function useAutoHide(enabled: boolean, pinned: boolean, onAutoHide?: () => void) {
  useEffect(() => {
    // ピン留め中、または自動非表示OFF → リスナー登録しない
    if (!enabled || pinned) return;

    const appWindow = getCurrentWebviewWindow();
    let cancelled = false;
    let unlisten: (() => void) | null = null;

    appWindow
      .onFocusChanged(({ payload: focused }) => {
        // クリーンアップ済みなら何もしない
        if (cancelled) return;
        if (!focused && !isDragDropActive && !isWindowDragging && !isSettingsWindowOpen && !isGroupPopupOpen && !isFolderBrowserOpen) {
          // 少し遅延させてD&Dやドラッグのフォーカス遷移を許容
          setTimeout(() => {
            if (!cancelled && !isDragDropActive && !isWindowDragging && !isSettingsWindowOpen && !isGroupPopupOpen && !isFolderBrowserOpen) {
              onAutoHide?.();
              appWindow.hide();
            }
          }, 300);
        }
      })
      .then((fn) => {
        if (cancelled) {
          // 既にクリーンアップ済み → 即座にunlisten
          fn();
        } else {
          unlisten = fn;
        }
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [enabled, pinned, onAutoHide]);
}
