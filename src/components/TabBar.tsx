/* ============================================================
   TabBar - タブバー
   P-03: D&D 並び替え / P-04: カラーマーカー / P-05: 削除フォールバック / P-06: 複製
   ============================================================ */
import "./TabBar.css";
import { useState, useRef, useEffect, useCallback } from "react";
import type { Tab } from "../types";

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
  /** P-12: ドラッグ中のタブホバー通知 */
  isDraggingItem?: boolean;
  /** タブ設定ダイアログを開く */
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
  isDraggingItem,
  onTabSettings,
}: TabBarProps) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [menu, setMenu] = useState<TabMenu | null>(null);
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
  };

  // メニュー外クリックで閉じる
  useEffect(() => {
    if (!menu) return;
    const handler = () => { setMenu(null); };
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
            {/* タブ複製 */}
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
            {/* タブ設定 */}
            {onTabSettings && (
              <div
                className="tab-context-item"
                onClick={() => {
                  onTabSettings(menu.tabId);
                  setMenu(null);
                }}
              >
                ⚙ タブ設定
              </div>
            )}
            {/* タブ削除 */}
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
