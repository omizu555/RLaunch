/* ============================================================
   TabBar - タブバー
   P-03: D&D 並び替え / P-04: カラーマーカー / P-05: 削除フォールバック / P-06: 複製
   ============================================================ */
import "./TabBar.css";
import { useState, useRef, useEffect, useCallback } from "react";
import type { Tab } from "../types";

/** タブカラーマーカーのプリセット */
const TAB_COLORS = [
  "", "#ef4444", "#f97316", "#eab308", "#22c55e",
  "#06b6d4", "#3b82f6", "#8b5cf6", "#ec4899",
];

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string;
  onSelectTab: (tabId: string) => void;
  onAddTab: () => void;
  onRenameTab: (tabId: string, newLabel: string) => void;
  onRemoveTab: (tabId: string) => void;
  /** P-03: タブ並び替え */
  onReorderTabs?: (fromIndex: number, toIndex: number) => void;
  /** P-06: タブ複製 */
  onDuplicateTab?: (tabId: string) => void;
  /** P-04: タブカラー変更 */
  onTabColorChange?: (tabId: string, color: string) => void;
  /** P-12: ドラッグ中のタブホバー通知 */
  isDraggingItem?: boolean;
  /** P-25: タブごとのグリッドサイズ変更 */
  onResizeTab?: (tabId: string, cols: number, rows: number) => void;
  /** タブ個別の表示設定 */
  onTabSettings?: (tabId: string) => void;
}

interface TabMenu {
  tabId: string;
  x: number;
  y: number;
}

