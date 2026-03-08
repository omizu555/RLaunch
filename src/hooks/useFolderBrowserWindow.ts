/* ============================================================
   useFolderBrowserWindow - フォルダブラウザウィンドウ管理（useChildWindow ベース）
   グループポップアップと同様のカーソル位置配置・リユース・フォーカス喪失クローズ。
   ============================================================ */
import { useCallback, useMemo, useRef } from "react";
import { useChildWindow } from "./useChildWindow";
import { useStableRef } from "./useStableRef";
import { setFolderBrowserOpen } from "./useAutoHide";
import { positionWindowAtCursor } from "../utils/windowPositioning";

interface UseFolderBrowserWindowOptions {
  onLaunchFile: (path: string) => void;
  onOpenExplorer: (path: string) => void;
  onClosed?: () => void;
}

interface FolderBrowserInitPayload {
  path: string;
}

export function useFolderBrowserWindow({
  onLaunchFile,
  onOpenExplorer,
  onClosed,
}: UseFolderBrowserWindowOptions) {
  const pendingPath = useRef<string | null>(null);
  const onLaunchFileRef = useStableRef(onLaunchFile);
  const onOpenExplorerRef = useStableRef(onOpenExplorer);
  const onClosedRef = useStableRef(onClosed);

  const events = useMemo(
    () => ({
      readyEvent: "folder-browser-ready",
      initEvent: "folder-browser-init",
      resultEvent: "folder-browser-launch",
      closedEvent: "folder-browser-closed",

      getInitPayload: () =>
        pendingPath.current ? { path: pendingPath.current } : null,

      onResult: (payload: { path: string }) => {
        onLaunchFileRef.current(payload.path);
        setFolderBrowserOpen(false);
      },

      onClosed: () => {
        setFolderBrowserOpen(false);
        onClosedRef.current?.();
      },

      extraListeners: [
        {
          event: "folder-browser-open-explorer",
          handler: (payload: unknown) => {
            const p = payload as { path: string };
            onOpenExplorerRef.current(p.path);
          },
        },
      ],
    }),
    [],
  );

  const config = useMemo(
    () => ({
      label: "folder-browser",
      url: "src/folder-browser.html",
      title: "📂 フォルダ参照",
      width: 500,
      height: 460,
      resizable: true,
      decorations: false,
      skipTaskbar: true,
      reusable: true,
    }),
    [],
  );

  const { openWindow, closeWindow } = useChildWindow<FolderBrowserInitPayload, { path: string }>(
    config,
    events,
  );

  /** フォルダブラウザを開く（クリック位置を基準に配置） */
  const openFolderBrowser = useCallback(
    async (folderPath: string) => {
      pendingPath.current = folderPath;

      const w = 500;
      const h = 460;

      // カーソル位置 + モニター作業領域を取得し、マルチモニター対応で配置
      const overrides = await positionWindowAtCursor(w, h);

      await openWindow(overrides);
      setFolderBrowserOpen(true);
    },
    [openWindow],
  );

  /** フォルダブラウザを閉じる */
  const closeFolderBrowser = useCallback(async () => {
    setFolderBrowserOpen(false);
    await closeWindow();
  }, [closeWindow]);

  return { openFolderBrowser, closeFolderBrowser };
}
