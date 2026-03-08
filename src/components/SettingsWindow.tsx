/* ============================================================
   SettingsWindow - 独立設定ウィンドウ
   Tauri emit/listen でメインウィンドウと通信
   ============================================================ */
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { AppSettings } from "../types";
import { DEFAULT_SETTINGS } from "../types";
import { listThemes, getThemesDirPath, applyThemeById } from "../utils/themeLoader";
import { applyWindowEffect } from "../utils/applyWindowEffect";
import type { ThemeInfo } from "../utils/themeLoader";

const POSITION_OPTIONS = [
  { value: "cursor", label: "カーソル位置" },
  { value: "center", label: "画面中央" },
  { value: "remember", label: "前回の位置を記憶" },
];

export function SettingsWindow() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [draft, setDraft] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [storePath, setStorePath] = useState<string>("");
  const [themes, setThemes] = useState<ThemeInfo[]>([]);
  const [themesDir, setThemesDir] = useState<string>("");

  // メインから設定データを受信
  useEffect(() => {
    let cancelled = false;

    async function init() {
      // listen を await して確実にリスナー登録してから ready を通知
      const unlistenFn = await listen<AppSettings>("settings-init", (event) => {
        if (cancelled) return;
        setSettings(event.payload);
        setDraft({ ...event.payload });
      });

      if (cancelled) {
        unlistenFn();
        return;
      }

      // リスナー登録完了後にメインへ準備完了を通知
      await emit("settings-ready", {});

      // 安全弁: 500ms 後に再度 ready を送信（メイン側の listen が遅れた場合）
      setTimeout(() => {
        if (!cancelled) emit("settings-ready", {});
      }, 500);

      // クリーンアップ用に unlisten を保存
      if (!cancelled) {
        cleanupRef = unlistenFn;
      } else {
        unlistenFn();
      }
    }

    let cleanupRef: (() => void) | null = null;
    init();
    invoke<string>("get_store_path").then(setStorePath).catch(() => {});
    // テーマ一覧を動的に取得
    listThemes().then(setThemes).catch(() => {});
    getThemesDirPath().then(setThemesDir).catch(() => {});

    return () => {
      cancelled = true;
      cleanupRef?.();
    };
  }, []);

  // 設定画面自体のテーマもプレビュー反映
  useEffect(() => {
    if (themes.length > 0 && draft.theme) {
      applyThemeById(themes, draft.theme);
      applyWindowEffect(themes, draft.theme);
    }
  }, [draft.theme, themes]);

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setDraft((prev) => {
      const next = { ...prev, [key]: value };
      // リアルタイムプレビュー: 変更をメインウィンドウに即座に反映
      emit("settings-preview", next);
      return next;
    });
  };

  const gridChanged = settings
    ? draft.defaultGridColumns !== settings.defaultGridColumns ||
      draft.defaultGridRows !== settings.defaultGridRows
    : false;

  const handleSave = async () => {
    if (gridChanged) {
      const ok = window.confirm(
        `グリッドサイズを ${draft.defaultGridColumns}×${draft.defaultGridRows} に変更します。\n` +
        `すべての既存タブのサイズが変更されます（アイテムは保持されます）。\n\n続行しますか？`
      );
      if (!ok) return;
    }
    await emit("settings-save", draft);
    getCurrentWebviewWindow().close();
  };

  const handleClose = async () => {
    await emit("settings-closed", {});
    getCurrentWebviewWindow().close();
  };

  const handleOpenStoreFolder = async () => {
    if (!storePath) return;
    try {
      await invoke("open_file_location", { path: storePath });
    } catch (e) {
      console.error("Failed to open store folder:", e);
    }
  };

  const handleExportConfig = async () => {
    if (!storePath) return;
    try {
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const content = await readTextFile(storePath);
      await navigator.clipboard.writeText(content);
      alert("設定をクリップボードにコピーしました");
    } catch (e) {
      console.error("Failed to export config:", e);
      alert("エクスポートに失敗しました");
    }
  };

  /** P-36: ファイルにエクスポート */
  const handleExportFile = async () => {
    if (!storePath) return;
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const { readTextFile, writeTextFile } = await import("@tauri-apps/plugin-fs");
      const dest = await save({
        title: "設定をエクスポート",
        defaultPath: "rlaunch-backup.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!dest) return;
      const content = await readTextFile(storePath);
      await writeTextFile(dest, content);
      alert("設定をエクスポートしました");
    } catch (e) {
      console.error("Failed to export config:", e);
      alert("エクスポートに失敗しました");
    }
  };

  /** P-36: ファイルからインポート */
  const handleImportFile = async () => {
    try {
      const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const selected = await openDialog({
        title: "設定をインポート",
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!selected) return;
      const filePath = typeof selected === "string" ? selected : selected;
      const raw = await readTextFile(filePath);
      const data = JSON.parse(raw);

      // 基本的なバリデーション
      if (typeof data !== "object" || data === null) {
        alert("無効なファイル形式です");
        return;
      }

      const mode = window.confirm(
        "インポート方法を選択してください:\n\n" +
        "OK = 上書き（現在のデータを完全に置換）\n" +
        "キャンセル = マージ（既存データに追加）"
      );

      await emit("settings-import", { data, mode: mode ? "overwrite" : "merge" });
      alert(mode ? "設定を上書きインポートしました" : "設定をマージインポートしました");
      // ウィンドウを閉じてリロードを促す
      getCurrentWebviewWindow().close();
    } catch (e) {
      console.error("Failed to import config:", e);
      alert("インポートに失敗しました: " + (e instanceof Error ? e.message : String(e)));
    }
  };

  if (!settings) {
    return (
      <div className="settings-app" data-theme={DEFAULT_SETTINGS.theme}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}>
          <span style={{ color: "var(--text-muted)" }}>設定を読み込み中...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="settings-app" data-theme={draft.theme}>
      <div className="settings-header">
        <h2>⚙ 設定</h2>
      </div>

      <div className="settings-body">
        {/* --- テーマ --- */}
        <div className="setting-group">
          <label className="setting-label">テーマ</label>
          <div className="setting-theme-row">
            <select
              className="setting-select"
              value={draft.theme}
              onChange={(e) => update("theme", e.target.value)}
            >
              {themes.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.label}{t.author !== "builtin" ? ` (${t.author})` : ""}
                </option>
              ))}
            </select>
            {themesDir && (
              <button
                className="settings-btn secondary small"
                title="テーマフォルダを開く"
                onClick={() => invoke("open_file_location", { path: themesDir + "\\" })}
              >
                📁
              </button>
            )}
          </div>
        </div>

        {/* --- ホットキー --- */}
        <div className="setting-group">
          <label className="setting-label">ホットキー</label>
          <input
            className="setting-input"
            type="text"
            value={draft.hotkey}
            onChange={(e) => update("hotkey", e.target.value)}
            placeholder="例: Ctrl+Space"
          />
        </div>

        {/* --- 表示位置 --- */}
        <div className="setting-group">
          <label className="setting-label">表示位置</label>
          <select
            className="setting-select"
            value={draft.windowPosition}
            onChange={(e) => update("windowPosition", e.target.value as AppSettings["windowPosition"])}
          >
            {POSITION_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
        </div>

        {/* --- グリッドサイズ --- */}
        <div className="setting-row">
          <div className="setting-group">
            <label className="setting-label">列数</label>
            <input
              className="setting-input"
              type="number"
              min={4}
              max={16}
              value={draft.defaultGridColumns}
              onChange={(e) => update("defaultGridColumns", parseInt(e.target.value) || 8)}
            />
          </div>
          <div className="setting-group">
            <label className="setting-label">行数</label>
            <input
              className="setting-input"
              type="number"
              min={2}
              max={10}
              value={draft.defaultGridRows}
              onChange={(e) => update("defaultGridRows", parseInt(e.target.value) || 4)}
            />
          </div>
          <div className="setting-group" style={{ justifyContent: "flex-end" }}>
            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
              {draft.defaultGridColumns * draft.defaultGridRows} スロット
            </span>
          </div>
        </div>
        {gridChanged && (
          <div style={{
            fontSize: "11px",
            color: "var(--accent-color)",
            background: "rgba(137, 180, 250, 0.08)",
            padding: "6px 10px",
            borderRadius: "4px",
            lineHeight: 1.4,
          }}>
            ⚠ グリッドサイズの変更はすべてのタブに適用されます。既存アイテムは保持されます。
          </div>
        )}

        {/* --- セルサイズ --- */}
        <div className="setting-group">
          <label className="setting-label">
            セルサイズ: {draft.cellSize}px
          </label>
          <input
            className="setting-range"
            type="range"
            min={40}
            max={120}
            step={4}
            value={draft.cellSize}
            onChange={(e) => update("cellSize", parseInt(e.target.value))}
          />
          <div className="setting-hint">
            ウィンドウサイズは セルサイズ × 列数 × 行数 から自動計算されます
          </div>
        </div>

        {/* --- トグル設定 --- */}
        <div className="setting-group">
          <label className="setting-toggle">
            <input
              type="checkbox"
              checked={draft.autoStart}
              onChange={(e) => update("autoStart", e.target.checked)}
            />
            <span>Windows 起動時に自動起動</span>
          </label>
        </div>

        <div className="setting-group">
          <label className="setting-toggle">
            <input
              type="checkbox"
              checked={draft.hideOnLaunch}
              onChange={(e) => update("hideOnLaunch", e.target.checked)}
            />
            <span>アプリ起動後にウィンドウを隠す</span>
          </label>
        </div>

        <div className="setting-group">
          <label className="setting-toggle">
            <input
              type="checkbox"
              checked={draft.showLabels}
              onChange={(e) => update("showLabels", e.target.checked)}
            />
            <span>ラベルを表示</span>
          </label>
        </div>

        {/* P-29: ラベルフォントサイズ */}
        <div className="setting-group">
          <label className="setting-label">
            ラベルフォントサイズ: {draft.labelFontSize ?? 10}px
          </label>
          <input
            className="setting-range"
            type="range"
            min={8}
            max={16}
            step={1}
            value={draft.labelFontSize ?? 10}
            onChange={(e) => update("labelFontSize", parseInt(e.target.value))}
          />
        </div>

        <div className="setting-group">
          <label className="setting-label">アプリタイトル</label>
          <input
            className="setting-input"
            type="text"
            value={draft.appTitle ?? "RLaunch"}
            placeholder="空欄はタイトル非表示"
            onChange={(e) => update("appTitle", e.target.value)}
          />
        </div>

      </div>

      <div className="settings-footer">
        {storePath && (
          <div className="settings-store-info">
            <div className="settings-store-path" title={storePath}>
              📁 {storePath}
            </div>
            <div className="settings-store-actions">
              <button className="settings-btn secondary small" onClick={handleOpenStoreFolder}>
                フォルダを開く
              </button>
              <button className="settings-btn secondary small" onClick={handleExportConfig}>
                📋 コピー
              </button>
              <button className="settings-btn secondary small" onClick={handleExportFile}>
                📤 エクスポート
              </button>
              <button className="settings-btn secondary small" onClick={handleImportFile}>
                📥 インポート
              </button>
            </div>
          </div>
        )}
        <div className="settings-footer-buttons">
          <button className="settings-btn secondary" onClick={handleClose}>
            キャンセル
          </button>
          <button className="settings-btn primary" onClick={handleSave}>
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
