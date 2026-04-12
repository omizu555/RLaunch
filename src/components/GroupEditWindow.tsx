/* ============================================================
   GroupEditWindow - 独立ウィンドウ版グループ編集ダイアログ
   useChildWindow パターンで親ウィンドウとイベント通信する。

   イベントフロー:
   1. 子ウィンドウ起動 → "group-edit-ready" emit
   2. 親が "group-edit-init" で初期データを送信
   3. 保存 → "group-edit-save" (label, columns, rows) → 子ウィンドウ閉じ
   4. ウィンドウ閉じ → "group-edit-closed"
   ============================================================ */
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useChildTheme } from "../hooks/useChildTheme";

interface IconInfo {
  filename: string;
  data_url: string;
}

export interface GroupEditInitPayload {
  mode: "create" | "rename";
  label: string;
  columns: number;
  rows: number;
  icon?: string;
  iconColor?: string;
  iconBase64?: string;
  libraryIcon?: string;
  viewMode?: "grid" | "list";
  listColumns?: number;
  /** 親（タブ/全体）から継承する表示モード */
  parentViewMode?: "grid" | "list";
  /** 親（タブ/全体）から継承するリスト列数 */
  parentListColumns?: number;
}

export interface GroupEditResultPayload {
  label: string;
  columns: number;
  rows: number;
  icon?: string;
  iconColor?: string;
  iconBase64?: string;
  libraryIcon?: string;
  viewMode?: "grid" | "list";
  listColumns?: number;
}

const GROUP_COLORS = ["", "#f38ba8", "#fab387", "#f9e2af", "#a6e3a1", "#89b4fa", "#cba6f7", "#f5c2e7", "#94e2d5"];

