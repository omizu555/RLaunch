/* ============================================================
   WidgetSelectWindow - 独立ウィジェット選択ウィンドウ
   Tauri emit/listen でメインウィンドウと通信
   ============================================================ */
import { useState, useEffect } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import type { WidgetManifest } from "../types";
import { listThemes, applyThemeById } from "../utils/themeLoader";
import { applyWindowEffect } from "../utils/applyWindowEffect";

interface WidgetSelectPayload {
  index: number;
  themeId?: string;
}

export function WidgetSelectWindow() {
  const [manifests, setManifests] = useState<WidgetManifest[]>([]);
  const [index, setIndex] = useState<number>(-1);

  // メインからデータを受信 & マニフェスト一覧取得
  useEffect(() => {
    let cancelled = false;

    // ブラウザデフォルト右クリック抑制
    const suppress = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", suppress);

    async function init() {
      const unlistenFn = await listen<WidgetSelectPayload>("widget-select-init", (event) => {
        if (cancelled) return;
        setIndex(event.payload.index);
        // テーマ適用
        if (event.payload.themeId) {
          listThemes().then((themes) => {
            if (!cancelled) {
              applyThemeById(themes, event.payload.themeId!);
              applyWindowEffect(themes, event.payload.themeId!);
            }
          });
        }
      });

      if (cancelled) {
        unlistenFn();
        return;
      }

      // マニフェスト一覧取得
      try {
        const list = await invoke<WidgetManifest[]>("list_widgets");
        if (!cancelled) setManifests(list);
      } catch (err) {
        console.error("Failed to list widgets:", err);
      }

      // リスナー登録完了後にメインへ準備完了を通知
      await emit("widget-select-ready", {});
      setTimeout(() => {
        if (!cancelled) emit("widget-select-ready", {});
      }, 500);

      if (!cancelled) {
        cleanupRef = unlistenFn;
      } else {
        unlistenFn();
      }
    }

    let cleanupRef: (() => void) | null = null;
    init();

    return () => {
      cancelled = true;
      cleanupRef?.();
      document.removeEventListener("contextmenu", suppress);
    };
  }, []);

  const handleSelect = async (widgetId: string) => {
    await emit("widget-select-result", { widgetId, index });
    getCurrentWebviewWindow().close();
  };

  const handleClose = async () => {
    await emit("widget-select-closed", {});
    getCurrentWebviewWindow().close();
  };

  return (
    <div className="widget-select-app">
      <div className="wsel-header">🧩 ウィジェットを選択</div>

      <div className="wsel-body">
        <div className="wsel-grid">
          {manifests.map((m) => (
            <button
              key={m.id}
              className="wsel-item"
              onClick={() => handleSelect(m.id)}
              title={m.description}
            >
              <span className="wsel-emoji">{m.emoji}</span>
              <span className="wsel-label">{m.label}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="wsel-footer">
        <button className="dialog-btn" onClick={handleClose}>キャンセル</button>
      </div>
    </div>
  );
}
