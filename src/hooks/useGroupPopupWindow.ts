/* ============================================================
   useGroupPopupWindow - グループポップアップウィンドウ管理
   useChildWindow ベース。グループクリック時に独立ウィンドウを開く。

   ウィンドウサイズはグループの gridColumns × gridRows から自動算出。
   位置はメインウィンドウの右下付近に配置。
   ============================================================ */
import { useCallback, useMemo, useRef } from "react";
import { useChildWindow } from "./useChildWindow";
import { useStableRef } from "./useStableRef";
import { setGroupPopupOpen } from "./useAutoHide";
import type { GroupPopupInitPayload, GroupPopupUpdatePayload } from "../components/GroupPopupWindow";
import type { GroupItem, LauncherItem } from "../types";
import { calcPopupSize } from "../utils/fileUtils";
import { positionWindowAtCursor } from "../utils/windowPositioning";

interface UseGroupPopupWindowOptions {
  /** アイテム起動通知（起動回数更新等） */
  onLaunch?: (item: LauncherItem) => void;
  /** グループの中身が変更されたとき（アイテム除去等） */
  onGroupUpdate?: (group: GroupItem) => void;
  /** ウィンドウが閉じられたとき */
  onClosed?: () => void;
}

export function useGroupPopupWindow({
  onLaunch,
  onGroupUpdate,
  onClosed,
}: UseGroupPopupWindowOptions) {
  const pendingGroup = useRef<GroupItem | null>(null);
  const onLaunchRef = useStableRef(onLaunch);
  const onGroupUpdateRef = useStableRef(onGroupUpdate);
  const onClosedRef = useStableRef(onClosed);

  const events = useMemo(
    () => ({
      readyEvent: "group-popup-ready",
      initEvent: "group-popup-init",
      resultEvent: "group-popup-launch",
      closedEvent: "group-popup-closed",

      getInitPayload: (): GroupPopupInitPayload | null =>
        pendingGroup.current ? { group: pendingGroup.current } : null,

      onResult: (payload: { item: LauncherItem }) => {
        onLaunchRef.current?.(payload.item);
        setGroupPopupOpen(false);
      },

      onClosed: () => {
        setGroupPopupOpen(false);
        onClosedRef.current?.();
      },

      extraListeners: [
        {
          event: "group-popup-update",
          handler: (payload: unknown) => {
            const p = payload as GroupPopupUpdatePayload;
            onGroupUpdateRef.current?.(p.group);
          },
        },
      ],
    }),
    [],
  );

  const config = useMemo(
    () => ({
      label: "group-popup",
      url: "src/group-popup.html",
      title: "📂 グループ",
      width: 400,
      height: 340,
      resizable: false,
      decorations: false,
      skipTaskbar: true,
      reusable: true,
    }),
    [],
  );

  const { openWindow, closeWindow } = useChildWindow<GroupPopupInitPayload, { item: LauncherItem }>(
    config,
    events,
  );

  /** グループポップアップを開く（クリック位置を基準に配置） */
  const openGroupPopup = useCallback(
    async (group: GroupItem) => {
      pendingGroup.current = group;

      // グループのグリッドサイズからウィンドウサイズを算出（論理ピクセル）
      const { w, h } = calcPopupSize(group.gridColumns, group.gridRows);

      // カーソル位置 + モニター作業領域を取得し、マルチモニター対応で配置
      const overrides = await positionWindowAtCursor(w, h);

      await openWindow(overrides);
      setGroupPopupOpen(true);
    },
    [openWindow],
  );

  /** グループポップアップを閉じる */
  const closeGroupPopup = useCallback(async () => {
    setGroupPopupOpen(false);
    await closeWindow();
  }, [closeWindow]);

  return { openGroupPopup, closeGroupPopup };
}
