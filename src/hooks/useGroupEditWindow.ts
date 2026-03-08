/* ============================================================
   useGroupEditWindow - グループ編集ウィンドウ管理（useChildWindow ベース）
   ============================================================ */
import { useCallback, useMemo, useRef } from "react";
import { useChildWindow } from "./useChildWindow";
import { useStableRef } from "./useStableRef";
import type { GroupEditInitPayload, GroupEditResultPayload } from "../components/GroupEditWindow";

interface UseGroupEditWindowOptions {
  onSave: (payload: GroupEditResultPayload) => void;
  onClosed?: () => void;
}

export function useGroupEditWindow({ onSave, onClosed }: UseGroupEditWindowOptions) {
  const pendingInit = useRef<GroupEditInitPayload | null>(null);
  const onSaveRef = useStableRef(onSave);
  const onClosedRef = useStableRef(onClosed);

  const events = useMemo(
    () => ({
      readyEvent: "group-edit-ready",
      initEvent: "group-edit-init",
      resultEvent: "group-edit-save",
      closedEvent: "group-edit-closed",

      getInitPayload: () => pendingInit.current,

      onResult: (payload: GroupEditResultPayload) => {
        onSaveRef.current(payload);
      },

      onClosed: () => {
        onClosedRef.current?.();
      },
    }),
    [],
  );

  const config = useMemo(
    () => ({
      label: "group-edit",
      url: "src/group-edit.html",
      title: "📁 グループ編集",
      width: 380,
      height: 360,
      resizable: false,
      decorations: false,
    }),
    [],
  );

  const { openWindow } = useChildWindow<GroupEditInitPayload, GroupEditResultPayload>(
    config,
    events,
  );

  /** 新規作成モードで開く */
  const openCreateGroup = useCallback(
    async (defaultColumns: number, defaultRows: number) => {
      pendingInit.current = {
        mode: "create",
        label: "新しいグループ",
        columns: defaultColumns,
        rows: defaultRows,
      };
      await openWindow();
    },
    [openWindow],
  );

  /** 名前変更（編集）モードで開く */
  const openRenameGroup = useCallback(
    async (label: string, columns: number, rows: number, icon?: string, iconColor?: string, iconBase64?: string, libraryIcon?: string) => {
      pendingInit.current = {
        mode: "rename",
        label,
        columns,
        rows,
        icon,
        iconColor,
        iconBase64,
        libraryIcon,
      };
      await openWindow();
    },
    [openWindow],
  );

  return { openCreateGroup, openRenameGroup };
}
