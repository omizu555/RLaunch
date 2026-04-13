/* ============================================================
   GroupPopupWindow - 独立ウィンドウ版グループポップアップ
   グループボタンをクリックすると独立ウィンドウとして表示される
   ミニランチャー。メイングリッドと同等の登録・操作を提供。

   機能:
   - アイテムクリックで起動
   - ネイティブファイル D&D でアイテム登録（複数ファイル対応）
   - 空セル右クリック → ファイル選択 / URL登録
   - 既存アイテム右クリック → 起動 / 管理者起動 / 編集 / 除去 等
   - LauncherButton コンポーネントで描画（メイングリッドと統一）
   - ItemEditDialog でアイテム編集

   イベントフロー:
   1. 子ウィンドウ起動 → "group-popup-ready" emit
   2. 親が "group-popup-init" で GroupItem データを送信
   3. アイテムクリック → invoke("launch_app") + "group-popup-launch" → 閉じ
   4. グループ変更 → "group-popup-update" (updated GroupItem)
   5. ウィンドウ閉じ → "group-popup-closed"
   6. フォルダ参照 → "group-popup-action" (親に委任)
   ============================================================ */
import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import type { GroupItem, GridCell, LauncherItem } from "../types";
import { isLauncherItem } from "../types";
import { createLauncherItemFromPath } from "../utils/fileRegistration";
import { buildCellArray } from "../utils/domHelpers";
import { useNativeDrop } from "../hooks/useNativeDrop";
import { useChildTheme } from "../hooks/useChildTheme";
import { useFocusLossAutoClose } from "../hooks/useFocusLossAutoClose";
import { LauncherButton } from "./LauncherButton";
import { ItemEditDialog } from "./ItemEditDialog";

export interface GroupPopupInitPayload {
  group: GroupItem;
  /** 親（タブ/全体）の表示モード — グループが未設定の場合に使う */
  parentViewMode?: "grid" | "list";
  /** 親（タブ/全体）のリスト列数 — グループが未設定の場合に使う */
  parentListColumns?: number;
}

export interface GroupPopupUpdatePayload {
  group: GroupItem;
}

/** 親ウィンドウへのアクション要求 */
export interface GroupPopupActionPayload {
  action: "browse-folder";
  cellIndex: number;
  path?: string;
}

/** 右クリックメニューのコンテキスト */
interface ContextState {
  index: number;
  cell: GridCell;
  pos: { x: number; y: number };
}