export function TabBar({
  tabs,
  activeTabId,
  onSelectTab,
  onAddTab,
  onRenameTab,
  onRemoveTab,
  onReorderTabs,
  onDuplicateTab,
  onTabColorChange,
  isDraggingItem,
  onResizeTab,
  onTabSettings,
}: TabBarProps) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [menu, setMenu] = useState<TabMenu | null>(null);
  const [showColorPicker, setShowColorPicker] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // ── P-03: タブ D&D 並び替え ──
  const [dragTabId, setDragTabId] = useState<string | null>(null);
  const [dragOverTabId, setDragOverTabId] = useState<string | null>(null);
  const dragStartX = useRef(0);
  const dragActivated = useRef(false);

  const handleTabPointerDown = useCallback((e: React.PointerEvent, tabId: string) => {
    if (e.button !== 0 || editingId) return;
    dragStartX.current = e.clientX;
    dragActivated.current = false;
    setDragTabId(tabId);
  }, [editingId]);

  useEffect(() => {
    if (!dragTabId) return;
    const handleMove = (e: PointerEvent) => {
      if (!dragActivated.current && Math.abs(e.clientX - dragStartX.current) > 5) {
        dragActivated.current = true;
      }
      if (!dragActivated.current) return;
      const el = document.elementFromPoint(e.clientX, e.clientY);
      const tabEl = el?.closest("[data-tab-id]");
      const targetId = tabEl?.getAttribute("data-tab-id") ?? null;
      setDragOverTabId(targetId);
    };
    const handleUp = () => {
      if (dragActivated.current && dragOverTabId && dragOverTabId !== dragTabId && onReorderTabs) {
        const fromIdx = tabs.findIndex((t) => t.id === dragTabId);
        const toIdx = tabs.findIndex((t) => t.id === dragOverTabId);
        if (fromIdx >= 0 && toIdx >= 0) {
          onReorderTabs(fromIdx, toIdx);
        }
      }
      setDragTabId(null);
      setDragOverTabId(null);
      dragActivated.current = false;
    };
    document.addEventListener("pointermove", handleMove);
    document.addEventListener("pointerup", handleUp);
    return () => {
      document.removeEventListener("pointermove", handleMove);
      document.removeEventListener("pointerup", handleUp);
    };
  }, [dragTabId, dragOverTabId, tabs, onReorderTabs]);

  // ── P-12: アイテム D&D 中のタブホバー切替 ──
  const hoverTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const handleTabDragHover = useCallback((tabId: string) => {
    if (!isDraggingItem || tabId === activeTabId) return;
    if (hoverTimer.current) clearTimeout(hoverTimer.current);
    hoverTimer.current = setTimeout(() => {
      onSelectTab(tabId);
    }, 500);
  }, [isDraggingItem, activeTabId, onSelectTab]);

  const handleTabDragLeave = useCallback(() => {
    if (hoverTimer.current) {
      clearTimeout(hoverTimer.current);
      hoverTimer.current = null;
    }
  }, []);

  const handleDoubleClick = (tabId: string) => {
    setEditingId(tabId);
    setTimeout(() => inputRef.current?.select(), 0);
  };

  const finishEdit = (tabId: string, value: string) => {
    const trimmed = value.trim();
    if (trimmed) {
      onRenameTab(tabId, trimmed);
    }
    setEditingId(null);
  };

  const handleWheel = (e: React.WheelEvent) => {
    const currentIdx = tabs.findIndex((t) => t.id === activeTabId);
    if (e.deltaY > 0 && currentIdx < tabs.length - 1) {
      onSelectTab(tabs[currentIdx + 1].id);
    } else if (e.deltaY < 0 && currentIdx > 0) {
      onSelectTab(tabs[currentIdx - 1].id);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, tabId: string) => {
    e.preventDefault();
    setMenu({ tabId, x: e.clientX, y: e.clientY });
    setShowColorPicker(false);
  };

  // メニュー外クリックで閉じる
  useEffect(() => {
    if (!menu) return;
    const handler = () => { setMenu(null); setShowColorPicker(false); };
    window.addEventListener("click", handler);
    return () => window.removeEventListener("click", handler);
  }, [menu]);

  return (
    <div className="tabbar" role="tablist" onWheel={handleWheel}>
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        const isDragSource = dragTabId === tab.id && dragActivated.current;
        const isDragTarget = dragOverTabId === tab.id && dragActivated.current;
        return (
          <div
            key={tab.id}
            data-tab-id={tab.id}
            role="tab"
            aria-selected={isActive}
            className={`tab ${isActive ? "active" : ""} ${isDragSource ? "tab-dragging" : ""} ${isDragTarget ? "tab-drag-over" : ""}`}
            onClick={() => {
              if (!dragActivated.current) onSelectTab(tab.id);
            }}
            onDoubleClick={() => handleDoubleClick(tab.id)}
            onContextMenu={(e) => handleContextMenu(e, tab.id)}
            onPointerDown={(e) => handleTabPointerDown(e, tab.id)}
            onPointerEnter={() => handleTabDragHover(tab.id)}
            onPointerLeave={handleTabDragLeave}
          >
            {/* P-04: カラーマーカー */}
            {(tab as Tab & { color?: string }).color && (
              <span className="tab-color-dot" style={{ background: (tab as Tab & { color?: string }).color }} />
            )}
            {editingId === tab.id ? (
              <input
                ref={inputRef}
                defaultValue={tab.label}
                style={{
                  background: "transparent",
                  border: "none",
                  color: "inherit",
                  font: "inherit",
                  width: "60px",
                  outline: "none",
                  textAlign: "center",
                }}
                onBlur={(e) => finishEdit(tab.id, e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") finishEdit(tab.id, e.currentTarget.value);
                  if (e.key === "Escape") setEditingId(null);
                }}
                autoFocus
              />
            ) : (
              tab.label
            )}
          </div>
        );
      })}
      <button className="tab-add" title="タブを追加" onClick={onAddTab}>
        ＋
      </button>

      {/* タブ右クリックメニュー */}
      {menu && (
        <>
          <div className="tab-context-overlay" onClick={() => setMenu(null)} onContextMenu={(e) => { e.preventDefault(); setMenu(null); }} />
          <div
            className="tab-context-menu"
            style={{ left: Math.min(menu.x, window.innerWidth - 180), top: menu.y }}
          >
            <div
              className="tab-context-item"
              onClick={() => {
                handleDoubleClick(menu.tabId);
                setMenu(null);
              }}
            >
              ✏ タブ名を変更
            </div>
            {/* P-06: タブ複製 */}
            {onDuplicateTab && (
              <div
                className="tab-context-item"
                onClick={() => {
                  onDuplicateTab(menu.tabId);
                  setMenu(null);
                }}
              >
                📋 タブを複製
              </div>
            )}
            {/* P-25: タブごとのグリッドサイズ変更 */}
            {onResizeTab && (
              <div
                className="tab-context-item"
                onClick={() => {
                  const tab = tabs.find((t) => t.id === menu.tabId);
                  if (!tab) return;
                  const input = window.prompt(
                    `グリッドサイズを変更（列×行）\n現在: ${tab.gridColumns}×${tab.gridRows}`,
                    `${tab.gridColumns}×${tab.gridRows}`
                  );
                  if (!input) { setMenu(null); return; }
                  const match = input.match(/^(\d+)\s*[×x,\s]\s*(\d+)$/i);
                  if (match) {
                    const cols = Math.max(1, Math.min(20, parseInt(match[1])));
                    const rows = Math.max(1, Math.min(10, parseInt(match[2])));
                    onResizeTab(menu.tabId, cols, rows);
                  }
                  setMenu(null);
                }}
              >
                📐 グリッドサイズを変更
              </div>
            )}
            {/* タブ個別表示設定 */}
            {onTabSettings && (
              <div
                className="tab-context-item"
                onClick={() => {
                  onTabSettings(menu.tabId);
                  setMenu(null);
                }}
              >
                ⚙ タブ表示設定
              </div>
            )}
            {/* P-04: カラー変更 */}
            {onTabColorChange && (
              <div
                className="tab-context-item"
                onClick={(e) => { e.stopPropagation(); setShowColorPicker(!showColorPicker); }}
              >
                🎨 カラーを変更
                {showColorPicker && (
                  <div className="tab-color-picker" onClick={(e) => e.stopPropagation()}>
                    {TAB_COLORS.map((c) => (
                      <button
                        key={c || "none"}
                        className={`tab-color-option ${c === "" ? "no-color" : ""}`}
                        style={c ? { background: c } : undefined}
                        title={c || "なし"}
                        onClick={() => {
                          onTabColorChange(menu.tabId, c);
                          setMenu(null);
                          setShowColorPicker(false);
                        }}
                      >
                        {c === "" ? "✕" : ""}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
            {tabs.length > 1 && (
              <>
                <div className="tab-context-separator" />
                <div
                  className="tab-context-item danger"
                  onClick={() => {
                    const tab = tabs.find((t) => t.id === menu.tabId);
                    const itemCount = tab?.items.filter(Boolean).length ?? 0;
                    const msg = itemCount > 0
                      ? `タブ「${tab?.label}」を削除しますか？\n（${itemCount} アイテム登録済み）`
                      : `タブ「${tab?.label}」を削除しますか？`;
                    if (window.confirm(msg)) {
                      onRemoveTab(menu.tabId);
                    }
                    setMenu(null);
                  }}
                >
                  🗑 タブを削除
                </div>
              </>
            )}
          </div>
        </>
      )}
    </div>
  );
}
