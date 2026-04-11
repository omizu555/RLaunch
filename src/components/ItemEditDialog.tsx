/* ============================================================
   ItemEditDialog - アイテムのプロパティ編集ダイアログ
   右クリック → 「✏ 編集」で表示される。
   ============================================================ */
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./ItemEditDialog.css";
import type { LauncherItem, WindowState } from "../types";

interface IconInfo {
  filename: string;
  data_url: string;
}

interface ItemEditDialogProps {
  item: LauncherItem;
  onSave: (updated: LauncherItem) => void;
  onClose: () => void;
}

export function ItemEditDialog({ item, onSave, onClose }: ItemEditDialogProps) {
  const [label, setLabel] = useState(item.label);
  const [path, setPath] = useState(item.path);
  const [args, setArgs] = useState(item.args ?? "");
  const [workingDir, setWorkingDir] = useState(item.workingDir ?? "");
  const [windowState, setWindowState] = useState<WindowState>(item.windowState ?? "normal");
  const [runAs, setRunAs] = useState(item.runAs ?? false);
  const [hotkey, setHotkey] = useState(item.hotkey ?? "");
  const [iconBase64, setIconBase64] = useState(item.iconBase64);
  const [libraryIcon, setLibraryIcon] = useState(item.libraryIcon);
  const [showIconPicker, setShowIconPicker] = useState(false);
  const [iconLibrary, setIconLibrary] = useState<IconInfo[]>([]);
  const labelRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    labelRef.current?.focus();
    labelRef.current?.select();
  }, []);

  // アイコンライブラリの読み込み
  useEffect(() => {
    invoke<IconInfo[]>("list_icon_library").then(setIconLibrary).catch((e) => console.error("Failed to load icon library:", e));
  }, []);

  // Escape で閉じる
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  const handleSave = () => {
    const updated: LauncherItem = {
      ...item,
      label: label.trim() || item.label,
      path: path.trim() || item.path,
      args: args.trim() || undefined,
      workingDir: workingDir.trim() || undefined,
      windowState,
      runAs,
      hotkey: hotkey.trim() || undefined,
      iconBase64,
      libraryIcon,
      updatedAt: new Date().toISOString(),
    };
    onSave(updated);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      handleSave();
    }
  };

  const launchCount = item.launchCount ?? 0;
  const lastLaunched = item.lastLaunchedAt
    ? new Date(item.lastLaunchedAt).toLocaleString("ja-JP")
    : "—";

  return (
    <>
      <div className="item-edit-overlay" onClick={onClose} />
      <div className="item-edit-dialog" onKeyDown={handleKeyDown}>
        <div className="item-edit-header">
          ✏ アイテム編集
        </div>

        <div className="item-edit-body">
          {/* 統計情報 */}
          <div className="item-edit-stats">
            <span>🚀 起動回数: {launchCount}回</span>
            <span>🕐 最終起動: {lastLaunched}</span>
          </div>

          {/* アイコン選択 */}
          <div className="item-edit-field">
            <label>アイコン</label>
            <div className="icon-picker-row">
              <div className="icon-picker-preview">
                {iconBase64 ? (
                  <img
                    src={iconBase64.startsWith('data:') ? iconBase64 : `data:image/png;base64,${iconBase64}`}
                    alt="icon"
                  />
                ) : (
                  <span className="icon-picker-emoji">—</span>
                )}
              </div>
              <button
                type="button"
                className="icon-picker-toggle"
                onClick={() => setShowIconPicker(!showIconPicker)}
              >
                {showIconPicker ? "▲ 閉じる" : "▼ ライブラリから選択"}
              </button>
              {(iconBase64 || libraryIcon) && (
                <button
                  type="button"
                  className="icon-picker-clear"
                  onClick={() => {
                    setIconBase64(undefined);
                    setLibraryIcon(undefined);
                  }}
                >
                  ✕
                </button>
              )}
            </div>
            {showIconPicker && iconLibrary.length > 0 && (
              <div className="icon-picker-grid">
                {iconLibrary.map((icon) => (
                  <div
                    key={icon.filename}
                    className={`icon-picker-item ${libraryIcon === icon.filename ? "selected" : ""}`}
                    title={icon.filename}
                    onClick={() => {
                      setIconBase64(icon.data_url);
                      setLibraryIcon(icon.filename);
                      setShowIconPicker(false);
                    }}
                  >
                    <img src={icon.data_url} alt={icon.filename} />
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="item-edit-field">
            <label>ラベル</label>
            <input
              ref={labelRef}
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder="ボタンに表示する名前"
            />
          </div>

          <div className="item-edit-field">
            <label>パス</label>
            <input
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="実行ファイルパス / URL"
            />
          </div>

          <div className="item-edit-field">
            <label>引数</label>
            <input
              value={args}
              onChange={(e) => setArgs(e.target.value)}
              placeholder="起動時の引数（省略可）"
            />
          </div>

          <div className="item-edit-field">
            <label>作業ディレクトリ</label>
            <input
              value={workingDir}
              onChange={(e) => setWorkingDir(e.target.value)}
              placeholder="作業フォルダ（省略可）"
            />
          </div>

          <div className="item-edit-field">
            <label>ウィンドウ状態</label>
            <select value={windowState} onChange={(e) => setWindowState(e.target.value as WindowState)}>
              <option value="normal">通常</option>
              <option value="maximized">最大化</option>
              <option value="minimized">最小化</option>
            </select>
          </div>

          <div className="item-edit-field" style={{ flexDirection: "row", alignItems: "center", gap: "8px" }}>
            <input
              type="checkbox"
              id="edit-runas"
              checked={runAs}
              onChange={(e) => setRunAs(e.target.checked)}
              style={{ width: "auto" }}
            />
            <label htmlFor="edit-runas" style={{ fontSize: "12px", color: "var(--text-primary)" }}>
              管理者として起動
            </label>
          </div>

          <div className="item-edit-field">
            <label>グローバルホットキー</label>
            <input
              value={hotkey}
              onChange={(e) => setHotkey(e.target.value)}
              placeholder="例: Ctrl+Alt+C（省略可）"
            />
          </div>

          <div className="item-edit-field">
            <label>タイプ</label>
            <input value={item.type} readOnly />
          </div>
        </div>

        <div className="item-edit-footer">
          <button onClick={onClose}>キャンセル</button>
          <button className="primary" onClick={handleSave}>保存 (Ctrl+Enter)</button>
        </div>
      </div>
    </>
  );
}