export function GroupEditWindow() {
  useChildTheme();
  const [mode, setMode] = useState<"create" | "rename">("create");
  const [label, setLabel] = useState("新しいグループ");
  const [columns, setColumns] = useState(4);
  const [rows, setRows] = useState(2);
  const [icon, setIcon] = useState("📂");
  const [iconColor, setIconColor] = useState("");
  const [iconBase64, setIconBase64] = useState<string | undefined>(undefined);
  const [libraryIcon, setLibraryIcon] = useState<string | undefined>(undefined);
  const [showIconPicker, setShowIconPicker] = useState(false);
  const [iconLibrary, setIconLibrary] = useState<IconInfo[]>([]);
  const [inheritViewMode, setInheritViewMode] = useState(true);
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [inheritListColumns, setInheritListColumns] = useState(true);
  const [listColumns, setListColumns] = useState(1);
  const [parentViewMode, setParentViewMode] = useState<"grid" | "list">("grid");
  const [parentListColumns, setParentListColumns] = useState(1);
  const [ready, setReady] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // ── 初期化: ready → init ハンドシェイク ──
  useEffect(() => {
    const unlistenInit = listen<GroupEditInitPayload>("group-edit-init", (event) => {
      const p = event.payload;
      setMode(p.mode);
      setLabel(p.label);
      setColumns(p.columns);
      setRows(p.rows);
      setIcon(p.icon ?? "📂");
      setIconColor(p.iconColor ?? "");
      setIconBase64(p.iconBase64);
      setLibraryIcon(p.libraryIcon);
      setInheritViewMode(!p.viewMode);
      setViewMode(p.viewMode ?? p.parentViewMode ?? "grid");
      setInheritListColumns(!p.listColumns);
      setListColumns(p.listColumns ?? p.parentListColumns ?? 1);
      setParentViewMode(p.parentViewMode ?? "grid");
      setParentListColumns(p.parentListColumns ?? 1);
      setReady(true);
      // フォーカスは次の tick で
      setTimeout(() => {
        inputRef.current?.focus();
        inputRef.current?.select();
      }, 50);
    });

    // アイコンライブラリの読み込み
    invoke<IconInfo[]>("list_icon_library").then(setIconLibrary).catch((e) => console.error("Failed to load icon library:", e));

    // ready 通知を送出
    emit("group-edit-ready");

    // ウィンドウ閉じ時に closed イベント
    const win = getCurrentWebviewWindow();
    win.onCloseRequested(() => {
      emit("group-edit-closed");
    });

    return () => {
      unlistenInit.then((fn) => fn());
    };
  }, []);

  // ── キーボード操作 ──
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        emit("group-edit-closed");
        getCurrentWebviewWindow().close();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  const handleSave = async () => {
    const trimmed = label.trim();
    if (!trimmed) return;
    await emit("group-edit-save", {
      label: trimmed, columns, rows,
      icon: icon || undefined, iconColor: iconColor || undefined,
      iconBase64: iconBase64 || undefined, libraryIcon: libraryIcon || undefined,
      viewMode: inheritViewMode ? undefined : viewMode,
      listColumns: inheritListColumns ? undefined : listColumns,
    } satisfies GroupEditResultPayload);
    try { await getCurrentWebviewWindow().close(); } catch { /* already closed */ }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      handleSave();
    }
  };

  if (!ready) {
    return (
      <div style={{ padding: 20, color: "var(--text-muted)", textAlign: "center" }}>
        読み込み中...
      </div>
    );
  }

  const title = mode === "create" ? "📁 サブグループ作成" : "✏ グループ名を変更";

  return (
    <div className="group-edit-window" onKeyDown={handleKeyDown} style={{
      display: "flex", flexDirection: "column", height: "100vh",
      background: "var(--bg-secondary)", color: "var(--text-primary)", fontSize: 13,
    }}>
      <div className="item-edit-header" data-tauri-drag-region style={{ cursor: "move" }}>
        <span data-tauri-drag-region style={{ flex: 1 }}>{title}</span>
        <button
          type="button"
          style={{ background: "none", border: "none", color: "var(--text-muted)", cursor: "pointer", fontSize: 16, padding: "0 2px", lineHeight: 1 }}
          onClick={() => { emit("group-edit-closed"); getCurrentWebviewWindow().close(); }}
          title="閉じる"
        >✕</button>
      </div>

      <div className="item-edit-body">
        <div className="item-edit-field">
          <label>グループ名</label>
          <input
            ref={inputRef}
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="グループの表示名"
          />
        </div>

        {/* アイコン選択 */}
        <div className="item-edit-field">
          <label>アイコン</label>
          <div className="icon-picker-row">
            <div className="icon-picker-preview">
              {iconBase64 ? (
                <img src={iconBase64.startsWith('data:') ? iconBase64 : `data:image/png;base64,${iconBase64}`} alt="icon" />
              ) : (
                <span className="icon-picker-emoji">{icon || "📂"}</span>
              )}
            </div>
            <button type="button" className="icon-picker-toggle" onClick={() => setShowIconPicker(!showIconPicker)}>
              {showIconPicker ? "▲ 閉じる" : "▼ ライブラリから選択"}
            </button>
            {(iconBase64 || libraryIcon) && (
              <button type="button" className="icon-picker-clear" onClick={() => { setIconBase64(undefined); setLibraryIcon(undefined); }}>
                ✕
              </button>
            )}
          </div>
          {showIconPicker && iconLibrary.length > 0 && (
            <div className="icon-picker-grid">
              {iconLibrary.map((ic) => (
                <div
                  key={ic.filename}
                  className={`icon-picker-item ${libraryIcon === ic.filename ? "selected" : ""}`}
                  title={ic.filename}
                  onClick={() => {
                    setIconBase64(ic.data_url);
                    setLibraryIcon(ic.filename);
                    setShowIconPicker(false);
                  }}
                >
                  <img src={ic.data_url} alt={ic.filename} />
                </div>
              ))}
            </div>
          )}
        </div>

        {/* P-35: アイコンカラー */}
        <div className="item-edit-field">
          <label>アイコンカラー</label>
          <div style={{ display: "flex", gap: 4, padding: "4px 0" }}>
            {GROUP_COLORS.map((c) => (
              <button key={c || "none"} onClick={() => setIconColor(c)} style={{
                width: 24, height: 24, borderRadius: "50%",
                background: c || "var(--text-muted)",
                border: iconColor === c ? "2px solid var(--accent-color)" : "2px solid transparent",
                cursor: "pointer",
                opacity: c ? 1 : 0.5,
              }} title={c || "デフォルト"} />
            ))}
          </div>
        </div>

        <div style={{ display: "flex", gap: 10 }}>
          <div className="item-edit-field" style={{ flex: 1 }}>
            <label>列数</label>
            <input
              type="number"
              min={2}
              max={8}
              value={columns}
              onChange={(e) => setColumns(Math.max(2, Math.min(8, parseInt(e.target.value) || 2)))}
            />
          </div>
          <div className="item-edit-field" style={{ flex: 1 }}>
            <label>行数</label>
            <input
              type="number"
              min={1}
              max={6}
              value={rows}
              onChange={(e) => setRows(Math.max(1, Math.min(6, parseInt(e.target.value) || 1)))}
            />
          </div>
          <div className="item-edit-field" style={{ flex: 1, justifyContent: "flex-end" }}>
            <label>&nbsp;</label>
            <span style={{ fontSize: 11, color: "var(--text-muted)", padding: "6px 0" }}>
              {columns * rows} スロット
            </span>
          </div>
        </div>

        {/* 表示設定 */}
        <div className="item-edit-field">
          <label>表示モード</label>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
            <label style={{ display: "flex", alignItems: "center", gap: 4, cursor: "pointer", fontSize: 11, color: "var(--text-muted)" }}>
              <input type="checkbox" checked={inheritViewMode} onChange={(e) => setInheritViewMode(e.target.checked)} />
              親設定を継承 ({parentViewMode === "grid" ? "グリッド" : "リスト"})
            </label>
          </div>
          {!inheritViewMode && (
            <select
              value={viewMode}
              onChange={(e) => setViewMode(e.target.value as "grid" | "list")}
              style={{ width: "100%", padding: "4px 8px", fontSize: 12, background: "var(--bg-primary)", color: "var(--text-primary)", border: "1px solid var(--border-color)", borderRadius: 3 }}
            >
              <option value="grid">グリッド (アイコン表示)</option>
              <option value="list">リスト (コンパクト表示)</option>
            </select>
          )}
        </div>

        <div className="item-edit-field">
          <label>リスト列数</label>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
            <label style={{ display: "flex", alignItems: "center", gap: 4, cursor: "pointer", fontSize: 11, color: "var(--text-muted)" }}>
              <input type="checkbox" checked={inheritListColumns} onChange={(e) => setInheritListColumns(e.target.checked)} />
              親設定を継承 ({parentListColumns}列)
            </label>
          </div>
          {!inheritListColumns && (
            <select
              value={listColumns}
              onChange={(e) => setListColumns(Number(e.target.value))}
              style={{ width: "100%", padding: "4px 8px", fontSize: 12, background: "var(--bg-primary)", color: "var(--text-primary)", border: "1px solid var(--border-color)", borderRadius: 3 }}
            >
              <option value="1">1列</option>
              <option value="2">2列</option>
              <option value="3">3列</option>
              <option value="4">4列</option>
            </select>
          )}
        </div>

        {/* プレビュー */}
        <div style={{
          display: "grid",
          gridTemplateColumns: `repeat(${columns}, 24px)`,
          gridTemplateRows: `repeat(${rows}, 24px)`,
          gap: 2,
          justifyContent: "center",
          padding: "6px 0",
        }}>
          {Array.from({ length: columns * rows }).map((_, i) => (
            <div
              key={i}
              style={{
                width: 24,
                height: 24,
                borderRadius: 3,
                border: "1px dashed var(--border-color)",
                background: "var(--bg-button-empty)",
              }}
            />
          ))}
        </div>
      </div>

      <div className="item-edit-footer">
        <button onClick={() => {
          emit("group-edit-closed");
          getCurrentWebviewWindow().close();
        }}>キャンセル</button>
        <button className="primary" onClick={handleSave}>
          {mode === "create" ? "作成" : "保存"} (Ctrl+Enter)
        </button>
      </div>
    </div>
  );
}
