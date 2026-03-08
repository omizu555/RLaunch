/* ============================================================
   useSettingsWindow - 設定ウィンドウ管理（useChildWindow ベース）
   ============================================================ */
import { useCallback, useMemo } from "react";
import type { AppSettings } from "../types";
import { useChildWindow } from "./useChildWindow";
import { useStableRef } from "./useStableRef";

interface UseSettingsWindowOptions {
  settings: AppSettings;
  onSettingsSave: (newSettings: AppSettings) => Promise<void>;
  onSettingsPreview?: (newSettings: AppSettings) => void;
  onImport?: (data: unknown, mode: "overwrite" | "merge") => Promise<void>;
}

export function useSettingsWindow({ settings, onSettingsSave, onSettingsPreview, onImport }: UseSettingsWindowOptions) {
  // コールバックを ref で保持（イベントコールバック内で最新値を参照するため）
  const settingsRef = useStableRef(settings);
  const onSettingsSaveRef = useStableRef(onSettingsSave);
  const onSettingsPreviewRef = useStableRef(onSettingsPreview);
  const onImportRef = useStableRef(onImport);

  const events = useMemo(() => ({
    readyEvent: "settings-ready",
    initEvent: "settings-init",
    resultEvent: "settings-save",
    closedEvent: "settings-closed",

    getInitPayload: () => settingsRef.current,

    onResult: (payload: AppSettings) => {
      onSettingsSaveRef.current(payload);
    },

    onClosed: () => {
      // プレビュー状態をリセット（元の settings に戻す）
      onSettingsPreviewRef.current?.(settingsRef.current);
    },

    extraListeners: [
      {
        event: "settings-preview",
        handler: (payload: unknown) => {
          onSettingsPreviewRef.current?.(payload as AppSettings);
        },
      },
      {
        event: "settings-import",
        handler: (payload: unknown) => {
          const p = payload as { data: unknown; mode: "overwrite" | "merge" };
          onImportRef.current?.(p.data, p.mode);
        },
      },
    ],
  }), []);

  const config = useMemo(() => ({
    label: "settings",
    url: "src/settings.html",
    title: "RLaunch - 設定",
    width: 480,
    height: 640,
  }), []);

  const { openWindow } = useChildWindow<AppSettings, AppSettings>(config, events);

  const openSettingsWindow = useCallback(async () => {
    await openWindow();
  }, [openWindow]);

  return { openSettingsWindow };
}
