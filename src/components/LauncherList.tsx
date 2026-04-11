/* ============================================================
   LauncherList - テキスト優先コンパクトリスト表示
   CLaunch のリストビューに着想を得た、行単位の表示モード
   ============================================================ */
import "./LauncherList.css";
import { useCallback, useRef } from "react";
import type { Tab, GridCell, LauncherItem, GroupItem } from "../types";
import { isWidgetItem, isGroupItem, isLauncherItem } from "../types";
import { DEFAULT_GROUP_ICON } from "./LauncherButton";

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
  onLaunch?: (cell: GridCell) => void;
  onLaunchAdmin?: (cell: GridCell) => void;
  onOpenLocation?: (cell: GridCell) => void;
  onEditItem?: (index: number, item: LauncherItem) => void;
  onEditGroup?: (index: number, group: GroupItem) => void;
  onContextMenu?: (e: React.MouseEvent, index: number, cell: GridCell) => void;
  invalidPaths?: Set<string>;
}

export function LauncherList({
  tab,
  onCellClick,
  onCellSwap,
  onLaunch,
  onEditItem,
  onEditGroup,
  invalidPaths,
}: LauncherListProps) {
  const dragSource = useRef<number | null>(null);
  const dragOver = useRef<number | null>(null);

  const total = tab.gridColumns * tab.gridRows;
  const items = tab.items.slice(0, total);

  // 非空セルだけをフィルタリングして表示
  const nonEmptyCells: { index: number; cell: NonNullable<GridCell> }[] = [];
  for (let i = 0; i < items.length; i++) {
    const cell = items[i];
    if (cell) nonEmptyCells.push({ index: i, cell });
  }

  const handleClick = useCallback(
    (index: number, cell: GridCell) => {
      if (onLaunch) {
        onLaunch(cell);
      } else {
        onCellClick(index, cell);
      }
    },
    [onCellClick, onLaunch]
  );

  const handleDoubleClick = useCallback(
    (index: number, cell: GridCell) => {
      if (isLauncherItem(cell)) {
        onEditItem?.(index, cell);
      } else if (isGroupItem(cell)) {
        onEditGroup?.(index, cell);
      }
    },
    [onEditItem, onEditGroup]
  );

  const handleDragStart = useCallback((e: React.DragEvent, index: number) => {
    dragSource.current = index;
    e.dataTransfer.effectAllowed = "move";
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, index: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    dragOver.current = index;
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent, toIndex: number) => {
      e.preventDefault();
      if (dragSource.current !== null && dragSource.current !== toIndex) {
        onCellSwap(dragSource.current, toIndex);
      }
      dragSource.current = null;
      dragOver.current = null;
    },
    [onCellSwap]
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, index: number, cell: GridCell) => {
      e.preventDefault();
      // コンテキストメニューはグリッドモードと同じイベントを発火
      handleClick(index, cell);
    },
    [handleClick]
  );

  return (
    <div className="launcher-list">
      {nonEmptyCells.length === 0 && (
        <div className="list-empty">アイテムが登録されていません</div>
      )}
      {nonEmptyCells.map(({ index, cell }) => {
        const isInvalid = cell && "id" in cell && invalidPaths?.has(cell.id);

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
            className={`list-row${isInvalid ? " invalid" : ""}`}
            draggable
            onClick={() => handleClick(index, cell)}
            onDoubleClick={() => handleDoubleClick(index, cell)}
            onContextMenu={(e) => handleContextMenu(e, index, cell)}
            onDragStart={(e) => handleDragStart(e, index)}
            onDragOver={(e) => handleDragOver(e, index)}
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
            {isInvalid && <span className="list-row-warning" title="パスが見つかりません">⚠</span>}
          </div>
        );
      })}
    </div>
  );
}
