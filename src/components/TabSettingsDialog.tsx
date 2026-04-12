/* ============================================================
   TabSettingsDialog - タブ設定ダイアログ
   タブ名、グリッドサイズ、表示モード、リスト列数を一括設定
   ============================================================ */
import { useState, useEffect } from "react";
import type { Tab } from "../types";
import "./TabSettingsDialog.css";

export interface TabSettingsResult {
  label?: string;
  gridColumns?: number;
  gridRows?: number;
  viewMode?: "grid" | "list";
  listColumns?: number;
}

interface TabSettingsDialogProps {
  tab: Tab;
  globalViewMode: "grid" | "list";
  globalListColumns: number;
  onSave: (tabId: string, settings: TabSettingsResult) => void;
  onClose: () => void;
}

export function TabSettingsDialog({
  tab,
  globalViewMode,
  globalListColumns,
  onSave,
  onClose,
}: TabSettingsDialogProps) {
  const [label, setLabel] = useState(tab.label);
  const [gridColumns, setGridColumns] = useState(tab.gridColumns);
  const [gridRows, setGridRows] = useState(tab.gridRows);
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
      label: label.trim() || tab.label,
      gridColumns: Math.max(1, Math.min(20, gridColumns)),
      gridRows: Math.max(1, Math.min(10, gridRows)),
      viewMode: useGlobalViewMode ? undefined : viewMode,
      listColumns: useGlobalListColumns ? undefined : listColumns,
    });
    onClose();
  };

  return (
    <div className="tab-settings-overlay" onClick={onClose}>
      <div className="tab-settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="tab-settings-header">
          <span className="tab-settings-title">⚙ タブ設定</span>
          <button className="tab-settings-close" onClick={onClose}>✕</button>
        </div>

        <div className="tab-settings-body">
          {/* ═══ 基本 ═══ */}
          <div className="tab-setting-section">
            <div className="tab-setting-section-title">🏷 基本</div>

            <div className="tab-setting-group">
              <label className="tab-setting-label">タブ名</label>
              <input
                className="tab-setting-input"
                type="text"
                value={label}
                onChange={(e) => setLabel(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleSave(); }}
                autoFocus
              />
            </div>

            <div className="tab-setting-group">
              <label className="tab-setting-label">グリッドサイズ</label>
              <div className="tab-setting-grid-size">
                <input
                  className="tab-setting-input-small"
                  type="number"
                  min={1}
                  max={20}
                  value={gridColumns}
                  onChange={(e) => setGridColumns(parseInt(e.target.value) || 1)}
                />
                <span className="tab-setting-separator">×</span>
                <input
                  className="tab-setting-input-small"
                  type="number"
                  min={1}
                  max={10}
                  value={gridRows}
                  onChange={(e) => setGridRows(parseInt(e.target.value) || 1)}
                />
                <span className="tab-setting-hint">列 × 行</span>
              </div>
            </div>
          </div>

          {/* ═══ 表示 ═══ */}
          <div className="tab-setting-section">
            <div className="tab-setting-section-title">📐 表示</div>

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
        </div>

        <div className="tab-settings-footer">
          <button className="tab-settings-btn cancel" onClick={onClose}>キャンセル</button>
          <button className="tab-settings-btn save" onClick={handleSave}>保存</button>
        </div>
      </div>
    </div>
  );
}
