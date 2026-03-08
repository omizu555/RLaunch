/* ============================================================
   useLauncher - アプリ起動ロジックの統合フック
   重複していた3つの起動関数を1つに統合
   ============================================================ */
import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GridCell, LauncherItem } from "../types";

interface UseLauncherOptions {
  hideOnLaunch: boolean;
  pinned: boolean;
  onNotify: (msg: string) => void;
  /** 起動回数を永続化するためのコールバック (index, updatedItem) */
  onItemLaunched?: (index: number, item: LauncherItem) => void;
}

/** 起動統計を更新したアイテムのコピーを返す */
function withLaunchStats(item: LauncherItem): LauncherItem {
  return {
    ...item,
    launchCount: (item.launchCount ?? 0) + 1,
    lastLaunchedAt: new Date().toISOString(),
  };
}

export function useLauncher({ hideOnLaunch, pinned, onNotify, onItemLaunched }: UseLauncherOptions) {
  /** 共通: ウィンドウ非表示処理 (ピン留め中はスキップ) */
  const hideWindow = useCallback(async () => {
    if (!hideOnLaunch || pinned) return;
    const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    getCurrentWebviewWindow().hide();
  }, [hideOnLaunch, pinned]);

  /** 通常起動 (GridCell / LauncherItem 両対応) */
  const launch = useCallback(
    async (cell: GridCell | LauncherItem) => {
      if (!cell || cell.type === "widget" || cell.type === "group") return;
      const item = cell as LauncherItem;
      try {
        await invoke("launch_app", {
          path: item.path,
          args: item.args ?? null,
        });
        await hideWindow();
      } catch (e) {
        console.error("Failed to launch:", e);
        onNotify("起動に失敗しました");
      }
    },
    [hideWindow, onNotify]
  );

  /** 管理者として起動 */
  const launchAdmin = useCallback(
    async (cell: GridCell | LauncherItem) => {
      if (!cell || cell.type === "widget" || cell.type === "group") return;
      const item = cell as LauncherItem;
      try {
        await invoke("run_as_admin", {
          path: item.path,
          args: item.args ?? null,
        });
        onNotify("管理者として起動しました");
        await hideWindow();
      } catch (e) {
        console.error("Failed to launch as admin:", e);
        onNotify("管理者として起動に失敗しました");
      }
    },
    [hideWindow, onNotify]
  );

  /** ファイルの場所を開く */
  const openLocation = useCallback(
    async (cell: GridCell | LauncherItem) => {
      if (!cell || cell.type === "widget" || cell.type === "group") return;
      const item = cell as LauncherItem;
      try {
        await invoke("open_file_location", { path: item.path });
      } catch (e) {
        console.error("Failed to open file location:", e);
        onNotify("ファイルの場所を開けませんでした");
      }
    },
    [onNotify]
  );

  /** セルクリック起動 (handleCellClick の統合版) — 起動統計も記録 */
  const launchFromCell = useCallback(
    async (index: number, cell: GridCell) => {
      if (!cell || cell.type === "widget" || cell.type === "group") return;
      const item = cell as LauncherItem;
      await launch(item);
      // 起動統計をコールバックで永続化
      onItemLaunched?.(index, withLaunchStats(item));
    },
    [launch, onItemLaunched]
  );

  return { launch, launchAdmin, openLocation, launchFromCell };
}
