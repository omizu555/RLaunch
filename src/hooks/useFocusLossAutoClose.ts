/* ============================================================
   useFocusLossAutoClose - フォーカス喪失時の自動クローズ（moveGuard 付き）
   ウィンドウドラッグ中のフォーカスロスを抑制しつつ、
   通常のフォーカスロスで closedEvent を emit する。

   GroupPopupWindow / FolderBrowserWindow で共通利用。
   ============================================================ */
import { useEffect, useRef } from "react";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

/**
 * ウィンドウのフォーカス喪失時に closedEventName を emit する。
 * ウィンドウドラッグ直後（600ms 以内）のフォーカスロスは無視する。
 */
export function useFocusLossAutoClose(closedEventName: string) {
  const moveGuard = useRef(0);

  useEffect(() => {
    const win = getCurrentWebviewWindow();
    let cancelled = false;
    let unlistenMove: (() => void) | null = null;
    let unlistenFocus: (() => void) | null = null;

    win.onMoved(() => {
      moveGuard.current = Date.now();
    }).then((fn) => {
      if (cancelled) fn(); else unlistenMove = fn;
    });

    win.onFocusChanged(({ payload: focused }) => {
      if (cancelled) return;
      if (!focused) {
        setTimeout(() => {
          if (!cancelled && Date.now() - moveGuard.current > 600) {
            emit(closedEventName);
          }
        }, 300);
      }
    }).then((fn) => {
      if (cancelled) fn(); else unlistenFocus = fn;
    });

    return () => {
      cancelled = true;
      unlistenMove?.();
      unlistenFocus?.();
    };
  }, [closedEventName]);
}
