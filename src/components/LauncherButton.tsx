/* ============================================================
   LauncherButton - 個別ボタン
   ============================================================ */
import type { GridCell, LauncherItem } from "../types";
import { isWidgetItem, isGroupItem } from "../types";
import { WidgetRenderer } from "../widgets/WidgetRenderer";

/** グループのデフォルトフォルダアイコン (SVG data URL) */
export const DEFAULT_GROUP_ICON = `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" fill="%23fbbf24"/></svg>')}`;

/** 起動統計を含むリッチなツールチップを生成 */
function buildTooltip(item: LauncherItem): string {
  const lines: string[] = [item.path];
  if (item.args) lines.push(`引数: ${item.args}`);
  if (item.workingDir) lines.push(`作業Dir: ${item.workingDir}`);
  if (item.launchCount) {
    lines.push(`起動回数: ${item.launchCount}回`);
  }
  if (item.lastLaunchedAt) {
    const d = new Date(item.lastLaunchedAt);
    lines.push(`最終起動: ${d.toLocaleDateString("ja-JP")} ${d.toLocaleTimeString("ja-JP", { hour: "2-digit", minute: "2-digit" })}`);
  }
  return lines.join("\n");
}

interface LauncherButtonProps {
  cell: GridCell;
  index: number;
  showLabels?: boolean;
  isDragSource?: boolean;
  isDragOver?: boolean;
  isFocused?: boolean;
  invalidPath?: boolean;
  onContextMenu: (e: React.MouseEvent, index: number, cell: GridCell) => void;
  onClick: (index: number, cell: GridCell) => void;
  onPointerDown?: (e: React.PointerEvent, index: number) => void;
}

export function LauncherButton({
  cell,
  index,
  showLabels = true,
  isDragSource = false,
  isDragOver = false,
  isFocused = false,
  invalidPath = false,
  onContextMenu,
  onClick,
  onPointerDown,
}: LauncherButtonProps) {

  // 空ボタン
  if (!cell) {
    return (
      <div
        className={`btn empty ${isDragOver ? "drag-over" : ""} ${isFocused ? "focused" : ""}`}
        data-cell-index={index}
        role="gridcell"
        aria-label="空きスロット"
        onContextMenu={(e) => onContextMenu(e, index, null)}
      />
    );
  }

  // ウィジェットボタン — Canvas 描画
  if (isWidgetItem(cell)) {
    return (
      <div
        className={`btn widget ${isDragOver ? "drag-over" : ""} ${isDragSource ? "dragging" : ""} ${isFocused ? "focused" : ""}`}
        data-cell-index={index}
        role="gridcell"
        aria-label={`ウィジェット: ${cell.label ?? cell.widgetType}`}
        onPointerDown={(e) => onPointerDown?.(e, index)}
        onContextMenu={(e) => onContextMenu(e, index, cell)}
        onClick={() => onClick(index, cell)}
        title={cell.label ?? cell.widgetType}
      >
        <WidgetRenderer widget={cell} />
      </div>
    );
  }

  // グループボタン — サブフォルダ的な存在
  if (isGroupItem(cell)) {
    const childCount = cell.items.filter(Boolean).length;
    return (
      <div
        className={`btn group-btn ${isDragOver ? "drag-over" : ""} ${isDragSource ? "dragging" : ""} ${isFocused ? "focused" : ""}`}
        data-cell-index={index}
        role="gridcell"
        aria-label={`グループ: ${cell.label} (${childCount} アイテム)`}
        onPointerDown={(e) => onPointerDown?.(e, index)}
        onContextMenu={(e) => onContextMenu(e, index, cell)}
        onClick={() => onClick(index, cell)}
        title={`${cell.label} (${childCount} アイテム)`}
      >
        <div className="btn-icon">
          {cell.iconBase64 ? (
            <img src={cell.iconBase64.startsWith('data:') ? cell.iconBase64 : `data:image/png;base64,${cell.iconBase64}`} alt={cell.label} draggable={false} />
          ) : cell.icon && cell.icon !== "📂" ? (
            <span className="group-icon" style={cell.iconColor ? { color: cell.iconColor } : undefined}>{cell.icon}</span>
          ) : (
            <img src={DEFAULT_GROUP_ICON} alt={cell.label} draggable={false} />
          )}
        </div>
        {showLabels && <div className="btn-label">{cell.label}</div>}
        {childCount > 0 && <span className="group-badge">{childCount}</span>}
      </div>
    );
  }

  // アプリボタン
  const item: LauncherItem = cell;
  return (
    <div
      className={`btn ${isDragOver ? "drag-over" : ""} ${isDragSource ? "dragging" : ""} ${isFocused ? "focused" : ""}`}
      data-cell-index={index}
      role="gridcell"
      aria-label={`${item.label} (${item.type})`}
      onPointerDown={(e) => onPointerDown?.(e, index)}
      onContextMenu={(e) => onContextMenu(e, index, cell)}
      onClick={() => onClick(index, cell)}
      title={buildTooltip(item)}
    >
      <div className="btn-icon">
        {item.iconBase64 ? (
          <img src={item.iconBase64.startsWith('data:') ? item.iconBase64 : `data:image/png;base64,${item.iconBase64}`} alt={item.label} draggable={false} />
        ) : (
          <span className="emoji">{getTypeEmoji(item.type)}</span>
        )}
      </div>
      {showLabels && <div className="btn-label">{item.label}</div>}
      {invalidPath && <span className="path-warning" title="パスが見つかりません">⚠</span>}
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
