/* ============================================================
   useHotkey - グローバルショートカット管理
   ============================================================ */
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { useStableRef } from "./useStableRef";

type WindowPosition = "center" | "cursor" | "remember";

export function useHotkey(
  hotkey: string,
  windowPosition: WindowPosition = "cursor",
  pinned: boolean = false
) {
  const pinnedRef = useStableRef(pinned);
  const positionRef = useStableRef(windowPosition);

  useEffect(() => {
    let mounted = true;

    async function setup() {
      try {
        const appWindow = getCurrentWebviewWindow();
        await register(hotkey, async () => {
          if (!mounted) return;
          const visible = await appWindow.isVisible();
          if (visible) {
            // ピン留め中は非表示にしない
            if (pinnedRef.current) return;
            await appWindow.hide();
          } else {
            // 表示位置の設定に応じてウィンドウを配置
            await positionWindow(appWindow, positionRef.current);
            await appWindow.show();
            await appWindow.setFocus();
          }
        });
      } catch (e) {
        console.warn("Failed to register hotkey:", hotkey, e);
      }
    }

    setup();

    return () => {
      mounted = false;
      unregister(hotkey).catch((e) => {
        console.warn("Failed to unregister hotkey:", hotkey, e);
      });
    };
  }, [hotkey]);
}

/** ウィンドウの表示位置をモードに応じて設定（マルチモニター対応） */
async function positionWindow(
  appWindow: Awaited<ReturnType<typeof getCurrentWebviewWindow>>,
  mode: WindowPosition
) {
  try {
    if (mode === "center") {
      await appWindow.center();
    } else if (mode === "cursor") {
      // カーソル位置 + モニター作業領域を取得
      const info = await invoke<{
        cursor_x: number; cursor_y: number;
        monitor_x: number; monitor_y: number;
        monitor_w: number; monitor_h: number;
      }>("get_cursor_monitor_info");
      const size = await appWindow.outerSize();
      // カーソル中心に配置し、モニター作業領域内にクランプ
      let x = info.cursor_x - Math.floor(size.width / 2);
      let y = info.cursor_y - Math.floor(size.height / 2);
      const monRight = info.monitor_x + info.monitor_w;
      const monBottom = info.monitor_y + info.monitor_h;
      if (x + size.width > monRight) x = monRight - size.width;
      if (y + size.height > monBottom) y = monBottom - size.height;
      if (x < info.monitor_x) x = info.monitor_x;
      if (y < info.monitor_y) y = info.monitor_y;
      await appWindow.setPosition(new PhysicalPosition(x, y));
    }
    // "remember" の場合は何もしない（前回位置のまま）
  } catch (e) {
    console.warn("Failed to position window:", e);
  }
}
