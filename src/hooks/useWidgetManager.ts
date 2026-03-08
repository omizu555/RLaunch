/* ============================================================
   useWidgetManager - ウィジェット管理フック
   マニフェストからデフォルト設定を取得して配置する。
   ============================================================ */
import { useCallback } from "react";
import type { WidgetItem, WidgetType } from "../types";
import { getDefaultWidgetConfig, getDefaultUpdateInterval, WIDGET_LABELS, isBuiltinWidget } from "../types";
import { getWidgetManifest, buildDefaultConfig } from "../utils/widgetLoader";
import { setGridCell } from "../stores/launcherStore";
import type { Tab } from "../types";

interface UseWidgetManagerOptions {
  activeTabId: string;
  onTabsUpdate: (tabs: Tab[]) => void;
  onNotify: (msg: string) => void;
}

export function useWidgetManager({ activeTabId, onTabsUpdate, onNotify }: UseWidgetManagerOptions) {

  /** ウィジェットタイプ選択 → 配置（ウィンドウから widgetType と index が直接渡される） */
  const handleWidgetSelect = useCallback(
    async (widgetType: WidgetType, cellIndex: number) => {
      if (!activeTabId) return;

      // マニフェストからデフォルト設定を取得（ビルトインはフォールバック）
      const manifest = await getWidgetManifest(widgetType);
      let config: Record<string, unknown>;
      let updateInterval: number;
      let label: string;

      if (manifest) {
        config = buildDefaultConfig(manifest);
        updateInterval = manifest.updateInterval;
        label = manifest.label;
        // ビルトインの場合はコンパイル済みデフォルトとマージ（後方互換）
        if (isBuiltinWidget(widgetType)) {
          config = { ...getDefaultWidgetConfig(widgetType), ...config };
        }
      } else {
        // マニフェストが無い場合はビルトインフォールバック
        config = getDefaultWidgetConfig(widgetType) as unknown as Record<string, unknown>;
        updateInterval = getDefaultUpdateInterval(widgetType);
        label = WIDGET_LABELS[widgetType] ?? widgetType;
      }

      const widget: WidgetItem = {
        id: crypto.randomUUID(),
        type: "widget",
        widgetType,
        label,
        config,
        updateInterval,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      const newTabs = await setGridCell(activeTabId, cellIndex, widget);
      onTabsUpdate(newTabs);
      onNotify(`「${label}」を配置しました`);
    },
    [activeTabId, onTabsUpdate, onNotify]
  );

  /** ウィジェット設定保存（ウィジェット設定ウィンドウから呼ばれる） */
  const handleWidgetSettingsSave = useCallback(
    async (updated: WidgetItem, cellIndex: number) => {
      if (!activeTabId) return;
      const newTabs = await setGridCell(activeTabId, cellIndex, updated);
      onTabsUpdate(newTabs);
      onNotify("ウィジェット設定を保存しました");
    },
    [activeTabId, onTabsUpdate, onNotify]
  );

  return {
    handleWidgetSelect,
    handleWidgetSettingsSave,
  };
}
