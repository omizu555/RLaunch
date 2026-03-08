/* ============================================================
   domHelpers.ts - DOM 操作ユーティリティ
   グリッドセルの位置特定など、複数コンポーネントで共通使用。
   ============================================================ */

interface Position {
  x: number;
  y: number;
}

/**
 * Tauri の PhysicalPosition (デバイスピクセル座標) から
 * `data-cell-index` 属性を持つグリッドセルのインデックスを特定する。
 *
 * @param position - { x, y } デバイスピクセル座標
 * @returns セルインデックス、または該当なしの場合 null
 */
export function getCellIndexFromPosition(position: Position): number | null {
  const scale = window.devicePixelRatio || 1;
  const x = position.x / scale;
  const y = position.y / scale;

  const el = document.elementFromPoint(x, y);
  if (!el) return null;

  const btn = el.closest("[data-cell-index]");
  if (!btn) return null;

  const index = parseInt(btn.getAttribute("data-cell-index") || "", 10);
  return isNaN(index) ? null : index;
}

/**
 * GridCell 配列を構築する。
 * items 配列を指定サイズに正規化し、不足分は null で埋める。
 *
 * @param items - 元の GridCell 配列
 * @param columns - グリッド列数
 * @param rows - グリッド行数
 * @returns columns * rows の長さの GridCell 配列
 */
export function buildCellArray<T>(items: T[], columns: number, rows: number): (T | null)[] {
  const total = columns * rows;
  const cells: (T | null)[] = [];
  for (let i = 0; i < total; i++) {
    cells.push(items[i] ?? null);
  }
  return cells;
}
