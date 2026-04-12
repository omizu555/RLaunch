/* ============================================================
   MyLauncher - App (メインコンポーネント)
   責務: レイアウト + フック統合のみ。
   ロジックは hooks/ に、スタイルは components/*.css に分離。
   ============================================================ */
import { useEffect, useState, useCallback, useRef } from "react";
import "./App.css";
import type { AppSettings, WidgetItem, GroupItem, GridCell, LauncherItem } from "./types";
import { isLauncherItem } from "./types";
import { NOTIFICATION_DURATION } from "./constants";
import { CustomTitleBar } from "./components/CustomTitleBar";
import { TabBar } from "./components/TabBar";
import { LauncherGrid } from "./components/LauncherGrid";
import { LauncherList } from "./components/LauncherList";
import { useWidgetSelectWindow } from "./hooks/useWidgetSelectWindow";
import { SearchBar } from "./components/SearchBar";
import { ItemEditDialog } from "./components/ItemEditDialog";
import { useGroupPopupWindow } from "./hooks/useGroupPopupWindow";
import { useGroupEditWindow } from "./hooks/useGroupEditWindow";
import { useFolderBrowserWindow } from "./hooks/useFolderBrowserWindow";
import { useHotkey } from "./hooks/useHotkey";
import { useAutoHide } from "./hooks/useAutoHide";
import { useDragDrop } from "./hooks/useDragDrop";
import { useLauncher } from "./hooks/useLauncher";
import { useTabManager } from "./hooks/useTabManager";
import { useWidgetManager } from "./hooks/useWidgetManager";
import { useSettingsWindow } from "./hooks/useSettingsWindow";
import { useWidgetSettingsWindow } from "./hooks/useWidgetSettingsWindow";
import { useWindowSize } from "./hooks/useWindowSize";
import { listThemes, applyThemeById } from "./utils/themeLoader";
import type { ThemeInfo } from "./utils/themeLoader";
import { applyWindowEffect } from "./utils/applyWindowEffect";
import { importData } from "./stores/launcherStore";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

