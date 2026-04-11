/* ============================================================
   useChildWindow - 子ウィンドウ（WebviewWindow）管理の汎用フック
   設定・ウィジェット選択・ウィジェット設定の3ウィンドウで共有。

   責務:
   - ウィンドウの生成・フォーカス管理
   - 「ready → init」イベントハンドシェイク
   - 閉じられた/結果返却イベントのリスン
   - setSettingsWindowOpen によるAutoHide連携
   ============================================================ */
import { useCallback, useEffect, useRef } from "react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { setSettingsWindowOpen } from "./useAutoHide";

export interface ChildWindowConfig {
  /** WebviewWindow の label（一意識别子） */
  label: string;
  /** HTMLファイルのURL */
  url: string;
  /** ウィンドウタイトル */
  title: string;
  /** ウィンドウ幅 */
  width: number;
  /** ウィンドウ高さ */
  height: number;
  /** リサイズ可否 (default: false) */
  resizable?: boolean;
  /** OS タイトルバー表示 (default: true) */
  decorations?: boolean;
  /** タスクバーに表示しない (default: false) */
  skipTaskbar?: boolean;
  /** true にするとウィンドウを破棄せず非表示で保持し、再表示時に再利用する */
  reusable?: boolean;
}

/** openWindow 呼び出し時に動的にオーバーライドするオプション */
export interface OpenWindowOverrides {
  width?: number;
  height?: number;
  x?: number;
  y?: number;
  /** false にすると center を使わず x/y で配置 (default: true) */
  center?: boolean;
}

export interface ChildWindowEvents<TInit, TResult> {
  /** 子ウィンドウが ready を通知するイベント名 (例: "settings-ready") */
  readyEvent: string;
  /** 親 → 子に初期データを送るイベント名 (例: "settings-init") */
  initEvent: string;
  /** 子 → 親に結果を返すイベント名 (例: "settings-save") */
  resultEvent: string;
  /** 子ウィンドウが閉じられた（キャンセル含む）イベント名 (例: "settings-closed") */
  closedEvent: string;
  /** 追加で listen するイベント群（プレビュー等） */
  extraListeners?: Array<{
    event: string;
    handler: (payload: unknown) => void;
  }>;
  /** 結果イベント受信時のコールバック */
  onResult: (payload: TResult) => void;
  /** ウィンドウ閉じ時のコールバック（オプション） */
  onClosed?: () => void;
  /** init イベントに送信するペイロードを返す関数 */
  getInitPayload: () => TInit | null;
}

/**
 * 子ウィンドウの生成・通信・ライフサイクルを管理するジェネリックフック。
 *
 * @returns openWindow - ウィンドウを開く関数（ペイロード更新後に呼ぶ）
 */
