/* ============================================================
   WidgetRenderer - Canvas ベースのウィジェット描画コンポーネント
   ビルトインウィジェットは TypeScript draw 関数、
   プラグインウィジェットは widget.js を動的ロードして描画。
   ============================================================ */
import React, { useEffect, useRef, useCallback, useState } from "react";
import type { WidgetItem } from "../types";
import type {
  AnalogClockConfig,
  DigitalClockConfig,
  CountdownTimerConfig,
  SystemMonitorConfig,
  DateCalendarConfig,
} from "../types";
import { isBuiltinWidget } from "../types";
import { drawAnalogClock } from "./drawAnalogClock";
import { drawDigitalClock } from "./drawDigitalClock";
import { drawCountdownTimer } from "./drawCountdownTimer";
import { drawSystemMonitor } from "./drawSystemMonitor";
import { drawDateCalendar } from "./drawDateCalendar";
import { loadPluginDrawFn, getWidgetManifest } from "../utils/widgetLoader";
import { invoke } from "@tauri-apps/api/core";

/** Rust 側 get_system_info の戻り値 */
interface SystemInfo {
  cpu_usage: number;
  memory_usage: number;
}

// システム情報のキャッシュ（複数モニターで共有）
let cachedSystemInfo: SystemInfo = { cpu_usage: 0, memory_usage: 0 };
let lastFetchTime = 0;
const FETCH_COOLDOWN = 1500; // ms

async function fetchSystemInfo(): Promise<SystemInfo> {
  const now = Date.now();
  if (now - lastFetchTime < FETCH_COOLDOWN) {
    return cachedSystemInfo;
  }
  try {
    cachedSystemInfo = await invoke<SystemInfo>("get_system_info");
    lastFetchTime = now;
  } catch {
    // Rust コマンド失敗時は最後の正常値を維持（初期値は 0,0）
  }
  return cachedSystemInfo;
}

/** needsSystemInfo か判定 (マニフェストからキャッシュ) */
const needsInfoCache = new Map<string, boolean>();
async function checkNeedsSystemInfo(widgetId: string): Promise<boolean> {
  if (needsInfoCache.has(widgetId)) return needsInfoCache.get(widgetId)!;
  const manifest = await getWidgetManifest(widgetId);
  const needs = manifest?.needsSystemInfo ?? false;
  needsInfoCache.set(widgetId, needs);
  return needs;
}

interface WidgetRendererProps {
  widget: WidgetItem;
  width?: number;
  height?: number;
}

export function WidgetRenderer({ widget, width, height }: WidgetRendererProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const clickedRef = useRef(false);
  const dpr = window.devicePixelRatio || 1;

  // サイズが指定されない場合は親要素のサイズを使う
  const [autoSize, setAutoSize] = useState({ w: 64, h: 64 });

  useEffect(() => {
    if (width && height) return; // 固定サイズの場合は不要
    const container = containerRef.current;
    if (!container) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width: cw, height: ch } = entry.contentRect;
        if (cw > 0 && ch > 0) {
          setAutoSize({ w: Math.floor(cw), h: Math.floor(ch) });
        }
      }
    });
    observer.observe(container);
    return () => observer.disconnect();
  }, [width, height]);

  const effectiveW = width ?? autoSize.w;
  const effectiveH = height ?? autoSize.h;
  const cw = effectiveW * dpr;
  const ch = effectiveH * dpr;

  const draw = useCallback(async () => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.save();
    ctx.clearRect(0, 0, cw, ch);
    ctx.scale(dpr, dpr);

    const config = widget.config as Record<string, unknown>;
    const wType = widget.widgetType;

    if (isBuiltinWidget(wType)) {
      // ═══ ビルトインウィジェット ═══
      switch (wType) {
        case "analog-clock":
          drawAnalogClock(ctx, effectiveW, effectiveH, config as unknown as AnalogClockConfig);
          break;
        case "digital-clock":
          drawDigitalClock(ctx, effectiveW, effectiveH, config as unknown as DigitalClockConfig);
          break;
        case "countdown-timer":
          drawCountdownTimer(ctx, effectiveW, effectiveH, config as unknown as CountdownTimerConfig);
          break;
        case "cpu-monitor": {
          const info = await fetchSystemInfo();
          drawSystemMonitor(ctx, effectiveW, effectiveH, config as unknown as SystemMonitorConfig, info.cpu_usage);
          break;
        }
        case "memory-monitor": {
          const info = await fetchSystemInfo();
          drawSystemMonitor(ctx, effectiveW, effectiveH, config as unknown as SystemMonitorConfig, info.memory_usage);
          break;
        }
        case "date-calendar":
          drawDateCalendar(ctx, effectiveW, effectiveH, config as unknown as DateCalendarConfig);
          break;
      }
    } else {
      // ═══ プラグインウィジェット ═══
      const drawFn = await loadPluginDrawFn(wType);
      if (drawFn) {
        const needsInfo = await checkNeedsSystemInfo(wType);
        const sysInfo = needsInfo ? await fetchSystemInfo() : undefined;
        const clicked = clickedRef.current;
        clickedRef.current = false; // 消費後にリセット
        drawFn(ctx, effectiveW, effectiveH, config, {
          now: new Date(),
          systemInfo: sysInfo,
          clicked,
          invoke,
        });
      } else {
        // draw 関数が無い場合のフォールバック
        ctx.fillStyle = "#6c7086";
        ctx.font = `${Math.max(8, effectiveW * 0.09)}px sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("⚠ No script", effectiveW / 2, effectiveH / 2);
      }
    }

    ctx.restore();
  }, [widget, effectiveW, effectiveH, cw, ch, dpr]);

  useEffect(() => {
    // 初回即描画
    draw();

    // 定期更新
    const interval = setInterval(draw, widget.updateInterval);
    return () => clearInterval(interval);
  }, [draw, widget.updateInterval]);

  /** キャンバスクリック → 次の draw サイクルで clicked=true を渡す */
  const handleCanvasClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    e.stopPropagation(); // LauncherButton の onClick に伝播させない
    clickedRef.current = true;
    // 即座に再描画してクリック反映
    draw();
  }, [draw]);

  return (
    <div
      ref={containerRef}
      style={{ width: "100%", height: "100%", display: "flex", alignItems: "center", justifyContent: "center" }}
    >
      <canvas
        ref={canvasRef}
        width={cw}
        height={ch}
        onClick={handleCanvasClick}
        style={{
          width: `${effectiveW}px`,
          height: `${effectiveH}px`,
          display: "block",
          borderRadius: "4px",
          cursor: "pointer",
        }}
      />
    </div>
  );
}
