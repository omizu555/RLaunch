/* ============================================================
   ContextMenu - 右クリックメニュー
   ============================================================ */
import "./ContextMenu.css";
import type { GridCell, LauncherItem, GroupItem } from "../types";
import { isGroupItem } from "../types";

export interface MenuPosition {
  x: number;
  y: number;
}

interface ContextMenuProps {
  pos: MenuPosition;
  cell: GridCell;
  onClose: () => void;
  onDelete: () => void;
  onAddWidget: () => void;
  onWidgetSettings?: () => void;
  onLaunch?: (cell: GridCell) => void;
  onLaunchAdmin?: (cell: GridCell) => void;
  onOpenLocation?: (cell: GridCell) => void;
  onBrowseFolder?: (path: string) => void;
  onToggleFolderAction?: (cell: LauncherItem) => void;
  onEditItem?: (item: LauncherItem) => void;
  onCreateGroup?: () => void;
  onEditGroup?: (group: GroupItem) => void;
  /** P-10: ファイル選択ダイアログでの登録 */
  onFilePickRegister?: () => void;
  /** フォルダ選択ダイアログでの登録 */
  onFolderPickRegister?: () => void;
  /** P-08: URL手動登録 */
  onRegisterUrl?: () => void;
}

export function ContextMenu({
  pos,
  cell,
  onClose,
  onDelete,
  onAddWidget,
  onWidgetSettings,
  onLaunch,
  onLaunchAdmin,
  onOpenLocation,
  onBrowseFolder,
  onToggleFolderAction,
  onEditItem,
  onCreateGroup,
  onEditGroup,
  onFilePickRegister,
  onFolderPickRegister,
  onRegisterUrl,
}: ContextMenuProps) {
  // 空ボタン
  if (!cell) {
    return (
      <ContextMenuWrapper pos={pos} onClose={onClose}>
        <div className="context-menu-item" onClick={onClose}>
          ➕ アイテム登録（ファイルをD&D）
        </div>
        {onFilePickRegister && (
          <div className="context-menu-item" onClick={() => { onFilePickRegister(); onClose(); }}>
            📁 ファイルを選択して追加
          </div>
        )}
        {onFolderPickRegister && (
          <div className="context-menu-item" onClick={() => { onFolderPickRegister(); onClose(); }}>
            📂 フォルダを選択して追加
          </div>
        )}
        {onRegisterUrl && (
          <div className="context-menu-item" onClick={() => { onRegisterUrl(); onClose(); }}>
            🌐 URLを登録
          </div>
        )}
        <div className="context-menu-separator" />
        <div className="context-menu-item" onClick={() => { onAddWidget(); onClose(); }}>
          🕐 ウィジェットを配置
        </div>
        <div className="context-menu-item" onClick={() => { onCreateGroup?.(); onClose(); }}>
          📁 サブグループを作成
        </div>
      </ContextMenuWrapper>
    );
  }

  // ウィジェット
  if (cell.type === "widget") {
    return (
      <ContextMenuWrapper pos={pos} onClose={onClose}>
        <div className="context-menu-item" onClick={() => { onWidgetSettings?.(); onClose(); }}>
          ⚙ ウィジェット設定
        </div>
        <div className="context-menu-item" onClick={() => { onAddWidget(); onClose(); }}>
          🔄 ウィジェットを変更
        </div>
        <div className="context-menu-separator" />
        <div className="context-menu-item danger" onClick={onDelete}>
          🗑 ウィジェットを解除
        </div>
      </ContextMenuWrapper>
    );
  }

  // グループ
  if (isGroupItem(cell)) {
    return (
      <ContextMenuWrapper pos={pos} onClose={onClose}>
        <div className="context-menu-item" onClick={() => { onEditGroup?.(cell); onClose(); }}>
          ✏ グループ名を変更
        </div>
        <div className="context-menu-separator" />
        <div className="context-menu-item danger" onClick={onDelete}>
          🗑 グループを削除
        </div>
      </ContextMenuWrapper>
    );
  }

  // アプリボタン
  const item: LauncherItem = cell as LauncherItem;
  const isFolder = item.type === "folder";
  const folderAction = item.folderAction ?? "open";
  return (
    <ContextMenuWrapper pos={pos} onClose={onClose}>
      <div className="context-menu-item" onClick={() => { onLaunch?.(cell); onClose(); }}>
        ▶ 起動
      </div>
      {isFolder && (
        <div className="context-menu-item" onClick={() => { onBrowseFolder?.(item.path); onClose(); }}>
          📂 フォルダを参照
        </div>
      )}
      <div className="context-menu-item" onClick={() => { onLaunchAdmin?.(cell); onClose(); }}>
        🛡 管理者として起動
      </div>
      <div className="context-menu-separator" />
      {isFolder && (
        <>
          <div className="context-menu-item" onClick={() => { onToggleFolderAction?.(item); onClose(); }}>
            {folderAction === "open" ? "🔄 クリック動作: 開く → 参照に変更" : "🔄 クリック動作: 参照 → 開くに変更"}
          </div>
          <div className="context-menu-separator" />
        </>
      )}
      <div className="context-menu-item" onClick={() => { onOpenLocation?.(cell); onClose(); }}>
        📂 ファイルの場所を開く
      </div>
      <div className="context-menu-item" onClick={() => { onEditItem?.(item); onClose(); }}>
        ✏ 編集
      </div>
      <div className="context-menu-item context-menu-info">
        📋 {item.path}
      </div>
      <div className="context-menu-separator" />
      <div className="context-menu-item danger" onClick={() => {
        if (window.confirm(`「${item.label}」の登録を解除しますか？`)) {
          onDelete();
        } else {
          onClose();
        }
      }}>
        🗑 登録解除
      </div>
    </ContextMenuWrapper>
  );
}

function ContextMenuWrapper({
  pos,
  onClose,
  children,
}: {
  pos: MenuPosition;
  onClose: () => void;
  children: React.ReactNode;
}) {
  // メニューが画面外に出ないよう位置を調整
  const adjustedX = Math.min(pos.x, window.innerWidth - 200);
  const adjustedY = Math.min(pos.y, window.innerHeight - 200);

  return (
    <>
      <div className="context-menu-overlay" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose(); }} />
      <div
        className="context-menu"
        role="menu"
        style={{ left: adjustedX, top: adjustedY }}
      >
        {children}
      </div>
    </>
  );
}
