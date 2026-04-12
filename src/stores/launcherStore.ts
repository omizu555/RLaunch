/* ============================================================
   MyLauncher - Launcher Store (状態管理)
   Tauri store プラグインで JSON 永続化
   ============================================================ */
import { load, type Store } from "@tauri-apps/plugin-store";
import type { Tab, AppSettings, GridCell } from "../types";
import { DEFAULT_SETTINGS } from "../types";

let store: Store | null = null;
let backupDone = false;

/** P-37: 自動バックアップ（3世代ローテーション） */
async function autoBackup(): Promise<void> {
  if (backupDone) return;
  backupDone = true;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const storePath: string = await invoke("get_store_path");
    const { exists, readTextFile, writeTextFile, remove } = await import("@tauri-apps/plugin-fs");
    if (!await exists(storePath)) return;
    const content = await readTextFile(storePath);
    // ローテーション: .bak.3 → 削除, .bak.2 → .bak.3, .bak.1 → .bak.2, 現在 → .bak.1
    const bak = (n: number) => storePath.replace(/\.json$/, `.bak.${n}.json`);
    try { if (await exists(bak(3))) await remove(bak(3)); } catch { /* ignore */ }
    for (let i = 2; i >= 1; i--) {
      try {
        if (await exists(bak(i))) {
          const c = await readTextFile(bak(i));
          await writeTextFile(bak(i + 1), c);
        }
      } catch { /* ignore */ }
    }
    await writeTextFile(bak(1), content);
  } catch (e) {
    console.warn("Auto backup failed:", e);
  }
}

/** Store の初期化 */
async function getStore(): Promise<Store> {
  if (!store) {
    store = await load("launcher-data.json", { autoSave: true, defaults: {} });
    // 初回ロード後にバックアップ
    autoBackup();
  }
  return store;
}

// ────────────────────────────────────────────────────────────
//  タブ操作
// ────────────────────────────────────────────────────────────

/** 全タブを取得 */
export async function getTabs(): Promise<Tab[]> {
  const s = await getStore();
  const tabs = await s.get<Tab[]>("tabs");
  if (!tabs || tabs.length === 0) {
    // 初回: デフォルトタブを作成
    const defaultTab = createDefaultTab();
    await s.set("tabs", [defaultTab]);
    return [defaultTab];
  }
  return tabs;
}

/** タブを保存 */
export async function saveTabs(tabs: Tab[]): Promise<void> {
  const s = await getStore();
  await s.set("tabs", tabs);
}

/** デフォルトタブ生成 */
function createDefaultTab(): Tab {
  return {
    id: crypto.randomUUID(),
    label: "メイン",
    order: 0,
    gridColumns: 8,
    gridRows: 4,
    items: new Array(32).fill(null),
  };
}

/** 新しいタブを追加（グリッドサイズ指定可能） */
export async function addTab(
  label: string,
  gridColumns?: number,
  gridRows?: number
): Promise<Tab[]> {
  const tabs = await getTabs();
  const cols = gridColumns ?? 8;
  const rows = gridRows ?? 4;
  const newTab: Tab = {
    id: crypto.randomUUID(),
    label,
    order: tabs.length,
    gridColumns: cols,
    gridRows: rows,
    items: new Array(cols * rows).fill(null),
  };
  tabs.push(newTab);
  await saveTabs(tabs);
  return tabs;
}

/**
 * グリッドアイテムを新しいサイズにリマップ（行列ベースで既存アイテムを保持）
 */
function remapGridItems(
  oldItems: GridCell[],
  oldCols: number,
  oldRows: number,
  newCols: number,
  newRows: number
): GridCell[] {
  const newItems: GridCell[] = new Array(newCols * newRows).fill(null);
  for (let r = 0; r < Math.min(oldRows, newRows); r++) {
    for (let c = 0; c < Math.min(oldCols, newCols); c++) {
      const oldIdx = r * oldCols + c;
      const newIdx = r * newCols + c;
      if (oldIdx < oldItems.length) {
        newItems[newIdx] = oldItems[oldIdx];
      }
    }
  }
  return newItems;
}

/** 全タブのグリッドサイズを一括変更 */
export async function resizeAllTabsGrid(
  newCols: number,
  newRows: number
): Promise<Tab[]> {
  const tabs = await getTabs();
  for (const tab of tabs) {
    tab.items = remapGridItems(tab.items, tab.gridColumns, tab.gridRows, newCols, newRows);
    tab.gridColumns = newCols;
    tab.gridRows = newRows;
  }
  await saveTabs(tabs);
  return tabs;
}

/** タブを削除 */
export async function removeTab(tabId: string): Promise<Tab[]> {
  let tabs = await getTabs();
  tabs = tabs.filter((t) => t.id !== tabId);
  // 最低1タブは維持
  if (tabs.length === 0) {
    tabs = [createDefaultTab()];
  }
  await saveTabs(tabs);
  return tabs;
}

/** タブ名を変更 */
export async function renameTab(tabId: string, newLabel: string): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (tab) {
    tab.label = newLabel;
    await saveTabs(tabs);
  }
  return tabs;
}

/** グリッドセルを更新 */
export async function setGridCell(
  tabId: string,
  index: number,
  item: GridCell
): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (tab && index >= 0 && index < tab.items.length) {
    tab.items[index] = item;
    await saveTabs(tabs);
  }
  return tabs;
}

/** グリッドセルをクリア */
export async function clearGridCell(tabId: string, index: number): Promise<Tab[]> {
  return setGridCell(tabId, index, null);
}

