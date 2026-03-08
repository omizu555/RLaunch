/* ============================================================
   useWidgetSettingsWindow - ウィジェット設定ウィンドウ管理（useChildWindow ベース）
   ============================================================ */
import { useCallback, useMemo, useRef } from "react";
import type { WidgetItem, AppSettings } from "../types";
import { useChildWindow } from "./useChildWindow";
import { useStableRef } from "./useStableRef";

interface WidgetSettingsPayload {
  widget: WidgetItem;
  index: number;
  themeId?: string;
}

interface UseWidgetSettingsWindowOptions {
  settings: AppSettings;
  onSave: (widget: WidgetItem, index: number) => void;
}

export function useWidgetSettingsWindow({ settings, onSave }: UseWidgetSettingsWindowOptions) {
  const pendingPayload = useRef<WidgetSettingsPayload | null>(null);
  const settingsRef = useStableRef(settings);
  const onSaveRef = useStableRef(onSave);

  const events = useMemo(() => ({
    readyEvent: "widget-settings-ready",
    initEvent: "widget-settings-init",
    resultEvent: "widget-settings-save",
    closedEvent: "widget-settings-closed",

    getInitPayload: () => pendingPayload.current,

    onResult: (payload: { widget: WidgetItem; index: number }) => {
      onSaveRef.current(payload.widget, payload.index);
    },
  }), []);

  const config = useMemo(() => ({
    label: "widget-settings",
    url: "src/widget-settings.html",
    title: "ウィジェット設定",
    width: 380,
    height: 480,
  }), []);

  const { openWindow } = useChildWindow<WidgetSettingsPayload, { widget: WidgetItem; index: number }>(config, events);

  const openWidgetSettingsWindow = useCallback(async (widget: WidgetItem, index: number) => {
    pendingPayload.current = { widget, index, themeId: settingsRef.current.theme };
    await openWindow();
  }, [openWindow]);

  return { openWidgetSettingsWindow };
}