export function GroupPopupWindow() {
  const { refreshTheme } = useChildTheme();
  const [group, setGroup] = useState<GroupItem | null>(null);
  const [context, setContext] = useState<ContextState | null>(null);
  const [editTarget, setEditTarget] = useState<{ index: number; item: LauncherItem } | null>(null);
  const groupRef = useRef<GroupItem | null>(null);
  const [parentViewMode, setParentViewMode] = useState<"grid" | "list">("grid");
  const [parentListColumns, setParentListColumns] = useState(1);

  // P-33: ポインタD&D用ステート
  const pointerDrag = useRef<{
    sourceIndex: number;
    startX: number;
    startY: number;
    activated: boolean;
  } | null>(null);
  const pointerTargetRef = useRef<number | null>(null);
  const justDragged = useRef(false);
  const [dragSource, setDragSource] = useState<number | null>(null);
  const [dragTarget, setDragTarget] = useState<number | null>(null);

  // groupRef を常に最新に
  useEffect(() => { groupRef.current = group; }, [group]);

  // ── ヘルパー: グループ更新 (state + 親通知) ──
  const updateGroup = useCallback((updated: GroupItem) => {
    setGroup(updated);
    emit("group-popup-update", { group: updated });
  }, []);

  // ── ヘルパー: セル更新 ──
  const updateCell = useCallback((cellIndex: number, item: GridCell) => {
    const g = groupRef.current;
    if (!g) return;
    const newItems = [...g.items];
    while (newItems.length <= cellIndex) newItems.push(null);
    newItems[cellIndex] = item;
    updateGroup({ ...g, items: newItems, updatedAt: new Date().toISOString() });
  }, [updateGroup]);

  // ── ヘルパー: アイテム登録 ──
  const registerItem = useCallback(async (filePath: string, cellIndex: number) => {
    const g = groupRef.current;
    if (!g) return;
    try {
      const item = await createLauncherItemFromPath(filePath);
      if (!item) return;
      updateCell(cellIndex, item);
    } catch (e) {
      console.error("Registration failed:", e);
    }
  }, [updateCell]);

  // ── ヘルパー: ウィンドウを閉じる（親側で hide 実行するためイベント通知のみ） ──
  const closeWindow = useCallback(() => {
    emit("group-popup-closed");
  }, []);

  // ── P-33: グループ内 D&D アイテム入れ替え ──
  const handleSwap = useCallback((fromIndex: number, toIndex: number) => {
    const g = groupRef.current;
    if (!g) return;
    const newItems = [...g.items];
    while (newItems.length <= Math.max(fromIndex, toIndex)) newItems.push(null);
    const temp = newItems[fromIndex];
    newItems[fromIndex] = newItems[toIndex];
    newItems[toIndex] = temp;
    updateGroup({ ...g, items: newItems, updatedAt: new Date().toISOString() });
  }, [updateGroup]);

  const handleSwapRef = useRef(handleSwap);
  handleSwapRef.current = handleSwap;

  // ── P-33: ポインタD&Dイベントハンドラ ──
  const handleCellPointerDown = useCallback((e: React.PointerEvent, index: number) => {
    if (e.button !== 0) return;
    pointerDrag.current = {
      sourceIndex: index,
      startX: e.clientX,
      startY: e.clientY,
      activated: false,
    };
  }, []);

  useEffect(() => {
    const handleMove = (e: PointerEvent) => {
      const drag = pointerDrag.current;
      if (!drag) return;
      if (!drag.activated) {
        const dx = e.clientX - drag.startX;
        const dy = e.clientY - drag.startY;
        if (Math.abs(dx) < 5 && Math.abs(dy) < 5) return;
        drag.activated = true;
        setDragSource(drag.sourceIndex);
      }
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
          handleSwapRef.current(drag.sourceIndex, target);
        }
        justDragged.current = true;
        requestAnimationFrame(() => { justDragged.current = false; });
      }
      pointerDrag.current = null;
      pointerTargetRef.current = null;
      setDragSource(null);
      setDragTarget(null);
    };
    document.addEventListener("pointermove", handleMove);
    document.addEventListener("pointerup", handleUp);
    return () => {
      document.removeEventListener("pointermove", handleMove);
      document.removeEventListener("pointerup", handleUp);
    };
  }, []);

  // ── 初期化: ready → init ハンドシェイク ──
  useEffect(() => {
    const unlistenInit = listen<GroupPopupInitPayload>("group-popup-init", (event) => {
      setGroup(event.payload.group);
      setParentViewMode(event.payload.parentViewMode ?? "grid");
      setParentListColumns(event.payload.parentListColumns ?? 1);
      setContext(null);
      setEditTarget(null);
      const label = event.payload.group.label;
      getCurrentWebviewWindow().setTitle(`📂 ${label}`).catch((e) => console.warn("Failed to set title:", e));
      refreshTheme();
    });

    emit("group-popup-ready");

    getCurrentWebviewWindow().onCloseRequested((event) => {
      event.preventDefault();
      emit("group-popup-closed");
    });

    return () => { unlistenInit.then((fn) => fn()); };
  }, []);

  // ── ウィンドウ移動検知でフォーカス喪失クローズを抑制 ──
  useFocusLossAutoClose("group-popup-closed");

  // ── Tauri ネイティブ D&D (複数ファイル対応) ──
  const hoverIndex = useNativeDrop(
    useCallback(async (filePaths: string[], cellIndex: number) => {
      const g = groupRef.current;
      if (!g) return;
      const totalSlots = g.gridColumns * g.gridRows;
      let targetIndex = cellIndex;
      for (const fp of filePaths) {
        if (targetIndex >= totalSlots) break;
        // 空きセルを探す
        const items = groupRef.current?.items ?? [];
        while (targetIndex < totalSlots && items[targetIndex]) targetIndex++;
        if (targetIndex >= totalSlots) break;
        await registerItem(fp, targetIndex);
        targetIndex++;
      }
    }, [registerItem])
  );

  // ── キーボード ──
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (editTarget) {
          setEditTarget(null);
        } else if (context !== null) {
          setContext(null);
        } else {
          closeWindow();
        }
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [context, editTarget, closeWindow]);

  // ── コンテキストメニュー外クリックで閉じ ──
  useEffect(() => {
    if (context === null) return;
    const handleClick = () => setContext(null);
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, [context]);

  // ── セルクリック → 起動 ──
  const handleCellClick = useCallback(
    async (_index: number, cell: GridCell) => {
      if (justDragged.current) return;
      if (!isLauncherItem(cell)) return;
      // フォルダの参照モード
      if (cell.type === "folder" && cell.folderAction === "browse") {
        emit("group-popup-action", { action: "browse-folder", cellIndex: _index, path: cell.path } as GroupPopupActionPayload);
        return;
      }
      try {
        await invoke("launch_app", { path: cell.path, args: cell.args ?? null });
        await emit("group-popup-launch", { item: cell });
        closeWindow();
      } catch (e) {
        console.error("Failed to launch:", e);
      }
    },
    [closeWindow],
  );

  // ── 右クリックメニュー表示 ──
  const handleContextMenu = useCallback(
    (e: React.MouseEvent, index: number, cell: GridCell) => {
      e.preventDefault();
      e.stopPropagation();
      setContext({ index, cell, pos: { x: e.clientX, y: e.clientY } });
    },
    [],
  );

  // ── セル除去 ──
  const handleRemoveFromGroup = useCallback(
    (index: number, confirmMsg?: string) => {
      if (confirmMsg && !window.confirm(confirmMsg)) {
        setContext(null);
        return;
      }
      if (!group) return;
      const newItems = [...group.items];
      newItems[index] = null;
      updateGroup({ ...group, items: newItems, updatedAt: new Date().toISOString() });
      setContext(null);
    },
    [group, updateGroup],
  );

  // ── ファイル選択ダイアログで登録 ──
  const handleFilePickRegister = useCallback(
    async (cellIndex: number) => {
      setContext(null);
      try {
        const selected = await dialogOpen({
          multiple: false,
          title: "登録するファイルを選択",
          filters: [
            { name: "実行ファイル", extensions: ["exe", "bat", "cmd", "ps1"] },
            { name: "ショートカット", extensions: ["lnk", "url"] },
            { name: "すべてのファイル", extensions: ["*"] },
          ],
        });
        if (selected) {
          await registerItem(selected as string, cellIndex);
        }
      } catch (e) {
        console.error("File dialog error:", e);
      }
    },
    [registerItem],
  );

  // ── URL登録 ──
  const handleRegisterUrl = useCallback(
    (cellIndex: number) => {
      setContext(null);
      const url = window.prompt("登録するURLを入力してください", "https://");
      if (!url || !url.trim()) return;
      const trimmed = url.trim();
      let label = trimmed;
      try { label = new URL(trimmed).hostname; } catch { /* use raw url */ }
      const item: LauncherItem = {
        id: crypto.randomUUID(),
        label,
        path: trimmed,
        type: "url",
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };
      updateCell(cellIndex, item);
    },
    [updateCell],
  );

  // ── 起動操作 ──
  const handleLaunch = useCallback(
    async (cell: GridCell) => {
      setContext(null);
      if (!isLauncherItem(cell)) return;
      try {
        await invoke("launch_app", { path: cell.path, args: cell.args ?? null });
        await emit("group-popup-launch", { item: cell });
        closeWindow();
      } catch (e) {
        console.error("Failed to launch:", e);
      }
    },
    [closeWindow],
  );

  const handleLaunchAdmin = useCallback(
    async (cell: GridCell) => {
      setContext(null);
      if (!isLauncherItem(cell)) return;
      try {
        await invoke("run_as_admin", { path: cell.path, args: cell.args ?? null });
        closeWindow();
      } catch (e) {
        console.error("Failed to launch as admin:", e);
      }
    },
    [closeWindow],
  );

  const handleOpenLocation = useCallback(
    async (cell: GridCell) => {
      setContext(null);
      if (!isLauncherItem(cell)) return;
      try {
        await invoke("open_file_location", { path: cell.path });
      } catch (e) {
        console.error("Failed to open file location:", e);
      }
    },
    [],
  );

  // ── アイテム編集 ──
  const handleEditItem = useCallback(
    (index: number, item: LauncherItem) => {
      setContext(null);
      setEditTarget({ index, item });
    },
    [],
  );

  const handleEditSave = useCallback(
    (updated: LauncherItem) => {
      if (editTarget) {
        updateCell(editTarget.index, updated);
      }
      setEditTarget(null);
    },
    [editTarget, updateCell],
  );

  // ── ダブルクリック: 空セルはファイル選択 ──
  const handleDoubleClick = useCallback(
    (index: number, cell: GridCell) => {
      if (!cell) handleFilePickRegister(index);
    },
    [handleFilePickRegister],
  );

  if (!group) {
    return (
      <div className="group-popup-window" style={{ padding: 20, color: "var(--text-muted)", textAlign: "center" }}>
        読み込み中...
      </div>
    );
  }

  const cells = buildCellArray(group.items, group.gridColumns, group.gridRows) as GridCell[];
  const effectiveViewMode = group.viewMode ?? parentViewMode;
  const effectiveListColumns = group.listColumns ?? parentListColumns;
  const isCompact = effectiveViewMode === "list";

  return (
    <div className="group-popup-window">
      {/* ── ヘッダー (ドラッグでウィンドウ移動可能) ── */}
      <div className="group-popup-header" data-tauri-drag-region>
        <span data-tauri-drag-region>📂 {group.label}</span>
        <button onClick={closeWindow} title="閉じる (Esc)">✕</button>
      </div>

      {/* ── グリッド / コンパクト表示 ── */}
      <div
        className={`group-popup-grid ${isCompact ? "group-popup-compact" : ""}`}
        style={isCompact ? {
          gridTemplateColumns: `repeat(${effectiveListColumns}, 1fr)`,
        } : {
          gridTemplateColumns: `repeat(${group.gridColumns}, var(--cell-size, 64px))`,
          gridTemplateRows: `repeat(${group.gridRows}, var(--cell-size, 64px))`,
        }}
      >
        {cells.map((cell, i) => (
          <div key={i} style={{ position: "relative" }}
            onDoubleClick={(e) => { e.stopPropagation(); handleDoubleClick(i, cell); }}
          >
            <LauncherButton
              cell={cell}
              index={i}
              showLabels
              compact={isCompact}
              isDragOver={hoverIndex === i || dragTarget === i}
              isDragSource={dragSource === i}
              onContextMenu={handleContextMenu}
              onClick={handleCellClick}
              onPointerDown={handleCellPointerDown}
            />

            {/* 空セル登録ヒント (D&D ホバー時) */}
            {hoverIndex === i && !cell && (
              <div className="group-popup-drop-indicator">
                <span>＋</span>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* ── フッター ── */}
      <div className="group-popup-footer">
        <span style={{ fontSize: 11, color: "var(--text-muted)" }}>
          {cells.filter(Boolean).length} / {cells.length} アイテム
          {" · "}ドロップ / ダブルクリック / 右クリックで登録
        </span>
      </div>

      {/* ── コンテキストメニュー ── */}
      {context && (
        <GroupPopupContextMenu
          context={context}
          onClose={() => setContext(null)}
          onFilePickRegister={handleFilePickRegister}
          onRegisterUrl={handleRegisterUrl}
          onRemove={handleRemoveFromGroup}
          onLaunch={handleLaunch}
          onLaunchAdmin={handleLaunchAdmin}
          onOpenLocation={handleOpenLocation}
          onEditItem={handleEditItem}
        />
      )}

      {/* ── アイテム編集ダイアログ ── */}
      {editTarget && (
        <ItemEditDialog
          item={editTarget.item}
          onSave={handleEditSave}
          onClose={() => setEditTarget(null)}
        />
      )}
    </div>
  );
}

/* ============================================================
   GroupPopupContextMenu - メイングリッドと同等のコンテキストメニュー
   ============================================================ */
function GroupPopupContextMenu({
  context,
  onClose,
  onFilePickRegister,
  onRegisterUrl,
  onRemove,
  onLaunch,
  onLaunchAdmin,
  onOpenLocation,
  onEditItem,
}: {
  context: ContextState;
  onClose: () => void;
  onFilePickRegister: (index: number) => void;
  onRegisterUrl: (index: number) => void;
  onRemove: (index: number, confirmMsg?: string) => void;
  onLaunch: (cell: GridCell) => void;
  onLaunchAdmin: (cell: GridCell) => void;
  onOpenLocation: (cell: GridCell) => void;
  onEditItem: (index: number, item: LauncherItem) => void;
}) {
  const { index, cell, pos } = context;
  const adjustedX = Math.min(pos.x, window.innerWidth - 200);
  const adjustedY = Math.min(pos.y, window.innerHeight - 200);

  // 空セルメニュー
  if (!cell) {
    return (
      <>
        <div className="context-menu-overlay" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose(); }} />
        <div className="context-menu" role="menu" style={{ left: adjustedX, top: adjustedY }}>
          <div className="context-menu-item" onClick={onClose}>
            ➕ アイテム登録（ファイルをD&D）
          </div>
          <div className="context-menu-item" onClick={(e) => { e.stopPropagation(); onFilePickRegister(index); }}>
            📁 ファイルを選択して追加
          </div>
          <div className="context-menu-item" onClick={(e) => { e.stopPropagation(); onRegisterUrl(index); }}>
            🌐 URLを登録
          </div>
        </div>
      </>
    );
  }

  // ランチャーアイテムメニュー
  if (isLauncherItem(cell)) {
    return (
      <>
        <div className="context-menu-overlay" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose(); }} />
        <div className="context-menu" role="menu" style={{ left: adjustedX, top: adjustedY }}>
          <div className="context-menu-item" onClick={(e) => { e.stopPropagation(); onLaunch(cell); }}>
            ▶ 起動
          </div>
          <div className="context-menu-item" onClick={(e) => { e.stopPropagation(); onLaunchAdmin(cell); }}>
            🛡 管理者として起動
          </div>
          <div className="context-menu-separator" />
          <div className="context-menu-item" onClick={(e) => { e.stopPropagation(); onOpenLocation(cell); }}>
            📂 ファイルの場所を開く
          </div>
          <div className="context-menu-item" onClick={(e) => { e.stopPropagation(); onEditItem(index, cell); }}>
            ✏ 編集
          </div>
          <div className="context-menu-item context-menu-info">
            📋 {cell.path}
          </div>
          <div className="context-menu-separator" />
          <div className="context-menu-item danger" onClick={(e) => {
            e.stopPropagation();
            onRemove(index, `「${cell.label}」の登録を解除しますか？`);
          }}>
            🗑 登録解除
          </div>
        </div>
      </>
    );
  }

  // フォールバック（ウィジェット等）
  return (
    <>
      <div className="context-menu-overlay" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose(); }} />
      <div className="context-menu" role="menu" style={{ left: adjustedX, top: adjustedY }}>
        <div className="context-menu-item danger" onClick={(e) => { e.stopPropagation(); onRemove(index); }}>
          🗑 グループから除去
        </div>
      </div>
    </>
  );
}