function App() {
  const [showSearch, setShowSearch] = useState(false);
  const [notification, setNotification] = useState<string | null>(null);
  const [pinned, setPinned] = useState(false);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const [themes, setThemes] = useState<ThemeInfo[]>([]);
  const themesRef = useRef<ThemeInfo[]>([]);
  const [editTarget, setEditTarget] = useState<{ index: number; item: LauncherItem } | null>(null);
  const groupPopupRef = useRef<{ index: number } | null>(null);
  const [isDraggingItem, setIsDraggingItem] = useState(false);
  const [invalidPaths, setInvalidPaths] = useState<Set<string>>(new Set());
  const groupEditRef = useRef<
    | { mode: "create"; index: number }
    | { mode: "rename"; index: number; group: GroupItem }
    | null
  >(null);

  // ── 通知ヘルパー ──
  const showNotification = useCallback((msg: string) => {
    setNotification(msg);
    setTimeout(() => setNotification(null), NOTIFICATION_DURATION);
  }, []);

  // ── データ管理 ──
  const tabManager = useTabManager(showNotification);
  const {
    tabs, activeTabId, activeTab, settings, loading,
    setActiveTabId, setTabs,
    handleAddTab, handleRenameTab, handleRemoveTab,
    handleReorderTabs, handleTabColorChange, handleDuplicateTab, handleResizeTab,
    handleCellClear, handleCellSwap, handleNativeDrop,
    handleSettingsChange, loadData,
  } = tabManager;

  // ── テーマ一覧取得 & 適用 ──
  useEffect(() => {
    listThemes().then((t) => {
      setThemes(t);
      themesRef.current = t;
    }).catch((e) => console.error("Failed to load themes:", e));
  }, []);

  // 設定のテーマが変わったら CSS 変数 + 透過効果を動的適用
  useEffect(() => {
    if (!loading && themes.length > 0) {
      applyThemeById(themes, settings.theme);
      applyWindowEffect(themes, settings.theme);
    }
  }, [settings.theme, themes, loading]);

  // P-38: パスの有効性チェック（タブ切替・ロード後）
  useEffect(() => {
    if (loading || !activeTab) return;
    let cancelled = false;
    (async () => {
      try {
        const { exists } = await import("@tauri-apps/plugin-fs");
        const bad = new Set<string>();
        for (const cell of activeTab.items) {
          if (!isLauncherItem(cell)) continue;
          if (cell.type === "url") continue;
          if (cell.path) {
            const ok = await exists(cell.path);
            if (!ok && !cancelled) bad.add(cell.id);
          }
        }
        if (!cancelled) setInvalidPaths(bad);
      } catch { /* ignore */ }
    })();
    return () => { cancelled = true; };
  }, [loading, activeTab]);

  // ── ウィンドウサイズ管理 ──
  const { resize } = useWindowSize(settings, loading);

  // ── 設定ウィンドウ ──
  const onSettingsSave = useCallback(async (newSettings: AppSettings) => {
    await handleSettingsChange(newSettings);
    await resize(newSettings);
  }, [handleSettingsChange, resize]);

  // リアルタイムプレビュー: 設定変更をテーマ含め即座反映
  const onSettingsPreview = useCallback((previewSettings: AppSettings) => {
    // テーマ CSS 変数を動的に適用
    if (themesRef.current.length > 0) {
      applyThemeById(themesRef.current, previewSettings.theme);
    }
    // セルサイズのプレビュー
    const app = document.querySelector('.app') as HTMLElement | null;
    if (app) {
      app.style.setProperty('--cell-size', `${previewSettings.cellSize}px`);
    }
    // P-50: テーマ変数から透過効果をプレビュー
    applyWindowEffect(themesRef.current, previewSettings.theme);
  }, []);

  const { openSettingsWindow } = useSettingsWindow({
    settings,
    onSettingsSave,
    onSettingsPreview,
    onImport: useCallback(async (data: unknown, mode: "overwrite" | "merge") => {
      await importData(data as Record<string, unknown>, mode);
      await loadData();
    }, [loadData]),
  });

  // ── ランチャー＆ウィジェット ──
  const launcher = useLauncher({
    hideOnLaunch: settings.hideOnLaunch,
    pinned,
    onNotify: showNotification,
    onItemLaunched: tabManager.handleCellUpdate,
  });

  /** FolderBrowser からファイルを起動 */
  const handleFolderLaunchFile = useCallback(
    (path: string) => {
      launcher.launch({
        id: "_browse",
        label: path.split(/[\\/]/).pop() || path,
        path,
        type: "document",
        createdAt: "",
        updatedAt: "",
      });
    },
    [launcher],
  );

  /** FolderBrowser から Explorer で開く */
  const handleFolderOpenExplorer = useCallback(
    (dirPath: string) => {
      launcher.launch({
        id: "_browse",
        label: dirPath,
        path: dirPath,
        type: "folder",
        createdAt: "",
        updatedAt: "",
      });
    },
    [launcher],
  );

  // ── フォルダブラウザウィンドウ ──
  const { openFolderBrowser } = useFolderBrowserWindow({
    onLaunchFile: handleFolderLaunchFile,
    onOpenExplorer: handleFolderOpenExplorer,
  });

  // ── グループ編集ウィンドウ ──
  const { openCreateGroup, openRenameGroup } = useGroupEditWindow({
    onSave: ({ label, columns, rows, icon, iconColor, iconBase64, libraryIcon }) => {
      const pending = groupEditRef.current;
      if (!pending) return;
      if (pending.mode === "create") {
        const group: GroupItem = {
          id: crypto.randomUUID(),
          type: "group",
          label,
          icon,
          iconColor,
          iconBase64,
          libraryIcon,
          items: new Array(columns * rows).fill(null),
          gridColumns: columns,
          gridRows: rows,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        };
        tabManager.handleCellUpdate(pending.index, group);
        showNotification(`グループ「${label}」を作成しました`);
      } else {
        const existing = pending.group;
        const newTotal = columns * rows;
        let newItems = [...existing.items];
        if (newTotal > newItems.length) {
          newItems = [...newItems, ...new Array(newTotal - newItems.length).fill(null)];
        } else if (newTotal < newItems.length) {
          newItems = newItems.slice(0, newTotal);
        }
        const updated: GroupItem = {
          ...existing,
          label,
          icon,
          iconColor,
          iconBase64,
          libraryIcon,
          gridColumns: columns,
          gridRows: rows,
          items: newItems,
          updatedAt: new Date().toISOString(),
        };
        tabManager.handleCellUpdate(pending.index, updated);
        showNotification(`グループ「${label}」を更新しました`);
      }
      groupEditRef.current = null;
    },
    onClosed: () => {
      groupEditRef.current = null;
    },
  });

  // ── グループポップアップウィンドウ ──
  const { openGroupPopup, closeGroupPopup } = useGroupPopupWindow({
    onLaunch: (item) => {
      // 起動後の処理（必要に応じてhideWindow等）
      showNotification(`「${item.label}」を起動しました`);
    },
    onGroupUpdate: (updatedGroup) => {
      const pending = groupPopupRef.current;
      if (pending) {
        tabManager.handleCellUpdate(pending.index, updatedGroup);
      }
    },
    onClosed: () => {
      groupPopupRef.current = null;
      // ポップアップが閉じた後、メインにフォーカスがなければメインも隠す（デスクトップクリック対応）
      if (!pinned) {
        setTimeout(() => {
          getCurrentWebviewWindow().isFocused().then((focused) => {
            if (!focused) {
              getCurrentWebviewWindow().hide();
            }
          }).catch((e) => console.error("Failed to check focus:", e));
        }, 100);
      }
    },
  });

  /** セルクリック: フォルダ(browse設定)なら FolderBrowser、グループならポップアップ、それ以外は起動 */
  const handleCellClick = useCallback(
    (index: number, cell: GridCell) => {
      // 他の操作時はグループポップアップを閉じる
      if (!(cell && cell.type === "group")) {
        closeGroupPopup();
      }
      if (cell && cell.type === "folder") {
        const item = cell as LauncherItem;
        if (item.folderAction === "browse") {
          openFolderBrowser(item.path);
          return;
        }
      }
      // グループクリック → 独立ウィンドウでポップアップ表示
      if (cell && cell.type === "group") {
        groupPopupRef.current = { index };
        openGroupPopup(cell as GroupItem);
        return;
      }
      launcher.launchFromCell(index, cell);
    },
    [launcher, openFolderBrowser, openGroupPopup, closeGroupPopup],
  );

  /** アイテム編集ダイアログを開く */
  const handleEditItem = useCallback(
    (index: number, item: LauncherItem) => {
      setEditTarget({ index, item });
    },
    [],
  );

  /** アイテム編集を保存 */
  const handleEditSave = useCallback(
    (updated: LauncherItem) => {
      if (editTarget) {
        tabManager.handleCellUpdate(editTarget.index, updated);
        showNotification(`「${updated.label}」を更新しました`);
      }
      setEditTarget(null);
    },
    [editTarget, tabManager, showNotification],
  );

  /** サブグループ作成 → 独立ウィンドウを開く */
  const handleCreateGroup = useCallback(
    (index: number) => {
      groupEditRef.current = { mode: "create", index };
      openCreateGroup(4, 2); // デフォルト: 4列×2行
    },
    [openCreateGroup],
  );

  /** サブグループ名を変更 → 独立ウィンドウを開く */
  const handleEditGroup = useCallback(
    (index: number, group: GroupItem) => {
      groupEditRef.current = { mode: "rename", index, group };
      openRenameGroup(group.label, group.gridColumns, group.gridRows, group.icon, group.iconColor, group.iconBase64, group.libraryIcon);
    },
    [openRenameGroup],
  );



  const widgetManager = useWidgetManager({
    activeTabId,
    onTabsUpdate: setTabs,
    onNotify: showNotification,
  });

  // ── ウィジェット選択ウィンドウ ──
  const { openWidgetSelectWindow } = useWidgetSelectWindow({
    settings,
    onSelect: widgetManager.handleWidgetSelect,
  });

  // ── ウィジェット設定ウィンドウ ──
  const { openWidgetSettingsWindow } = useWidgetSettingsWindow({
    settings,
    onSave: widgetManager.handleWidgetSettingsSave,
  });

  /** ウィジェット設定ボタン押下 → ウィンドウを開く */
  const handleWidgetSettings = useCallback(
    (index: number) => {
      const item = activeTab?.items[index];
      if (item?.type === "widget") {
        openWidgetSettingsWindow(item as WidgetItem, index);  // type narrowing through isWidgetItem not applicable here
      }
    },
    [activeTab, openWidgetSettingsWindow],
  );

  // ── P-10: ファイル選択ダイアログでの登録 ──
  const handleFilePickRegister = useCallback(
    async (index: number) => {
      try {
        const { open: dialogOpen } = await import("@tauri-apps/plugin-dialog");
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
          const { createLauncherItemFromPath } = await import("./utils/fileRegistration");
          const item = await createLauncherItemFromPath(selected as string);
          if (item) {
            tabManager.handleCellUpdate(index, item);
            showNotification(`「${item.label}」を登録しました`);
          }
        }
      } catch (e) {
        console.error("File dialog error:", e);
      }
    },
    [tabManager, showNotification],
  );

  // ── P-08: URL手動登録 ──
  const handleRegisterUrl = useCallback(
    (index: number) => {
      const url = window.prompt("登録するURLを入力してください", "https://");
      if (!url || !url.trim()) return;
      const trimmed = url.trim();
      let label = trimmed;
      try {
        label = new URL(trimmed).hostname;
      } catch { /* use url as label */ }
      const item = {
        id: crypto.randomUUID(),
        label,
        path: trimmed,
        type: "url" as const,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };
      tabManager.handleCellUpdate(index, item);
      showNotification(`「${label}」を登録しました`);
    },
    [tabManager, showNotification],
  );

  // ── グローバルホットキー ──
  useHotkey(settings.hotkey, settings.windowPosition, pinned);

  // P-43: アイテムごとのグローバルホットキー
  useEffect(() => {
    if (loading || tabs.length === 0) return;
    let mounted = true;
    const registered: string[] = [];

    (async () => {
      const { register: regShortcut } = await import("@tauri-apps/plugin-global-shortcut");
      const { invoke: invokeCmd } = await import("@tauri-apps/api/core");
      for (const tab of tabs) {
        for (const cell of tab.items) {
          if (!cell || cell.type === "widget" || cell.type === "group") continue;
          const item = cell as LauncherItem;
          if (!item.hotkey || item.hotkey === settings.hotkey) continue;
          try {
            await regShortcut(item.hotkey, async () => {
              if (!mounted) return;
              try {
                await invokeCmd("launch_app", { path: item.path, args: item.args ?? null });
              } catch (e) {
                console.warn("Hotkey launch failed:", item.label, e);
              }
            });
            registered.push(item.hotkey);
          } catch (e) {
            console.warn("Failed to register item hotkey:", item.hotkey, e);
          }
        }
      }
    })();

    return () => {
      mounted = false;
      (async () => {
        const { unregister: unregShortcut } = await import("@tauri-apps/plugin-global-shortcut");
        for (const key of registered) {
          try { await unregShortcut(key); } catch { /* ignore */ }
        }
      })();
    };
  }, [loading, tabs, settings.hotkey]);

  // ── 自動非表示（グループポップアップも連動して閉じる） ──
  useAutoHide(true, pinned, closeGroupPopup);

  // ── メインウィンドウがフォーカスを取り戻したらグループポップアップを閉じる ──
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;

    getCurrentWebviewWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (cancelled) return;
        if (focused) {
          closeGroupPopup();
        }
      })
      .then((fn) => {
        if (cancelled) { fn(); } else { unlisten = fn; }
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [closeGroupPopup]);

  // ── 初期データ読み込み ──
  useEffect(() => {
    loadData();
  }, [loadData]);

  // ── キーボードショートカット ──
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        setShowSearch(true);
        return;
      }
      if (e.key === "Escape") {
        if (showSearch) { setShowSearch(false); return; }
        if (pinned) return;
        const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
        getCurrentWebviewWindow().hide();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showSearch, pinned]);

  // ── ブラウザデフォルト右クリックメニュー抑制 ──
  useEffect(() => {
    const suppress = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", suppress);
    return () => document.removeEventListener("contextmenu", suppress);
  }, []);

  // ── P-26: Ctrl+マウスホイールでセルサイズ変更 ──
  useEffect(() => {
    const CELL_SIZES = [40, 48, 56, 64, 72, 80, 96, 112, 120];
    const handleWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      const currentIdx = CELL_SIZES.findIndex((s) => s >= settings.cellSize);
      const idx = currentIdx === -1 ? CELL_SIZES.length - 1 : currentIdx;
      let newIdx: number;
      if (e.deltaY < 0) {
        newIdx = Math.min(idx + 1, CELL_SIZES.length - 1);
      } else {
        newIdx = Math.max(idx - 1, 0);
      }
      if (CELL_SIZES[newIdx] !== settings.cellSize) {
        handleSettingsChange({ ...settings, cellSize: CELL_SIZES[newIdx] });
      }
    };
    window.addEventListener("wheel", handleWheel, { passive: false });
    return () => window.removeEventListener("wheel", handleWheel);
  }, [settings, handleSettingsChange]);

  // ── Tauri ネイティブ D&D ──
  const handleNativeHover = useCallback((cellIndex: number | null) => {
    setDragOverIndex(cellIndex);
  }, []);
  useDragDrop(handleNativeDrop, handleNativeHover);

  // ── 検索ナビゲーション ──
  const handleSearchNavigate = useCallback(
    (tabId: string, _index: number) => setActiveTabId(tabId),
    [setActiveTabId],
  );

  // ── ローディング画面 ──
  if (loading) {
    return (
      <div className="app" style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
        <span style={{ color: "var(--text-muted)" }}>Loading...</span>
      </div>
    );
  }

  // ── レンダリング ──
  const itemCount = activeTab?.items.filter(Boolean).length ?? 0;
  const totalSlots = (activeTab?.gridColumns ?? 8) * (activeTab?.gridRows ?? 4);

  return (
    <div
      className="app"
      style={{
        "--cell-size": `${settings.cellSize}px`,
        "--label-font-size": `${settings.labelFontSize ?? 10}px`,
      } as React.CSSProperties}
    >
      <CustomTitleBar pinned={pinned} onPinnedChange={setPinned} onOpenSettings={openSettingsWindow} appTitle={settings.appTitle} />

      <TabBar
        tabs={tabs} activeTabId={activeTabId}
        onSelectTab={setActiveTabId} onAddTab={handleAddTab}
        onRenameTab={handleRenameTab} onRemoveTab={handleRemoveTab}
        onReorderTabs={handleReorderTabs}
        onDuplicateTab={handleDuplicateTab}
        onTabColorChange={handleTabColorChange}
        isDraggingItem={isDraggingItem}
        onResizeTab={handleResizeTab}
      />

      {activeTab && (settings.viewMode ?? "grid") === "list" ? (
        <LauncherList
          tab={activeTab}
          onCellClick={handleCellClick}
          onCellClear={handleCellClear}
          onCellSwap={handleCellSwap}
          onAddWidget={openWidgetSelectWindow}
          onWidgetSettings={handleWidgetSettings}
          onLaunch={launcher.launch}
          onLaunchAdmin={launcher.launchAdmin}
          onOpenLocation={launcher.openLocation}
          onBrowseFolder={openFolderBrowser}
          onCellUpdate={tabManager.handleCellUpdate}
          onEditItem={handleEditItem}
          onCreateGroup={handleCreateGroup}
          onEditGroup={handleEditGroup}
          onFilePickRegister={handleFilePickRegister}
          onRegisterUrl={handleRegisterUrl}
          invalidPaths={invalidPaths}
        />
      ) : activeTab ? (
        <LauncherGrid
          tab={activeTab} showLabels={settings.showLabels}
          onCellClick={handleCellClick}
          onCellClear={handleCellClear} onCellSwap={handleCellSwap}
          onAddWidget={openWidgetSelectWindow}
          onWidgetSettings={handleWidgetSettings}
          onLaunch={launcher.launch} onLaunchAdmin={launcher.launchAdmin}
          onOpenLocation={launcher.openLocation}
          onBrowseFolder={openFolderBrowser}
          onCellUpdate={tabManager.handleCellUpdate}
          onEditItem={handleEditItem}
          onCreateGroup={handleCreateGroup}
          onEditGroup={handleEditGroup}
          externalDragOverIndex={dragOverIndex}
          onFilePickRegister={handleFilePickRegister}
          onRegisterUrl={handleRegisterUrl}
          onDragStateChange={setIsDraggingItem}
          invalidPaths={invalidPaths}
        />
      ) : null}

      <div className="statusbar">
        <span>{activeTab?.label} — {itemCount} アイテム / {totalSlots} スロット</span>
        <span>Ctrl+Space で表示切替</span>
      </div>

      {notification && <div className="toast">{notification}</div>}

      {showSearch && (
        <SearchBar
          tabs={tabs} onNavigate={handleSearchNavigate}
          onLaunch={launcher.launch} onClose={() => setShowSearch(false)}
        />
      )}

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

export default App;
