/* ============================================================
   useWindowSize - ウィンドウサイズの自動計算 & リサイズ
   cellSize × cols × rows からウィンドウサイズを算出
   ============================================================ */
import { useCallback, useEffect, useRef } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";
import type { AppSettings } from "../types";
import { LAYOUT } from "../constants";

/** セルサイズとグリッド構成からウィンドウサイズを計算 */
export function calcWindowSize(s: AppSettings) {
  const width =
    s.cellSize * s.defaultGridColumns +
    LAYOUT.GRID_GAP * (s.defaultGridColumns - 1) +
    LAYOUT.GRID_PADDING +
    LAYOUT.BORDER_EXTRA;
  const height =
    s.cellSize * s.defaultGridRows +
    LAYOUT.GRID_GAP * (s.defaultGridRows - 1) +
    LAYOUT.GRID_PADDING +
    LAYOUT.TITLEBAR_HEIGHT +
    LAYOUT.TABBAR_HEIGHT +
    LAYOUT.STATUSBAR_HEIGHT +
    LAYOUT.BORDER_EXTRA;
  return { width, height };
}

/** メインウィンドウを指定設定に合わせてリサイズ */
export async function resizeMainWindow(s: AppSettings) {
  try {
    const { width, height } = calcWindowSize(s);
    const appWindow = getCurrentWebviewWindow();
    await appWindow.setSize(new LogicalSize(width, height));
  } catch (e) {
    console.warn("Failed to resize window:", e);
  }
}

/**
 * loading 完了時に一度だけウィンドウサイズを設定に合わせるフック
 */
export function useWindowSize(settings: AppSettings, loading: boolean) {
  const initializedRef = useRef(false);

  useEffect(() => {
    if (!loading && !initializedRef.current) {
      initializedRef.current = true;
      resizeMainWindow(settings);
    }
  }, [loading, settings]);

  /** 設定変更後にリサイズを実行するコールバック */
  const resize = useCallback((s: AppSettings) => resizeMainWindow(s), []);

  return { resize };
}