/**
 * グリッドセルに挿入（既存アイテムを後ろにシフト）
 * index の位置に item を挿入し、index 以降のアイテムを1つ後ろにずらす。
 * 最後のアイテムが null でなければ溢れて消失するので注意。
 */
export async function insertGridCell(
  tabId: string,
  index: number,
  item: GridCell
): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (tab && index >= 0 && index < tab.items.length) {
    // index 以降を1つ後ろにシフト（最後の要素は溢れて消える）
    for (let i = tab.items.length - 1; i > index; i--) {
      tab.items[i] = tab.items[i - 1];
    }
    tab.items[index] = item;
    await saveTabs(tabs);
  }
  return tabs;
}

/** グリッドセルを入れ替え（ボタン並び替え） */
export async function swapGridCells(
  tabId: string,
  fromIndex: number,
  toIndex: number
): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (
    tab &&
    fromIndex >= 0 && fromIndex < tab.items.length &&
    toIndex >= 0 && toIndex < tab.items.length
  ) {
    const temp = tab.items[fromIndex];
    tab.items[fromIndex] = tab.items[toIndex];
    tab.items[toIndex] = temp;
    await saveTabs(tabs);
  }
  return tabs;
}

/** P-03: タブの並び替え */
export async function reorderTabs(fromIndex: number, toIndex: number): Promise<Tab[]> {
  const tabs = await getTabs();
  if (fromIndex < 0 || fromIndex >= tabs.length || toIndex < 0 || toIndex >= tabs.length) return tabs;
  const [moved] = tabs.splice(fromIndex, 1);
  tabs.splice(toIndex, 0, moved);
  tabs.forEach((t, i) => { t.order = i; });
  await saveTabs(tabs);
  return tabs;
}

/** P-04: タブカラー変更 */
export async function setTabColor(tabId: string, color: string): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (tab) {
    (tab as Tab & { color?: string }).color = color || undefined;
    await saveTabs(tabs);
  }
  return tabs;
}

/** P-06: タブ複製 */
export async function duplicateTab(tabId: string): Promise<Tab[]> {
  const tabs = await getTabs();
  const source = tabs.find((t) => t.id === tabId);
  if (!source) return tabs;
  const newTab: Tab = {
    id: crypto.randomUUID(),
    label: `${source.label} のコピー`,
    order: tabs.length,
    color: (source as Tab & { color?: string }).color,
    gridColumns: source.gridColumns,
    gridRows: source.gridRows,
    viewMode: source.viewMode,
    listColumns: source.listColumns,
    items: JSON.parse(JSON.stringify(source.items)).map((item: GridCell) => {
      if (item) return { ...item, id: crypto.randomUUID() };
      return null;
    }),
  };
  tabs.push(newTab);
  await saveTabs(tabs);
  return tabs;
}

/** P-25: 個別タブのグリッドサイズ変更 */
export async function resizeTabGrid(
  tabId: string,
  newCols: number,
  newRows: number
): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (!tab) return tabs;
  tab.items = remapGridItems(tab.items, tab.gridColumns, tab.gridRows, newCols, newRows);
  tab.gridColumns = newCols;
  tab.gridRows = newRows;
  await saveTabs(tabs);
  return tabs;
}

/** タブ個別の表示設定を更新 */
export async function setTabDisplaySettings(
  tabId: string,
  displaySettings: { viewMode?: "grid" | "list"; listColumns?: number }
): Promise<Tab[]> {
  const tabs = await getTabs();
  const tab = tabs.find((t) => t.id === tabId);
  if (tab) {
    if (displaySettings.viewMode !== undefined) {
      tab.viewMode = displaySettings.viewMode || undefined;
    }
    if (displaySettings.listColumns !== undefined) {
      tab.listColumns = displaySettings.listColumns || undefined;
    }
    await saveTabs(tabs);
  }
  return tabs;
}

// ────────────────────────────────────────────────────────────
//  設定操作
// ────────────────────────────────────────────────────────────

/** 設定を取得（デフォルト値とマージして新項目追加時も安全） */
export async function getSettings(): Promise<AppSettings> {
  const s = await getStore();
  const stored = await s.get<Partial<AppSettings>>("settings");
  return { ...DEFAULT_SETTINGS, ...(stored ?? {}) };
}

/** 設定を保存 */
export async function saveSettings(settings: AppSettings): Promise<void> {
  const s = await getStore();
  await s.set("settings", settings);
}

/** P-36: インポート（上書き or マージ） */
export async function importData(
  data: Record<string, unknown>,
  mode: "overwrite" | "merge"
): Promise<void> {
  const s = await getStore();
  if (mode === "overwrite") {
    // 全キーを置換
    if (data.tabs) await s.set("tabs", data.tabs);
    if (data.settings) await s.set("settings", data.settings);
  } else {
    // マージ: タブは追加、設定は上書き
    if (Array.isArray(data.tabs)) {
      const existing = await s.get<Tab[]>("tabs") ?? [];
      const existingIds = new Set(existing.map((t) => t.id));
      const newTabs = (data.tabs as Tab[]).filter((t) => !existingIds.has(t.id));
      await s.set("tabs", [...existing, ...newTabs]);
    }
    if (data.settings && typeof data.settings === "object") {
      const current = await s.get<Partial<AppSettings>>("settings") ?? {};
      await s.set("settings", { ...current, ...(data.settings as object) });
    }
  }
}
