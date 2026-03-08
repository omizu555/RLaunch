/* ============================================================
   widgetLoader - ウィジェットプラグインの読み込みユーティリティ
   Rust コマンド経由でマニフェスト一覧・スクリプトを取得し、
   draw 関数をキャッシュ付きで動的ロードする。
   ============================================================ */
import { invoke } from "@tauri-apps/api/core";
import type { WidgetManifest } from "../types";

/** draw 関数のシグネチャ */
export type WidgetDrawFn = (
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: Record<string, unknown>,
  data: {
    now: Date;
    systemInfo?: { cpu_usage: number; memory_usage: number };
    clicked?: boolean;
    invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
  },
) => void;

// キャッシュ
let manifestCache: WidgetManifest[] | null = null;
const drawFnCache = new Map<string, WidgetDrawFn>();

/** マニフェスト一覧を取得（キャッシュ付き） */
async function listWidgetManifests(forceRefresh = false): Promise<WidgetManifest[]> {
  if (!forceRefresh && manifestCache) return manifestCache;
  manifestCache = await invoke<WidgetManifest[]>("list_widgets");
  return manifestCache;
}

/** 指定 ID のマニフェストを取得 */
export async function getWidgetManifest(id: string): Promise<WidgetManifest | undefined> {
  const all = await listWidgetManifests();
  return all.find((m) => m.id === id);
}

/** マニフェストの configSchema からデフォルト設定オブジェクトを生成 */
export function buildDefaultConfig(manifest: WidgetManifest): Record<string, unknown> {
  const cfg: Record<string, unknown> = {};
  for (const field of manifest.configSchema) {
    cfg[field.key] = field.default;
  }
  return cfg;
}

/**
 * プラグインウィジェットの draw 関数をロード（キャッシュ付き）
 * widget.js を Rust 経由で取得し、new Function() で draw 関数を抽出
 */
export async function loadPluginDrawFn(widgetId: string): Promise<WidgetDrawFn | null> {
  const cached = drawFnCache.get(widgetId);
  if (cached) return cached;

  try {
    const script = await invoke<string>("get_widget_script", { widgetId });
    if (!script || script.trim() === "") return null;

    // スクリプトを「ファクトリ関数」として一度だけ実行し、draw 関数を抽出する。
    // こうすることで var _pom = ... 等のグローバル状態がクロージャに閉じ込められ、
    // 描画呼び出しをまたいで状態が保持される。
    const factory = new Function(
      script + "\n; return typeof draw === 'function' ? draw : null;"
    );
    const extractedDraw = factory();

    if (typeof extractedDraw !== "function") return null;

    const drawFn: WidgetDrawFn = (ctx, w, h, config, data) => {
      try {
        extractedDraw(ctx, w, h, config, data);
      } catch (err) {
        console.error(`Widget '${widgetId}' draw error:`, err);
        // エラー時はフォールバック描画
        ctx.clearRect(0, 0, w, h);
        ctx.fillStyle = "#f38ba8";
        ctx.font = `${Math.max(8, w * 0.08)}px sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("⚠ Error", w / 2, h / 2);
      }
    };

    drawFnCache.set(widgetId, drawFn);
    return drawFn;
  } catch (err) {
    console.error(`Failed to load widget script '${widgetId}':`, err);
    return null;
  }
}

