/* ============================================================
   useWindowSize - ウィンドウサイズの自動計算 & リサイズ
   cellSize × cols × rows からウィンドウサイズを算出
   ============================================================ */
import { useCallback, useEffect, useRef } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";
import type { AppSettings, Tab } from "../types";
import { LAYOUT } from "../constants";

interface WindowSizeOptions {
  viewMode?: "grid" | "list";
  listColumns?: number;
  gridColumns?: number;
  gridRows?: number;
}

/** セルサイズとグリッド構成からウィンドウサイズを計算 */
export function calcWindowSize(s: AppSettings, tabOverrides?: WindowSizeOptions) {
  const viewMode = tabOverrides?.viewMode ?? s.viewMode ?? "grid";
  const gridCols = tabOverrides?.gridColumns ?? s.defaultGridColumns;
  const gridRows = tabOverrides?.gridRows ?? s.defaultGridRows;
  const listCols = tabOverrides?.listColumns ?? s.listColumns ?? 1;

  if (viewMode === "list") {
    const LIST_ROW_HEIGHT = 29; // 28px row + 1px gap
    const totalCells = gridRows * gridCols;
    const maxVisibleRows = Math.min(Math.ceil(totalCells / listCols), 20);
    const width =
      s.cellSize * gridCols +
      LAYOUT.GRID_GAP * (gridCols - 1) +
      LAYOUT.GRID_PADDING +
      LAYOUT.BORDER_EXTRA;
    const height =
      LIST_ROW_HEIGHT * maxVisibleRows +
      8 + // list padding
      LAYOUT.TITLEBAR_HEIGHT +
      LAYOUT.TABBAR_HEIGHT +
      LAYOUT.STATUSBAR_HEIGHT +
      LAYOUT.BORDER_EXTRA;
    return { width, height };
  }

  const width =
    s.cellSize * gridCols +
    LAYOUT.GRID_GAP * (gridCols - 1) +
    LAYOUT.GRID_PADDING +
    LAYOUT.BORDER_EXTRA;
  const height =
    s.cellSize * gridRows +
    LAYOUT.GRID_GAP * (gridRows - 1) +
    LAYOUT.GRID_PADDING +
    LAYOUT.TITLEBAR_HEIGHT +
    LAYOUT.TABBAR_HEIGHT +
    LAYOUT.STATUSBAR_HEIGHT +
    LAYOUT.BORDER_EXTRA;
  return { width, height };
}

/** メインウィンドウを指定設定に合わせてリサイズ */
export async function resizeMainWindow(s: AppSettings, activeTab?: Tab) {
  try {
    const tabOverrides: WindowSizeOptions | undefined = activeTab
      ? {
          viewMode: activeTab.viewMode,
          listColumns: activeTab.listColumns,
          gridColumns: activeTab.gridColumns,
          gridRows: activeTab.gridRows,
        }
      : undefined;
    const { width, height } = calcWindowSize(s, tabOverrides);
    const appWindow = getCurrentWebviewWindow();
    await appWindow.setSize(new LogicalSize(width, height));
  } catch (e) {
    console.warn("Failed to resize window:", e);
  }
}

/**
 * loading 完了時に一度だけウィンドウサイズを設定に合わせるフック
 */
export function useWindowSize(settings: AppSettings, activeTab: Tab | undefined, loading: boolean) {
  const initializedRef = useRef(false);

  useEffect(() => {
    if (!loading && !initializedRef.current) {
      initializedRef.current = true;
      resizeMainWindow(settings, activeTab);
    }
  }, [loading, settings, activeTab]);

  /** 設定変更後にリサイズを実行するコールバック */
  const resize = useCallback(
    (s: AppSettings) => resizeMainWindow(s, activeTab),
    [activeTab]
  );

  return { resize };
}
