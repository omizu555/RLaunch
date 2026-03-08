/* ============================================================
   FolderBrowserWindow - 独立ウィンドウ版フォルダブラウザ
   useChildWindow パターンで親ウィンドウとイベント通信する。

   イベントフロー:
   1. 子ウィンドウ起動 → "folder-browser-ready" emit
   2. 親が "folder-browser-init" で初期パスを送信
   3. ファイルクリック → "folder-browser-launch" (path) → 子ウィンドウ閉じ
   4. Explorerで開く → "folder-browser-open-explorer" (path)
   5. ウィンドウ閉じ → "folder-browser-closed"
   ============================================================ */
import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { type DirectoryEntry, formatSize, getFileEmoji } from "../utils/fileUtils";
import { useChildTheme } from "../hooks/useChildTheme";
import { useFocusLossAutoClose } from "../hooks/useFocusLossAutoClose";

export function FolderBrowserWindow() {
  const { refreshTheme } = useChildTheme();
  const [currentPath, setCurrentPath] = useState<string>("");
  const [entries, setEntries] = useState<DirectoryEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<string[]>([]);

  /** ディレクトリの中身を取得 */
  const loadDir = useCallback(async (dirPath: string) => {
    try {
      setError(null);
      const result = await invoke<DirectoryEntry[]>("list_directory", { path: dirPath });
      setEntries(result);
      setCurrentPath(dirPath);
      // ウィンドウタイトルを更新
      const folderName = dirPath.split(/[\\/]/).pop() || dirPath;
      getCurrentWebviewWindow().setTitle(`📂 ${folderName}`).catch(() => {});
    } catch (e) {
      setError(String(e));
      setEntries([]);
    }
  }, []);

  /** サブフォルダに移動 */
  const navigateInto = useCallback(
    (dirPath: string) => {
      setHistory((prev) => [...prev, currentPath]);
      loadDir(dirPath);
    },
    [currentPath, loadDir],
  );

  /** 1つ上のフォルダに戻る */
  const navigateUp = useCallback(() => {
    if (history.length > 0) {
      const prev = history[history.length - 1];
      setHistory((h) => h.slice(0, -1));
      loadDir(prev);
    } else {
      const parent = currentPath.replace(/[\\/][^\\/]+$/, "");
      if (parent && parent !== currentPath) {
        setHistory((prev) => [...prev, currentPath]);
        loadDir(parent);
      }
    }
  }, [currentPath, history, loadDir]);

  /** ファイルクリック → 起動イベントを親に送信して閉じる */
  const handleLaunchFile = useCallback(async (path: string) => {
    await emit("folder-browser-launch", { path });
    emit("folder-browser-closed");
  }, []);

  /** Explorer で開く → ウィンドウも閉じる */
  const handleOpenExplorer = useCallback(async (dirPath: string) => {
    await emit("folder-browser-open-explorer", { path: dirPath });
    emit("folder-browser-closed");
  }, []);

  /** エントリクリック */
  const handleClick = useCallback(
    (entry: DirectoryEntry) => {
      if (entry.is_dir) {
        navigateInto(entry.path);
      } else {
        handleLaunchFile(entry.path);
      }
    },
    [navigateInto, handleLaunchFile],
  );

  // ── 初期化: ready → init ハンドシェイク ──
  useEffect(() => {
    const unlistenInit = listen<{ path: string }>("folder-browser-init", (event) => {
      const p = event.payload.path;
      if (p) {
        setHistory([]);
        loadDir(p);
      }
      // reusable ウィンドウなのでテーマ変更に追従
      refreshTheme();
    });

    // ready 通知を送出
    emit("folder-browser-ready");

    // ウィンドウ閉じリクエスト → デフォルト抑止 + イベント通知（reusable 対応）
    const win = getCurrentWebviewWindow();
    win.onCloseRequested((event) => {
      event.preventDefault();
      emit("folder-browser-closed");
    });

    return () => {
      unlistenInit.then((fn) => fn());
    };
  }, [loadDir, refreshTheme]);

  // ── キーボード操作 ──
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") emit("folder-browser-closed");
      if (e.key === "Backspace") navigateUp();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [navigateUp]);

  // ── ウィンドウ移動検知でフォーカス喪失クローズを抑制 ──
  useFocusLossAutoClose("folder-browser-closed");

  if (!currentPath) {
    return (
      <div className="folder-browser folder-browser--window">
        <div className="folder-browser-empty">読み込み中...</div>
      </div>
    );
  }

  return (
    <div className="folder-browser folder-browser--window">
      {/* ── ナビゲーションバー（ヘッダーなし: フォーカス外/Esc で閉じる） ── */}
      <div className="folder-browser-nav" data-tauri-drag-region>
        <button onClick={navigateUp} title="上のフォルダへ (Backspace)">⬆</button>
        <span className="folder-browser-path" data-tauri-drag-region title={currentPath}>{currentPath}</span>
      </div>

      {/* ── エントリリスト ── */}
      <div className="folder-browser-list">
        {error ? (
          <div className="folder-browser-empty">⚠ {error}</div>
        ) : entries.length === 0 ? (
          <div className="folder-browser-empty">空のフォルダです</div>
        ) : (
          entries.map((entry) => (
            <div
              key={entry.path}
              className="folder-browser-entry"
              onClick={() => handleClick(entry)}
              title={entry.path}
            >
              <span className="folder-browser-entry-icon">
                {entry.is_dir ? "📁" : getFileEmoji(entry.extension)}
              </span>
              <span className="folder-browser-entry-name">{entry.name}</span>
              {!entry.is_dir && (
                <span className="folder-browser-entry-size">{formatSize(entry.size)}</span>
              )}
            </div>
          ))
        )}
      </div>

      {/* ── フッター ── */}
      <div className="folder-browser-footer">
        <button onClick={() => handleOpenExplorer(currentPath)}>
          📂 Explorer で開く
        </button>
      </div>
    </div>
  );
}
