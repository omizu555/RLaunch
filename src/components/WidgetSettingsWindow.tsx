/* ============================================================
   WidgetSettingsWindow - 独立ウィジェット設定ウィンドウ
   Tauri emit/listen でメインウィンドウと通信
   ============================================================ */
import { useState, useEffect } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { WidgetItem, WidgetManifest } from "../types";
import { getWidgetManifest } from "../utils/widgetLoader";
import { listThemes, applyThemeById } from "../utils/themeLoader";
import { applyWindowEffect } from "../utils/applyWindowEffect";
import { SchemaField } from "./SchemaFieldRenderer";

/** メインから受け取るペイロード */
interface WidgetSettingsPayload {
  widget: WidgetItem;
  index: number;
  themeId?: string;
}

export function WidgetSettingsWindow() {
  const [widget, setWidget] = useState<WidgetItem | null>(null);
  const [index, setIndex] = useState<number>(-1);
  const [config, setConfig] = useState<Record<string, unknown>>({});
  const [interval_, setInterval_] = useState(1000);
  const [colSpan, setColSpan] = useState(1);
  const [rowSpan, setRowSpan] = useState(1);
  const [manifest, setManifest] = useState<WidgetManifest | null>(null);

  // メインから設定データを受信
  useEffect(() => {
    let cancelled = false;

    async function init() {
      const unlistenFn = await listen<WidgetSettingsPayload>("widget-settings-init", (event) => {
        if (cancelled) return;
        const { widget: w, index: idx, themeId } = event.payload;
        setWidget(w);
        setIndex(idx);
        setConfig({ ...w.config });
        setInterval_(w.updateInterval);
        setColSpan(w.colSpan ?? 1);
        setRowSpan(w.rowSpan ?? 1);

        // マニフェスト取得
        getWidgetManifest(w.widgetType).then((m) => {
          if (!cancelled) setManifest(m ?? null);
        });

        // テーマ適用
        if (themeId) {
          listThemes().then((themes) => {
            if (!cancelled) {
              applyThemeById(themes, themeId);
              applyWindowEffect(themes, themeId);
            }
          });
        }
      });

      if (cancelled) {
        unlistenFn();
        return;
      }

      // リスナー登録完了後にメインへ準備完了を通知
      await emit("widget-settings-ready", {});

      // 安全弁: 500ms 後に再度 ready を送信
      setTimeout(() => {
        if (!cancelled) emit("widget-settings-ready", {});
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
    };
  }, []);

  const update = (key: string, value: unknown) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  };

  const handleSave = async () => {
    if (!widget) return;
    const updated: WidgetItem = {
      ...widget,
      config,
      updateInterval: interval_,
      colSpan: colSpan > 1 ? colSpan : undefined,
      rowSpan: rowSpan > 1 ? rowSpan : undefined,
      updatedAt: new Date().toISOString(),
    };
    await emit("widget-settings-save", { widget: updated, index });
    getCurrentWebviewWindow().close();
  };

  const handleClose = async () => {
    await emit("widget-settings-closed", {});
    getCurrentWebviewWindow().close();
  };

  // まだデータ受信前
  if (!widget) {
    return (
      <div className="widget-settings-app" style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
        <span style={{ color: "var(--text-muted, #6c7086)" }}>読み込み中...</span>
      </div>
    );
  }

  const title = manifest ? manifest.label : widget.widgetType;

  return (
    <div className="widget-settings-app">
      <div className="ws-header">⚙ {title} の設定</div>

      <div className="ws-body">
        {/* 更新間隔（共通） */}
        <div className="ws-field">
          <label>更新間隔 (ms)</label>
          <input
            type="number"
            value={interval_}
            min={16}
            max={3600000}
            step={100}
            onChange={(e) => setInterval_(Number(e.target.value))}
          />
        </div>

        {/* P-30: ウィジェットサイズ（スパン） */}
        <div className="ws-field" style={{ display: "flex", gap: 8 }}>
          <div style={{ flex: 1 }}>
            <label>横スパン</label>
            <input
              type="number"
              value={colSpan}
              min={1}
              max={4}
              onChange={(e) => setColSpan(Math.max(1, Math.min(4, Number(e.target.value))))}
            />
          </div>
          <div style={{ flex: 1 }}>
            <label>縦スパン</label>
            <input
              type="number"
              value={rowSpan}
              min={1}
              max={4}
              onChange={(e) => setRowSpan(Math.max(1, Math.min(4, Number(e.target.value))))}
            />
          </div>
        </div>

        {/* configSchema から自動生成 */}
        {manifest?.configSchema.map((field) => (
          <SchemaField key={field.key} field={field} config={config} update={update} />
        ))}
      </div>

      <div className="ws-footer">
        <button className="dialog-btn secondary" onClick={handleClose}>
          キャンセル
        </button>
        <button className="dialog-btn primary" onClick={handleSave}>
          保存
        </button>
      </div>
    </div>
  );
}

/* (SchemaField, ColorField, CheckboxField, SelectField, toLocalDatetime は
   SchemaFieldRenderer.tsx に共通化済み) */
