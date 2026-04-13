/* ============================================================
   LauncherGrid - ボタングリッド
   ============================================================ */
import "./LauncherGrid.css";
import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import type { Tab, GridCell, LauncherItem, GroupItem } from "../types";
import { isWidgetItem, isGroupItem } from "../types";
import { LauncherButton, DEFAULT_GROUP_ICON } from "./LauncherButton";
import { ContextMenu, type MenuPosition } from "./ContextMenu";

interface LauncherGridProps {
  tab: Tab;
  showLabels?: boolean;
  /** list=コンパクト横長セル, grid=正方形セル */
  viewMode?: "grid" | "list";
  /** リスト表示時の列数 (1-4) */
  listColumns?: number;
  onCellClick: (index: number, cell: GridCell) => void;
  onCellClear: (index: number) => void;
  onCellSwap: (fromIndex: number, toIndex: number) => void;
  onAddWidget: (index: number) => void;
  onWidgetSettings?: (index: number) => void;
  onLaunch?: (cell: GridCell) => void;
  onLaunchAdmin?: (cell: GridCell) => void;
  onOpenLocation?: (cell: GridCell) => void;
  onBrowseFolder?: (path: string) => void;
  onCellUpdate?: (index: number, item: GridCell) => void;
  onEditItem?: (index: number, item: LauncherItem) => void;
  onCreateGroup?: (index: number) => void;
  onEditGroup?: (index: number, group: GroupItem) => void;
  externalDragOverIndex?: number | null;
  /** P-10: ファイル選択ダイアログ登録 */
  onFilePickRegister?: (index: number) => void;
  /** フォルダ選択ダイアログ登録 */
  onFolderPickRegister?: (index: number) => void;
  /** P-08: URL登録 */
  onRegisterUrl?: (index: number) => void;
  /** P-12: ドラッグ中フラグ（タブ切替用） */
  onDragStateChange?: (isDragging: boolean) => void;
  /** P-38: 無効パスのアイテムIDセット */
  invalidPaths?: Set<string>;
}

/** ドラッグ開始とみなす移動距離しきい値 (px) */
const DRAG_THRESHOLD = 5;

