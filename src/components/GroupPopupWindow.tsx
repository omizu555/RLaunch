/* ============================================================
   GroupPopupWindow - 独立ウィンドウ版グループポップアップ
   グループボタンをクリックすると独立ウィンドウとして表示される
   ミニランチャー。

   機能:
   - アイテムクリックで起動
   - ネイティブファイル D&D でアイテム登録
   - 空セル右クリック / ダブルクリック → ファイル選択ダイアログで登録
   - 既存セル右クリック → 除去メニュー
   - LauncherButton コンポーネントで描画（メイングリッドと統一）

   イベントフロー:
   1. 子ウィンドウ起動 → "group-popup-ready" emit
   2. 親が "group-popup-init" で GroupItem データを送信
   3. アイテムクリック → invoke("launch_app") + "group-popup-launch" → 閉じ
   4. グループ変更 → "group-popup-update" (updated GroupItem)
   5. ウィンドウ閉じ → "group-popup-closed"
   ============================================================ */
import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import type { GroupItem, GridCell } from "../types";
import { isLauncherItem } from "../types";
import { createLauncherItemFromPath } from "../utils/fileRegistration";
import { buildCellArray } from "../utils/domHelpers";
import { useNativeDrop } from "../hooks/useNativeDrop";
import { useChildTheme } from "../hooks/useChildTheme";
import { useFocusLossAutoClose } from "../hooks/useFocusLossAutoClose";
import { LauncherButton } from "./LauncherButton";

export interface GroupPopupInitPayload {
  group: GroupItem;
}

export interface GroupPopupUpdatePayload {
  group: GroupItem;
}

/** 右クリックメニューのコンテキスト */
interface ContextState {
  index: number;
  type: "item" | "empty";
}

export function GroupPopupWindow() {
  const { refreshTheme } = useChildTheme();
  const [group, setGroup] = useState<GroupItem | null>(null);
  const [context, setContext] = useState<ContextState | null>(null);
  const groupRef = useRef<GroupItem | null>(null);

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

  // ── ヘルパー: アイテム登録 ──
  const registerItem = useCallback(async (filePath: string, cellIndex: number) => {
    const g = groupRef.current;
    if (!g) return;
    try {
      const item = await createLauncherItemFromPath(filePath);
      if (!item) return;
      const newItems = [...g.items];
      while (newItems.length <= cellIndex) newItems.push(null);
      newItems[cellIndex] = item;
      updateGroup({ ...g, items: newItems, updatedAt: new Date().toISOString() });
    } catch (e) {
      console.error("Registration failed:", e);
    }
  }, [updateGroup]);

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
      setContext(null);
      const label = event.payload.group.label;
      getCurrentWebviewWindow().setTitle(`📂 ${label}`).catch((e) => console.warn("Failed to set title:", e));
      // reusable ウィンドウなのでテーマ変更に追従
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

  // ── Tauri ネイティブ D&D (useNativeDrop フック) ──
  const hoverIndex = useNativeDrop(
    useCallback((filePaths: string[], cellIndex: number) => {
      registerItem(filePaths[0], cellIndex);
    }, [registerItem])
  );

  // ── キーボード ──
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (context !== null) {
          setContext(null);
        } else {
          closeWindow();
        }
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [context, closeWindow]);

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
      setContext({ index, type: cell ? "item" : "empty" });
    },
    [],
  );

  // ── グループからアイテムを除去 ──
  const handleRemoveFromGroup = useCallback(
    (index: number) => {
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
            { name: "すべてのファイル", extensions: ["*"] },
            { name: "実行ファイル", extensions: ["exe", "bat", "cmd", "ps1"] },
            { name: "ショートカット", extensions: ["lnk", "url"] },
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

  return (
    <div className="group-popup-window">
      {/* ── ヘッダー (ドラッグでウィンドウ移動可能) ── */}
      <div className="group-popup-header" data-tauri-drag-region>
        <span data-tauri-drag-region>📂 {group.label}</span>
        <button onClick={closeWindow} title="閉じる (Esc)">✕</button>
      </div>

      {/* ── グリッド ── */}
      <div
        className="group-popup-grid"
        style={{
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

            {/* 右クリックメニュー (既存アイテム) */}
            {context?.index === i && context.type === "item" && cell && (
              <GroupPopupContextMenu
                onAction={() => handleRemoveFromGroup(i)}
                actionLabel="🗑 グループから除去"
                onClose={() => setContext(null)}
              />
            )}

            {/* 右クリックメニュー (空セル) */}
            {context?.index === i && context.type === "empty" && !cell && (
              <GroupPopupContextMenu
                onAction={() => handleFilePickRegister(i)}
                actionLabel="📂 ファイルを選択して登録"
                onClose={() => setContext(null)}
              />
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
    </div>
  );
}

/* ============================================================
   GroupPopupContextMenu - グループポップアップ内の簡易メニュー
   ============================================================ */
function GroupPopupContextMenu({
  onAction,
  actionLabel,
  onClose,
}: {
  onAction: () => void;
  actionLabel: string;
  onClose: () => void;
}) {
  return (
    <div className="group-popup-context-menu">
      <div
        className="group-popup-context-item"
        onClick={(e) => { e.stopPropagation(); onAction(); }}
      >
        {actionLabel}
      </div>
      <div
        className="group-popup-context-item muted"
        onClick={(e) => { e.stopPropagation(); onClose(); }}
      >
        閉じる
      </div>
    </div>
  );
}
