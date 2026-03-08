/* ============================================================
   MyLauncher - Layout & UI Constants
   マジックナンバーを排除し、一箇所で管理
   ============================================================ */

/** ウィンドウサイズ計算用レイアウト定数 (px) */
export const LAYOUT = {
  /** グリッドセル間ギャップ（CSS --grid-gap と一致させること） */
  GRID_GAP: 6,
  /** グリッドエリアのパディング合計 (left + right = 10+10) */
  GRID_PADDING: 20,
  /** ボーダー等の余白 */
  BORDER_EXTRA: 2,
  /** タイトルバー高さ（CSS --titlebar-height と一致） */
  TITLEBAR_HEIGHT: 36,
  /** タブバー高さ＋ボーダー */
  TABBAR_HEIGHT: 36,
  /** ステータスバー高さ */
  STATUSBAR_HEIGHT: 28,
} as const;

/** 通知表示時間 (ms) */
export const NOTIFICATION_DURATION = 3000;