export function useChildWindow<TInit, TResult>(
  config: ChildWindowConfig,
  events: ChildWindowEvents<TInit, TResult>,
) {
  const windowRef = useRef<WebviewWindow | null>(null);

  /** ウィンドウを開く（既に開いている場合はリユースまたは再作成） */
  const openWindow = useCallback(async (overrides?: OpenWindowOverrides) => {
    const w = overrides?.width ?? config.width;
    const h = overrides?.height ?? config.height;
    const useCenter = overrides?.center ?? true;

    // ── リユースモード: 既存ウィンドウを再利用（高速パス） ──
    if (config.reusable && windowRef.current) {
      try {
        const win = windowRef.current;
        await win.setSize(new LogicalSize(w, h));
        if (!useCenter && overrides?.x != null && overrides?.y != null) {
          await win.setPosition(new LogicalPosition(overrides.x, overrides.y));
        }
        // init データを直接送信（子ウィンドウは既にリスン中）
        const payload = events.getInitPayload();
        if (payload) await emit(events.initEvent, payload);
        await win.show();
        await win.setFocus();
        if (!config.reusable) setSettingsWindowOpen(true);
        return;
      } catch {
        // ウィンドウが外部で破棄済み → 通常の作成フローへフォールバック
        windowRef.current = null;
      }
    }

    // 既存ウィンドウがある場合は閉じてから再作成（サイズ不整合を回避）
    if (windowRef.current) {
      try {
        await windowRef.current.close();
      } catch { /* already closed */ }
      windowRef.current = null;
      setSettingsWindowOpen(false);
      // 閉じた後のタイミングを待つ（label 再利用の衝突回避）
      await new Promise((r) => setTimeout(r, 200));
    }

    // ちらつき防止: 非表示で作成 → サイズ確定 → show()
    const win = new WebviewWindow(config.label, {
      url: config.url,
      title: config.title,
      width: w,
      height: h,
      visible: false,               // ← 最初は非表示
      resizable: config.resizable ?? false,
      center: useCenter,
      ...((!useCenter && overrides?.x != null && overrides?.y != null) ? { x: overrides.x, y: overrides.y } : {}),
      decorations: config.decorations ?? true,
      skipTaskbar: config.skipTaskbar ?? false,
      transparent: true,
      alwaysOnTop: true,
    });

    windowRef.current = win;
    if (!config.reusable) setSettingsWindowOpen(true);

    // ウィンドウ作成成功 → サイズ・位置を確定してから表示
    win.once("tauri://created", async () => {
      try {
        await win.setSize(new LogicalSize(w, h));
        if (!useCenter && overrides?.x != null && overrides?.y != null) {
          await win.setPosition(new LogicalPosition(overrides.x, overrides.y));
        }
      } catch { /* ignore */ }
      // サイズ確定後に表示（ちらつき防止）
      await win.show();
      await win.setFocus();
    });
    win.once("tauri://error", (e) => {
      console.error(`[useChildWindow] Window creation failed:`, e);
      windowRef.current = null;
      setSettingsWindowOpen(false);
    });

    if (config.reusable) {
      // リユースモード: OS レベルのクローズリクエストを非表示に変換
      win.onCloseRequested(async (event) => {
        event.preventDefault();
        await invoke("hide_webview_window", { label: config.label });
      });
    } else {
      win.onCloseRequested(() => {
        windowRef.current = null;
        setSettingsWindowOpen(false);
      });
    }
  }, [config, events]);

  /** イベントリスナーの登録 */
  useEffect(() => {
    // 子ウィンドウが準備完了 → init データを送信
    const unlistenReady = listen(events.readyEvent, () => {
      const payload = events.getInitPayload();
      if (payload) emit(events.initEvent, payload);
    });

    // 結果イベント → コールバック実行
    const unlistenResult = listen<TResult>(events.resultEvent, async (event) => {
      events.onResult(event.payload);
      if (config.reusable && windowRef.current) {
        try { await invoke("hide_webview_window", { label: config.label }); } catch { /* ignore */ }
      } else {
        windowRef.current = null;
      }
      if (!config.reusable) setSettingsWindowOpen(false);
    });

    // 閉じられた（キャンセル含む）→ reusable なら Rust 経由で hide
    const unlistenClosed = listen(events.closedEvent, async () => {
      if (config.reusable && windowRef.current) {
        try { await invoke("hide_webview_window", { label: config.label }); } catch { /* ignore */ }
      } else {
        windowRef.current = null;
      }
      if (!config.reusable) setSettingsWindowOpen(false);
      events.onClosed?.();
    });

    // 追加リスナー（プレビュー等）
    const extraUnlistens = (events.extraListeners ?? []).map(({ event, handler }) =>
      listen(event, (e) => handler(e.payload))
    );

    return () => {
      unlistenReady.then((fn) => fn());
      unlistenResult.then((fn) => fn());
      unlistenClosed.then((fn) => fn());
      extraUnlistens.forEach((p) => p.then((fn) => fn()));
    };
  }, [events, config.reusable]);

  /** 子ウィンドウを閉じる（reusable の場合は Rust 経由で非表示） */
  const closeWindow = useCallback(async () => {
    if (windowRef.current) {
      try {
        if (config.reusable) {
          await invoke("hide_webview_window", { label: config.label });
        } else {
          await windowRef.current.close();
          windowRef.current = null;
        }
      } catch {
        windowRef.current = null;
      }
      if (!config.reusable) setSettingsWindowOpen(false);
    }
  }, [config.reusable, config.label]);

  // コンポーネントアンマウント時にリユースウィンドウを実際に破棄
  useEffect(() => {
    return () => {
      if (config.reusable && windowRef.current) {
        windowRef.current.close().catch((e) => console.warn("Failed to close child window:", e));
        windowRef.current = null;
      }
    };
  }, [config.reusable]);

  return { openWindow, closeWindow };
}
