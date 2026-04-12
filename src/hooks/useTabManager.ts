/* ============================================================
   useTabManager - タブ & グリッドセル管理フック
   ============================================================ */
import { useState, useCallback } from "react";
import type { Tab, AppSettings, GridCell } from "../types";
import {
  getTabs,
  addTab,
  renameTab,
  removeTab,
  setGridCell,
  clearGridCell,
  swapGridCells,
  insertGridCell,
  getSettings,
  saveSettings,
  resizeAllTabsGrid,
  reorderTabs,
  duplicateTab,
  updateTabSettings,
} from "../stores/launcherStore";
import { DEFAULT_SETTINGS } from "../types";
import { createLauncherItemFromPath } from "../utils/fileRegistration";

export function useTabManager(onNotify: (msg: string) => void) {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string>("");
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);

  /** 初期読み込み */
  const loadData = useCallback(async () => {
    try {
      const [loadedTabs, loadedSettings] = await Promise.all([
        getTabs(),
        getSettings(),
      ]);
      setTabs(loadedTabs);
      setActiveTabId(loadedTabs[0]?.id ?? "");
      setSettings(loadedSettings);
    } catch (e) {
      console.error("Failed to load data:", e);
      setTabs([
        {
          id: crypto.randomUUID(),
          label: "メイン",
          order: 0,
          gridColumns: 8,
          gridRows: 4,
          items: new Array(32).fill(null),
        },
      ]);
    } finally {
      setLoading(false);
    }
  }, []);

  /** アクティブタブ */
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? tabs[0];

  // ── タブ操作 ──

  const handleAddTab = useCallback(async () => {
    const newTabs = await addTab(
      `タブ ${tabs.length + 1}`,
      settings.defaultGridColumns,
      settings.defaultGridRows
    );
    setTabs(newTabs);
    setActiveTabId(newTabs[newTabs.length - 1].id);
  }, [tabs.length, settings.defaultGridColumns, settings.defaultGridRows]);

  const handleRenameTab = useCallback(
    async (tabId: string, newLabel: string) => {
      const newTabs = await renameTab(tabId, newLabel);
      setTabs(newTabs);
    },
    []
  );

  const handleRemoveTab = useCallback(
    async (tabId: string) => {
      // P-05: 削除されたタブの左隣に遷移（Chrome式）
      if (activeTabId === tabId) {
        const currentIdx = tabs.findIndex((t) => t.id === tabId);
        const fallbackTab = currentIdx > 0 ? tabs[currentIdx - 1] : tabs[currentIdx + 1];
        if (fallbackTab) {
          setActiveTabId(fallbackTab.id);
        }
      }
      const newTabs = await removeTab(tabId);
      setTabs(newTabs);
      if (!newTabs.find((t) => t.id === activeTabId)) {
        setActiveTabId(newTabs[0]?.id ?? "");
      }
    },
    [activeTabId, tabs]
  );

  // ── P-03: タブ並び替え ──
  const handleReorderTabs = useCallback(
    async (fromIndex: number, toIndex: number) => {
      const newTabs = await reorderTabs(fromIndex, toIndex);
      setTabs(newTabs);
    },
    []
  );

  // ── P-06: タブ複製 ──
  const handleDuplicateTab = useCallback(
    async (tabId: string) => {
      const newTabs = await duplicateTab(tabId);
      setTabs(newTabs);
      setActiveTabId(newTabs[newTabs.length - 1].id);
      onNotify("タブを複製しました");
    },
    [onNotify]
  );

  // ── タブ設定の一括更新 ──
  const handleTabSettings = useCallback(
    async (tabId: string, settings: {
      label?: string;
      gridColumns?: number;
      gridRows?: number;
      color?: string;
      viewMode?: "grid" | "list";
      listColumns?: number;
    }) => {
      const newTabs = await updateTabSettings(tabId, settings);
      setTabs(newTabs);
      onNotify("タブ設定を更新しました");
    },
    [onNotify]
  );

  // ── グリッドセル操作 ──

  const handleCellClear = useCallback(
    async (index: number) => {
      const newTabs = await clearGridCell(activeTabId, index);
      setTabs(newTabs);
      onNotify("登録を解除しました");
    },
    [activeTabId, onNotify]
  );

  /** セルのアイテムを更新（folderAction 切替等に使用） */
  const handleCellUpdate = useCallback(
    async (index: number, item: GridCell) => {
      const newTabs = await setGridCell(activeTabId, index, item);
      setTabs(newTabs);
    },
    [activeTabId]
  );

  const handleCellSwap = useCallback(
    async (fromIndex: number, toIndex: number) => {
      const newTabs = await swapGridCells(activeTabId, fromIndex, toIndex);
      setTabs(newTabs);
    },
    [activeTabId]
  );

  /** Tauri ネイティブ D&D でアイテム登録 (P-07: 複数ファイル一括対応) */
  const handleNativeDrop = useCallback(
    async (filePaths: string[], cellIndex: number | null) => {
      if (cellIndex === null || !activeTabId) return;

      const tab = tabs.find((t) => t.id === activeTabId);
      if (!tab) return;

      let registered = 0;
      let currentIndex = cellIndex;

      for (const filePath of filePaths) {
        if (currentIndex >= tab.items.length) break;

        try {
          const item = await createLauncherItemFromPath(filePath);
          if (!item) continue;

          const isOccupied = tab.items[currentIndex] != null;

          if (isOccupied) {
            await insertGridCell(activeTabId, currentIndex, item);
          } else {
            await setGridCell(activeTabId, currentIndex, item);
          }

          registered++;

          // 次の空きセルを探す（右方向）
          currentIndex++;
          while (currentIndex < tab.items.length && tab.items[currentIndex] != null) {
            currentIndex++;
          }
        } catch (e) {
          console.error("Registration failed:", e);
        }
      }

      // 最新のタブデータを再取得
      const newTabs = await getTabs();
      setTabs(newTabs);

      if (registered === 0) {
        onNotify("登録に失敗しました");
      } else if (registered < filePaths.length) {
        onNotify(`${registered}/${filePaths.length} アイテムを登録しました（空きスロット不足）`);
      } else {
        onNotify(
          registered === 1
            ? `アイテムを登録しました`
            : `${registered} アイテムを登録しました`
        );
      }
    },
    [activeTabId, tabs, onNotify]
  );

  // ── 設定操作 ──

  const handleSettingsChange = useCallback(
    async (newSettings: AppSettings) => {
      await saveSettings(newSettings);

      // グリッドサイズが変更されたら全タブに反映
      if (
        newSettings.defaultGridColumns !== settings.defaultGridColumns ||
        newSettings.defaultGridRows !== settings.defaultGridRows
      ) {
        const resized = await resizeAllTabsGrid(
          newSettings.defaultGridColumns,
          newSettings.defaultGridRows
        );
        setTabs(resized);
      }

      setSettings(newSettings);
    },
    [settings.defaultGridColumns, settings.defaultGridRows]
  );

  return {
    // state
    tabs,
    activeTabId,
    activeTab,
    settings,
    loading,
    // setters
    setActiveTabId,
    setTabs,
    // tab ops
    handleAddTab,
    handleRenameTab,
    handleRemoveTab,
    handleReorderTabs,
    handleDuplicateTab,
    handleTabSettings,
    // cell ops
    handleCellClear,
    handleCellUpdate,
    handleCellSwap,
    handleNativeDrop,
    // settings
    handleSettingsChange,
    // data loader
    loadData,
  };
}
