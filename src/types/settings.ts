/* ============================================================
   MyLauncher - アプリ設定の型定義
   ============================================================ */

/** アプリ全体設定 */
export interface AppSettings {
  autoStart: boolean;
  hotkey: string;
  defaultGridColumns: number;
  defaultGridRows: number;
  cellSize: number;
  showLabels: boolean;
  theme: string;
  windowEffect: "none" | "mica" | "acrylic";
  autoHide: boolean;
  hideOnLaunch: boolean;
  windowPosition: "center" | "cursor" | "remember";
  windowX?: number;
  windowY?: number;
  appTitle: string;
  /** P-29: ラベルフォントサイズ (px) */
  labelFontSize?: number;
  /** P-50: ウィンドウ不透明度 (0-100, 100=完全不透明) */
  windowOpacity?: number;
  /** 表示モード: grid=アイコングリッド, list=テキストリスト */
  viewMode?: "grid" | "list";
}

/** デフォルト設定 */
export const DEFAULT_SETTINGS: AppSettings = {
  autoStart: false,
  hotkey: "Ctrl+Space",
  defaultGridColumns: 8,
  defaultGridRows: 4,
  cellSize: 64,
  showLabels: true,
  theme: "dark",
  windowEffect: "none",
  autoHide: true,
  hideOnLaunch: true,
  windowPosition: "cursor",
  appTitle: "RLaunch",
  labelFontSize: 10,
  windowOpacity: 100,
  viewMode: "grid",
};
