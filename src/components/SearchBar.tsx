/* ============================================================
   SearchBar - アイテム検索 (Ctrl+F)
   ============================================================ */
import "./SearchBar.css";
import { useState, useEffect, useRef, useMemo } from "react";
import type { Tab, LauncherItem } from "../types";
import { isLauncherItem } from "../types";

interface SearchResult {
  tabId: string;
  tabLabel: string;
  index: number;
  item: LauncherItem;
}

interface SearchBarProps {
  tabs: Tab[];
  onNavigate: (tabId: string, index: number) => void;
  onLaunch: (item: LauncherItem) => void;
  onClose: () => void;
}

export function SearchBar({ tabs, onNavigate, onLaunch, onClose }: SearchBarProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // 全タブから LauncherItem を検索
  const results: SearchResult[] = useMemo(() => {
    if (!query.trim()) return [];
    const q = query.toLowerCase();
    const matches: SearchResult[] = [];
    for (const tab of tabs) {
      for (let i = 0; i < tab.items.length; i++) {
        const cell = tab.items[i];
        if (!isLauncherItem(cell)) continue;
        if (
          cell.label.toLowerCase().includes(q) ||
          cell.path.toLowerCase().includes(q)
        ) {
          matches.push({
            tabId: tab.id,
            tabLabel: tab.label,
            index: i,
            item: cell,
          });
        }
      }
    }
    return matches;
  }, [query, tabs]);

  // 選択インデックスをリセット
  useEffect(() => {
    setSelectedIndex(0);
  }, [results.length]);

  // スクロール追従
  useEffect(() => {
    const el = listRef.current?.children[selectedIndex] as HTMLElement | undefined;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === "Enter" && results[selectedIndex]) {
      const r = results[selectedIndex];
      if (e.ctrlKey || e.metaKey) {
        // Ctrl+Enter: タブを移動してハイライト
        onNavigate(r.tabId, r.index);
        onClose();
      } else {
        // Enter: 起動
        onLaunch(r.item);
        onClose();
      }
    }
  };

  return (
    <div className="search-overlay" onClick={onClose}>
      <div className="search-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="search-input-wrapper">
          <span className="search-icon">🔍</span>
          <input
            ref={inputRef}
            type="text"
            className="search-input"
            placeholder="アイテムを検索... (Enter:起動 / Ctrl+Enter:移動)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          {query && (
            <span className="search-count">
              {results.length} 件
            </span>
          )}
        </div>
        {results.length > 0 && (
          <div className="search-results" ref={listRef}>
            {results.map((r, i) => (
              <div
                key={`${r.tabId}-${r.index}`}
                className={`search-result-item ${i === selectedIndex ? "selected" : ""}`}
                onClick={() => {
                  onLaunch(r.item);
                  onClose();
                }}
                onMouseEnter={() => setSelectedIndex(i)}
              >
                {r.item.iconBase64 ? (
                  <img
                    src={`data:image/png;base64,${r.item.iconBase64}`}
                    className="search-result-icon"
                    alt=""
                  />
                ) : (
                  <span className="search-result-icon-placeholder">📄</span>
                )}
                <div className="search-result-info">
                  <div className="search-result-label">{r.item.label}</div>
                  <div className="search-result-path">{r.item.path}</div>
                </div>
                <span className="search-result-tab">{r.tabLabel}</span>
              </div>
            ))}
          </div>
        )}
        {query && results.length === 0 && (
          <div className="search-empty">一致するアイテムがありません</div>
        )}
      </div>
    </div>
  );
}
