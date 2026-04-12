/* ============================================================
   MyLauncher - ランチャーコアデータ型
   ============================================================ */
import type { WidgetType } from "./widgets";

export type ItemType = "executable" | "shortcut" | "folder" | "url" | "document" | "widget" | "group";

export type WindowState = "normal" | "maximized" | "minimized";

/** フォルダ型アイテムのクリック時動作 */
export type FolderAction = "open" | "browse";

/** ランチャーに登録されたアイテム */
export interface LauncherItem {
  id: string;
  label: string;
  path: string;
  args?: string;
  workingDir?: string;
  iconBase64?: string;
  iconPath?: string;
  /** アイコンライブラリから選択したアイコンファイル名 */
  libraryIcon?: string;
  type: ItemType;
  runAs?: boolean;
  windowState?: WindowState;
  hotkey?: string;
  /** フォルダ型: クリック時の動作 ("open" = Explorer で開く, "browse" = 階層ブラウズ) */
  folderAction?: FolderAction;
  /** 起動回数 */
  launchCount?: number;
  /** 最終起動日時 (ISO 8601) */
  lastLaunchedAt?: string;
  createdAt: string;
  updatedAt: string;
}

/** ウィジェットアイテム */
export interface WidgetItem {
  id: string;
  type: "widget";
  widgetType: WidgetType;
  label?: string;
  config: Record<string, unknown>;
  updateInterval: number;
  /** P-30: ウィジェットの横スパン (デフォルト1) */
  colSpan?: number;
  /** P-30: ウィジェットの縦スパン (デフォルト1) */
  rowSpan?: number;
  createdAt: string;
  updatedAt: string;
}

/** サブグループ: 子アイテムを格納するフォルダ的な存在 */
export interface GroupItem {
  id: string;
  type: "group";
  label: string;
  /** P-35: カスタム絵文字アイコン */
  icon?: string;
  iconColor?: string;
  /** アイコンライブラリから選択したアイコン (data URL) */
  iconBase64?: string;
  /** アイコンライブラリのファイル名 */
  libraryIcon?: string;
  items: GridCell[];
  gridColumns: number;
  gridRows: number;
  /** 表示モード（undefined = 親タブ/全体設定を継承） */
  viewMode?: "grid" | "list";
  /** リスト列数（undefined = 親タブ/全体設定を継承） */
  listColumns?: number;
  createdAt: string;
  updatedAt: string;
}

/** グリッドセル: アプリ / ウィジェット / グループ / 空 */
export type GridCell = LauncherItem | WidgetItem | GroupItem | null;

// ── 型ガード関数 ──

export function isWidgetItem(cell: GridCell): cell is WidgetItem {
  return cell !== null && cell.type === "widget";
}

export function isGroupItem(cell: GridCell): cell is GroupItem {
  return cell !== null && cell.type === "group";
}

export function isLauncherItem(cell: GridCell): cell is LauncherItem {
  return cell !== null && cell.type !== "widget" && cell.type !== "group";
}

/** タブデータ */
export interface Tab {
  id: string;
  label: string;
  order: number;
  /** P-04: タブカラーマーカー */
  color?: string;
  gridColumns: number;
  gridRows: number;
  items: GridCell[];
  /** タブ個別: 表示モード (未設定時は全体設定にフォールバック) */
  viewMode?: "grid" | "list";
  /** タブ個別: リスト表示の列数 (未設定時は全体設定にフォールバック) */
  listColumns?: number;
}
