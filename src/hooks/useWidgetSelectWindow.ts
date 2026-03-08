/* ============================================================
   useWidgetSelectWindow - ウィジェット選択ウィンドウ管理（useChildWindow ベース）
   ============================================================ */
import { useCallback, useMemo, useRef } from "react";
import type { AppSettings, WidgetType } from "../types";
import { useChildWindow } from "./useChildWindow";
import { useStableRef } from "./useStableRef";

interface WidgetSelectPayload {
  index: number;
  themeId?: string;
}

interface UseWidgetSelectWindowOptions {
  settings: AppSettings;
  onSelect: (widgetType: WidgetType, index: number) => void;
}

export function useWidgetSelectWindow({ settings, onSelect }: UseWidgetSelectWindowOptions) {
  const pendingPayload = useRef<WidgetSelectPayload | null>(null);
  const settingsRef = useStableRef(settings);
  const onSelectRef = useStableRef(onSelect);

  const events = useMemo(() => ({
    readyEvent: "widget-select-ready",
    initEvent: "widget-select-init",
    resultEvent: "widget-select-result",
    closedEvent: "widget-select-closed",

    getInitPayload: () => pendingPayload.current,

    onResult: (payload: { widgetId: string; index: number }) => {
      onSelectRef.current(payload.widgetId, payload.index);
    },
  }), []);

  const config = useMemo(() => ({
    label: "widget-select",
    url: "src/widget-select.html",
    title: "ウィジェット選択",
    width: 400,
    height: 520,
  }), []);

  const { openWindow } = useChildWindow<WidgetSelectPayload, { widgetId: string; index: number }>(config, events);

  const openWidgetSelectWindow = useCallback(async (index: number) => {
    pendingPayload.current = { index, themeId: settingsRef.current.theme };
    await openWindow();
  }, [openWindow]);

  return { openWidgetSelectWindow };
}
