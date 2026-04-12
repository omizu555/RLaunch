/* ============================================================
   TabSettingsDialog - タブ個別の表示設定ダイアログ
   ============================================================ */
import { useState, useEffect } from "react";
import type { Tab } from "../types";
import "./TabSettingsDialog.css";

interface TabSettingsDialogProps {
  tab: Tab;
  globalViewMode: "grid" | "list";
  globalListColumns: number;
  onSave: (tabId: string, settings: { viewMode?: "grid" | "list"; listColumns?: number }) => void;
  onClose: () => void;
}

export function TabSettingsDialog({
  tab,
  globalViewMode,
  globalListColumns,
  onSave,
  onClose,
}: TabSettingsDialogProps) {
  const [useGlobalViewMode, setUseGlobalViewMode] = useState(!tab.viewMode);
  const [viewMode, setViewMode] = useState<"grid" | "list">(tab.viewMode ?? globalViewMode);
  const [useGlobalListColumns, setUseGlobalListColumns] = useState(!tab.listColumns);
  const [listColumns, setListColumns] = useState(tab.listColumns ?? globalListColumns);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  const handleSave = () => {
    onSave(tab.id, {
      viewMode: useGlobalViewMode ? undefined : viewMode,
      listColumns: useGlobalListColumns ? undefined : listColumns,
    });
    onClose();
  };

  return (
    <div className="tab-settings-overlay" onClick={onClose}>
      <div className="tab-settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="tab-settings-header">
          <span className="tab-settings-title">⚙ タブ設定: {tab.label}</span>
          <button className="tab-settings-close" onClick={onClose}>✕</button>
        </div>

        <div className="tab-settings-body">
          {/* 表示モード */}
          <div className="tab-setting-group">
            <div className="tab-setting-row">
              <label className="tab-setting-label">表示モード</label>
              <label className="tab-setting-checkbox">
                <input
                  type="checkbox"
                  checked={useGlobalViewMode}
                  onChange={(e) => setUseGlobalViewMode(e.target.checked)}
                />
                全体設定を使用 ({globalViewMode === "grid" ? "グリッド" : "リスト"})
              </label>
            </div>
            {!useGlobalViewMode && (
              <select
                className="tab-setting-select"
                value={viewMode}
                onChange={(e) => setViewMode(e.target.value as "grid" | "list")}
              >
                <option value="grid">グリッド (アイコン表示)</option>
                <option value="list">リスト (コンパクト表示)</option>
              </select>
            )}
          </div>

          {/* リスト列数 */}
          <div className="tab-setting-group">
            <div className="tab-setting-row">
              <label className="tab-setting-label">リスト列数</label>
              <label className="tab-setting-checkbox">
                <input
                  type="checkbox"
                  checked={useGlobalListColumns}
                  onChange={(e) => setUseGlobalListColumns(e.target.checked)}
                />
                全体設定を使用 ({globalListColumns}列)
              </label>
            </div>
            {!useGlobalListColumns && (
              <select
                className="tab-setting-select"
                value={listColumns}
                onChange={(e) => setListColumns(Number(e.target.value))}
              >
                <option value="1">1列</option>
                <option value="2">2列</option>
                <option value="3">3列</option>
                <option value="4">4列</option>
              </select>
            )}
          </div>
        </div>

        <div className="tab-settings-footer">
          <button className="tab-settings-btn cancel" onClick={onClose}>キャンセル</button>
          <button className="tab-settings-btn save" onClick={handleSave}>保存</button>
        </div>
      </div>
    </div>
  );
}
