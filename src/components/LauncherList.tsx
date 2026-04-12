/* ============================================================
   LauncherList - テキスト優先コンパクトリスト表示
   CLaunch のリストビューに着想を得た、行単位の表示モード
   ============================================================ */
import "./LauncherList.css";
import { useState, useCallback, useRef, useEffect } from "react";
import type { Tab, GridCell, LauncherItem, GroupItem } from "../types";
import { isWidgetItem, isGroupItem } from "../types";
import { DEFAULT_GROUP_ICON } from "./LauncherButton";
import { ContextMenu, type MenuPosition } from "./ContextMenu";

/** アイテムタイプに対応する絵文字 */
function getTypeEmoji(type: string): string {
  switch (type) {
    case "app": return "🚀";
    case "folder": return "📁";
    case "file": return "📄";
    case "url": return "🌐";
    default: return "📎";
  }
}

interface LauncherListProps {
  tab: Tab;
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
  onFilePickRegister?: (index: number) => void;
  onRegisterUrl?: (index: number) => void;
  invalidPaths?: Set<string>;
}

export function LauncherList({
  tab,
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
  onFilePickRegister,
  onRegisterUrl,
  invalidPaths,
}: LauncherListProps) {
  const dragSource = useRef<number | null>(null);
  const [focusIndex, setFocusIndex] = useState(0);
  const [menu, setMenu] = useState<{
    pos: MenuPosition;
    index: number;
    cell: GridCell;
  } | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const total = tab.gridColumns * tab.gridRows;
  const items = tab.items.slice(0, total);

  // 全セル（空含む）をインデックス付きで保持
  const allCells: { index: number; cell: GridCell }[] = [];
  for (let i = 0; i < items.length; i++) {
    allCells.push({ index: i, cell: items[i] ?? null });
  }

  // 非空セルのみ（表示用）
  const nonEmptyCells = allCells.filter((c) => c.cell !== null) as {
    index: number;
    cell: NonNullable<GridCell>;
  }[];

  const handleClick = useCallback(
    (index: number, cell: GridCell) => {
      onCellClick(index, cell);
    },
    [onCellClick],
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, index: number, cell: GridCell) => {
      e.preventDefault();
      setMenu({ pos: { x: e.clientX, y: e.clientY }, index, cell });
    },
    [],
  );

  const handleDragStart = useCallback((e: React.DragEvent, index: number) => {
    dragSource.current = index;
    e.dataTransfer.effectAllowed = "move";
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent, toIndex: number) => {
      e.preventDefault();
      if (dragSource.current !== null && dragSource.current !== toIndex) {
        onCellSwap(dragSource.current, toIndex);
      }
      dragSource.current = null;
    },
    [onCellSwap],
  );

  // キーボードナビゲーション
  useEffect(() => {
    const el = listRef.current;
    if (!el) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (nonEmptyCells.length === 0) return;

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setFocusIndex((prev) => Math.min(prev + 1, nonEmptyCells.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setFocusIndex((prev) => Math.max(prev - 1, 0));
          break;
        case "Home":
          e.preventDefault();
          setFocusIndex(0);
          break;
        case "End":
          e.preventDefault();
          setFocusIndex(nonEmptyCells.length - 1);
          break;
        case "Enter": {
          e.preventDefault();
          const focused = nonEmptyCells[focusIndex];
          if (focused) handleClick(focused.index, focused.cell);
          break;
        }
        case "Delete": {
          e.preventDefault();
          const focused = nonEmptyCells[focusIndex];
          if (focused) {
            onCellClear(focused.index);
          }
          break;
        }
      }
    };

    el.addEventListener("keydown", handleKeyDown);
    return () => el.removeEventListener("keydown", handleKeyDown);
  }, [nonEmptyCells, focusIndex, handleClick, onCellClear]);

  // フォーカスが範囲外になったらクランプ
  useEffect(() => {
    if (focusIndex >= nonEmptyCells.length && nonEmptyCells.length > 0) {
      setFocusIndex(nonEmptyCells.length - 1);
    }
  }, [nonEmptyCells.length, focusIndex]);

  return (
    <div className="launcher-list" ref={listRef} tabIndex={0} role="listbox">
      {nonEmptyCells.length === 0 && (
        <div
          className="list-empty"
          onContextMenu={(e) => handleContextMenu(e, 0, null)}
        >
          アイテムが登録されていません（右クリックで追加）
        </div>
      )}
      {nonEmptyCells.map(({ index, cell }, displayIdx) => {
        const isInvalid = "id" in cell && invalidPaths?.has(cell.id);
        const isFocused = displayIdx === focusIndex;

        let icon: React.ReactNode;
        let label: string;
        let sublabel: string | undefined;

        if (isWidgetItem(cell)) {
          icon = <span className="list-row-emoji">🕐</span>;
          label = cell.label ?? cell.widgetType;
          sublabel = `ウィジェット: ${cell.widgetType}`;
        } else if (isGroupItem(cell)) {
          const iconSrc = cell.iconBase64
            ? `data:image/png;base64,${cell.iconBase64}`
            : DEFAULT_GROUP_ICON;
          icon = <img className="list-row-icon" src={iconSrc} alt="" />;
          label = cell.label;
          sublabel = `グループ (${cell.items.filter(Boolean).length}件)`;
        } else {
          const iconSrc = cell.iconBase64
            ? `data:image/png;base64,${cell.iconBase64}`
            : undefined;
          icon = iconSrc ? (
            <img className="list-row-icon" src={iconSrc} alt="" />
          ) : (
            <span className="list-row-emoji">{getTypeEmoji(cell.type)}</span>
          );
          label = cell.label;
          sublabel = cell.path;
        }

        return (
          <div
            key={index}
            className={`list-row${isInvalid ? " invalid" : ""}${isFocused ? " focused" : ""}`}
            role="option"
            aria-selected={isFocused}
            draggable
            onClick={() => {
              setFocusIndex(displayIdx);
              handleClick(index, cell);
            }}
            onContextMenu={(e) => {
              setFocusIndex(displayIdx);
              handleContextMenu(e, index, cell);
            }}
            onDragStart={(e) => handleDragStart(e, index)}
            onDragOver={(e) => handleDragOver(e)}
            onDrop={(e) => handleDrop(e, index)}
          >
            <div className="list-row-icon-area">{icon}</div>
            <div className="list-row-text">
              <span className="list-row-label">{label}</span>
              {sublabel && (
                <span className="list-row-sublabel" title={sublabel}>
                  {sublabel}
                </span>
              )}
            </div>
            {isInvalid && (
              <span className="list-row-warning" title="パスが見つかりません">
                ⚠
              </span>
            )}
          </div>
        );
      })}

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
          onToggleFolderAction={
            onCellUpdate
              ? (item: LauncherItem) => {
                  const toggled = {
                    ...item,
                    folderAction:
                      item.folderAction === "browse"
                        ? ("open" as const)
                        : ("browse" as const),
                  };
                  onCellUpdate(menu.index, toggled);
                }
              : undefined
          }
          onEditItem={
            onEditItem
              ? (item: LauncherItem) => {
                  onEditItem(menu.index, item);
                  setMenu(null);
                }
              : undefined
          }
          onCreateGroup={
            onCreateGroup
              ? () => {
                  onCreateGroup(menu.index);
                  setMenu(null);
                }
              : undefined
          }
          onEditGroup={
            onEditGroup
              ? (group: GroupItem) => {
                  onEditGroup(menu.index, group);
                  setMenu(null);
                }
              : undefined
          }
          onFilePickRegister={
            onFilePickRegister
              ? () => {
                  onFilePickRegister(menu.index);
                  setMenu(null);
                }
              : undefined
          }
          onRegisterUrl={
            onRegisterUrl
              ? () => {
                  onRegisterUrl(menu.index);
                  setMenu(null);
                }
              : undefined
          }
        />
      )}
    </div>
  );
}