export function LauncherGrid({
  tab,
  showLabels = true,
  viewMode = "grid",
  listColumns,
  onCellClick,
  onCellClear,
  onCellSwap,
  onAddWidget,
  onWidgetSettings,
  onLaunch,
  onLaunchAdmin,
  onOpenLocation,
  onBrowseFolder,
  onCellUpdate,
  onEditItem,
  onCreateGroup,
  onEditGroup,
  externalDragOverIndex,
  onFilePickRegister,
  onFolderPickRegister,
  onRegisterUrl,
  onDragStateChange,
  invalidPaths,
}: LauncherGridProps) {
  const [menu, setMenu] = useState<{ pos: MenuPosition; index: number; cell: GridCell } | null>(null);

  // ── キーボードナビゲーション ──
  const [focusedIndex, setFocusedIndex] = useState<number | null>(null);
  const gridRef = useRef<HTMLDivElement>(null);

  // タブ切替時にフォーカスリセット
  useEffect(() => {
    setFocusedIndex(null);
  }, [tab.id]);

  /** キーボード操作 */
  const handleGridKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const cols = viewMode === "list" ? (listColumns ?? 1) : tab.gridColumns;
      const total = tab.gridColumns * tab.gridRows;
      let idx = focusedIndex ?? 0;

      switch (e.key) {
        case "ArrowRight":
          e.preventDefault();
          idx = (idx + 1) % total;
          break;
        case "ArrowLeft":
          e.preventDefault();
          idx = (idx - 1 + total) % total;
          break;
        case "ArrowDown":
          e.preventDefault();
          idx = (idx + cols) % total;
          break;
        case "ArrowUp":
          e.preventDefault();
          idx = (idx - cols + total) % total;
          break;
        case "Enter":
          e.preventDefault();
          if (focusedIndex !== null) {
            const cell = tab.items[focusedIndex] ?? null;
            onCellClick(focusedIndex, cell);
          }
          return;
        case "Delete":
          e.preventDefault();
          if (focusedIndex !== null && tab.items[focusedIndex]) {
            onCellClear(focusedIndex);
          }
          return;
        default:
          return;
      }
      setFocusedIndex(idx);
    },
    [focusedIndex, tab.gridColumns, tab.gridRows, tab.items, onCellClick, onCellClear, viewMode, listColumns],
  );

  // ── ポインタベース D&D（HTML5 DnD は Tauri ネイティブ D&D と競合するため使わない） ──
  const pointerDrag = useRef<{
    sourceIndex: number;
    startX: number;
    startY: number;
    activated: boolean;
  } | null>(null);
  const pointerTargetRef = useRef<number | null>(null);
  const justDragged = useRef(false);
  const onCellSwapRef = useRef(onCellSwap);
  onCellSwapRef.current = onCellSwap;

  const [dragSource, setDragSource] = useState<number | null>(null);
  const [dragTarget, setDragTarget] = useState<number | null>(null);
  // P-11: ドラッグゴースト位置
  const [ghostPos, setGhostPos] = useState<{ x: number; y: number } | null>(null);

  /** セル上でポインタが押された → ドラッグ候補として記録 */
  const handleCellPointerDown = useCallback((e: React.PointerEvent, index: number) => {
    if (e.button !== 0) return;
    pointerDrag.current = {
      sourceIndex: index,
      startX: e.clientX,
      startY: e.clientY,
      activated: false,
    };
  }, []);

  /** グローバルポインタイベントでドラッグ＆ドロップ処理 */
  useEffect(() => {
    const handleMove = (e: PointerEvent) => {
      const drag = pointerDrag.current;
      if (!drag) return;

      if (!drag.activated) {
        const dx = e.clientX - drag.startX;
        const dy = e.clientY - drag.startY;
        if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
        drag.activated = true;
        setDragSource(drag.sourceIndex);
        onDragStateChange?.(true);
      }

      // P-11: ゴースト位置を更新
      setGhostPos({ x: e.clientX, y: e.clientY });

      const el = document.elementFromPoint(e.clientX, e.clientY);
      const btn = el?.closest("[data-cell-index]");
      const idx = btn ? parseInt(btn.getAttribute("data-cell-index") || "", 10) : NaN;
      const target = !isNaN(idx) ? idx : null;

      if (target !== pointerTargetRef.current) {
        pointerTargetRef.current = target;
        setDragTarget(target);
      }
    };

    const handleUp = () => {
      const drag = pointerDrag.current;
      if (drag?.activated) {
        const target = pointerTargetRef.current;
        if (target !== null && target !== drag.sourceIndex) {
          onCellSwapRef.current(drag.sourceIndex, target);
        }
        justDragged.current = true;
        requestAnimationFrame(() => { justDragged.current = false; });
        onDragStateChange?.(false);
      }
      pointerDrag.current = null;
      pointerTargetRef.current = null;
      setDragSource(null);
      setDragTarget(null);
      setGhostPos(null);
    };

    document.addEventListener("pointermove", handleMove);
    document.addEventListener("pointerup", handleUp);
    return () => {
      document.removeEventListener("pointermove", handleMove);
      document.removeEventListener("pointerup", handleUp);
    };
  }, []);

  const handleContextMenu = (e: React.MouseEvent, index: number, cell: GridCell) => {
    e.preventDefault();
    setMenu({ pos: { x: e.clientX, y: e.clientY }, index, cell });
  };

  /** ドラッグ直後のクリック発火を抑制するラッパー */
  const handleCellClick = useCallback(
    (index: number, cell: GridCell) => {
      if (justDragged.current) return;
      onCellClick(index, cell);
    },
    [onCellClick],
  );

  /** 空セルダブルクリック → ファイル選択ダイアログ */
  const handleCellDoubleClick = useCallback(
    (index: number, cell: GridCell) => {
      if (!cell && onFilePickRegister) {
        onFilePickRegister(index);
      }
    },
    [onFilePickRegister],
  );

  const displayDragOver = externalDragOverIndex ?? dragTarget;

  const totalCells = tab.gridColumns * tab.gridRows;
  const cells: GridCell[] = [];
  for (let i = 0; i < totalCells; i++) {
    cells.push(tab.items[i] ?? null);
  }

  // P-30: スパンされたウィジェットがカバーするセルインデックスを計算
  const coveredCells = useMemo(() => {
    const covered = new Set<number>();
    const cols = tab.gridColumns;
    for (let i = 0; i < totalCells; i++) {
      const cell = cells[i];
      if (isWidgetItem(cell)) {
        const cs = cell.colSpan ?? 1;
        const rs = cell.rowSpan ?? 1;
        if (cs > 1 || rs > 1) {
          const startCol = i % cols;
          const startRow = Math.floor(i / cols);
          for (let dr = 0; dr < rs; dr++) {
            for (let dc = 0; dc < cs; dc++) {
              if (dr === 0 && dc === 0) continue;
              const ci = (startRow + dr) * cols + (startCol + dc);
              if (startCol + dc < cols && ci < totalCells) {
                covered.add(ci);
              }
            }
          }
        }
      }
    }
    return covered;
  }, [cells, tab.gridColumns, totalCells]);

  const isCompact = viewMode === "list";
  const effectiveCols = isCompact ? (listColumns ?? 1) : tab.gridColumns;

  return (
    <div className="grid-area">
      <div
        ref={gridRef}
        className={`grid ${isCompact ? "grid-compact" : ""}`}
        role="grid"
        aria-rowcount={tab.gridRows}
        aria-colcount={effectiveCols}
        tabIndex={0}
        onKeyDown={handleGridKeyDown}
        onDragOver={(e) => e.preventDefault()}
        style={{
          "--grid-cols": effectiveCols,
          "--grid-rows": isCompact ? Math.ceil(totalCells / effectiveCols) : tab.gridRows,
        } as React.CSSProperties}
      >
        {cells.map((cell, i) => {
          // P-30: スパンウィジェットにカバーされたセルはスキップ
          if (coveredCells.has(i)) return null;
          const isSpanWidget = isWidgetItem(cell) && (
            (cell.colSpan ?? 1) > 1 || (cell.rowSpan ?? 1) > 1
          );
          const spanStyle: React.CSSProperties = isSpanWidget && isWidgetItem(cell) ? {
            gridColumn: `span ${cell.colSpan ?? 1}`,
            gridRow: `span ${cell.rowSpan ?? 1}`,
          } : {};
          return (
            <div key={i} style={spanStyle} className={isSpanWidget ? "span-cell" : undefined}>
              <LauncherButton
                cell={cell}
                index={i}
                showLabels={showLabels}
                compact={isCompact}
                isDragSource={dragSource === i}
                isDragOver={displayDragOver === i}
                isFocused={focusedIndex === i}
                invalidPath={!!(cell && invalidPaths?.has(cell.id))}
                onContextMenu={handleContextMenu}
                onClick={handleCellClick}
                onDoubleClick={handleCellDoubleClick}
                onPointerDown={handleCellPointerDown}
              />
            </div>
          );
        })}
      </div>

      {menu && (
        <ContextMenu
          pos={menu.pos}
          cell={menu.cell}
          onClose={() => setMenu(null)}
          onDelete={() => {
            onCellClear(menu.index);
            setMenu(null);
          }}
          onAddWidget={() => {
            onAddWidget(menu.index);
            setMenu(null);
          }}
          onWidgetSettings={() => {
            onWidgetSettings?.(menu.index);
            setMenu(null);
          }}
          onLaunch={onLaunch}
          onLaunchAdmin={onLaunchAdmin}
          onOpenLocation={onOpenLocation}
          onBrowseFolder={onBrowseFolder}
          onToggleFolderAction={(item: LauncherItem) => {
            if (menu) {
              const toggled = { ...item, folderAction: item.folderAction === "browse" ? "open" as const : "browse" as const };
              onCellUpdate?.(menu.index, toggled);
            }
          }}
          onEditItem={onEditItem ? (item: LauncherItem) => {
            if (menu) {
              onEditItem(menu.index, item);
              setMenu(null);
            }
          } : undefined}
          onCreateGroup={onCreateGroup ? () => {
            if (menu) {
              onCreateGroup(menu.index);
              setMenu(null);
            }
          } : undefined}
          onEditGroup={onEditGroup ? (group: GroupItem) => {
            if (menu) {
              onEditGroup(menu.index, group);
              setMenu(null);
            }
          } : undefined}
          onFilePickRegister={onFilePickRegister ? () => {
            if (menu) {
              onFilePickRegister(menu.index);
              setMenu(null);
            }
          } : undefined}
          onFolderPickRegister={onFolderPickRegister ? () => {
            if (menu) {
              onFolderPickRegister(menu.index);
              setMenu(null);
            }
          } : undefined}
          onRegisterUrl={onRegisterUrl ? () => {
            if (menu) {
              onRegisterUrl(menu.index);
              setMenu(null);
            }
          } : undefined}
        />
      )}

      {/* P-11: ドラッグゴースト */}
      {dragSource !== null && ghostPos && (() => {
        const dragCell = tab.items[dragSource] ?? null;
        if (!dragCell) return null;
        let label: string;
        let icon: string | undefined;
        let iconBase64: string | undefined;
        if (isGroupItem(dragCell)) {
          label = dragCell.label;
          icon = undefined;
          iconBase64 = dragCell.iconBase64;
        } else if (isWidgetItem(dragCell)) {
          label = dragCell.label ?? dragCell.widgetType;
          icon = "🕐";
          iconBase64 = undefined;
        } else {
          label = dragCell.label;
          icon = dragCell.iconBase64 ? undefined : getTypeEmoji(dragCell.type);
          iconBase64 = dragCell.iconBase64;
        }
        return (
          <div
            className="drag-ghost"
            style={{ left: ghostPos.x + 12, top: ghostPos.y + 12 }}
          >
            {iconBase64 ? (
              <img src={iconBase64.startsWith('data:') ? iconBase64 : `data:image/png;base64,${iconBase64}`} alt="" draggable={false} />
            ) : icon ? (
              <span>{icon}</span>
            ) : dragCell.type === "group" ? (
              <img src={DEFAULT_GROUP_ICON} alt="" draggable={false} />
            ) : (
              <span>📦</span>
            )}
            <span className="drag-ghost-label">{label}</span>
          </div>
        );
      })()}
    </div>
  );
}

function getTypeEmoji(type: string): string {
  switch (type) {
    case "executable": return "📦";
    case "shortcut": return "🔗";
    case "folder": return "📁";
    case "url": return "🌐";
    case "document": return "📄";
    default: return "❓";
  }
}
