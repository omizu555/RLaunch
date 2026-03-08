/* ============================================================
   CustomTitleBar - カスタムタイトルバー
   ============================================================ */
import "./CustomTitleBar.css";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { setWindowDragging } from "../hooks/useAutoHide";

interface CustomTitleBarProps {
  pinned: boolean;
  onPinnedChange: (pinned: boolean) => void;
  onOpenSettings: () => void;
  appTitle?: string;
}

export function CustomTitleBar({
  pinned,
  onPinnedChange,
  onOpenSettings,
  appTitle,
}: CustomTitleBarProps) {
  const appWindow = getCurrentWebviewWindow();
  const handleHide = () => appWindow.hide();

  // ドラッグ開始・終了でフラグ制御（フォーカス喪失抑制用）
  const handleDragMouseDown = () => {
    setWindowDragging(true);
    // document レベルで mouseup を捕捉（Tauri ネイティブドラッグでは
    // 元の要素に mouseup が届かないことがあるため）
    const onMouseUp = () => {
      // 少し遅延してフォーカス復帰を待つ
      setTimeout(() => setWindowDragging(false), 400);
      document.removeEventListener("mouseup", onMouseUp);
    };
    document.addEventListener("mouseup", onMouseUp);
  };

  const titleText = appTitle ?? "RLaunch";

  return (
    <div className="titlebar">
      <button className="titlebar-menu-btn" title="設定" onClick={onOpenSettings}>
        ☰
      </button>
      {titleText ? (
        <span
          className="titlebar-title"
          data-tauri-drag-region
          onMouseDown={handleDragMouseDown}
        >
          {titleText}
        </span>
      ) : null}
      <div
        className="titlebar-spacer"
        data-tauri-drag-region
        onMouseDown={handleDragMouseDown}
      />
      <div className="titlebar-controls">
        <button
          className={`titlebar-btn pin-btn ${pinned ? "pinned" : ""}`}
          title={pinned ? "ピン留め解除 (表示固定中)" : "ピン留め (表示を固定)"}
          onClick={() => onPinnedChange(!pinned)}
        >
          <span className="pin-icon">📌</span>
        </button>
        <button className="titlebar-btn" title="最小化" onClick={handleHide}>
          ─
        </button>
        <button className="titlebar-btn close" title="閉じる（トレイに格納）" onClick={handleHide}>
          ✕
        </button>
      </div>
    </div>
  );
}
