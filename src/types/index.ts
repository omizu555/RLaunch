/* ============================================================
   MyLauncher - 型定義バレルエクスポート
   全型を `import { ... } from "../types"` で使えるようにする。
   ============================================================ */

// ── ランチャー基本型 ──
export type { ItemType, WindowState, FolderAction, LauncherItem, WidgetItem, GroupItem, GridCell, Tab } from "./core";

// ── ウィジェット型 ──
export type {
  BuiltinWidgetType,
  WidgetType,
  SelectOption,
  ConfigSchemaField,
  WidgetManifest,
  AnalogClockConfig,
  DigitalClockConfig,
  CountdownTimerConfig,
  SystemMonitorConfig,
  DateCalendarConfig,
  WidgetConfig,
} from "./widgets";
export { BUILTIN_WIDGET_IDS, isBuiltinWidget } from "./widgets";

// ── ウィジェットデフォルト値 ──
export {
  DEFAULT_ANALOG_CLOCK,
  DEFAULT_DIGITAL_CLOCK,
  DEFAULT_COUNTDOWN_TIMER,
  DEFAULT_CPU_MONITOR,
  DEFAULT_MEMORY_MONITOR,
  DEFAULT_DATE_CALENDAR,
  getDefaultWidgetConfig,
  getDefaultUpdateInterval,
  WIDGET_LABELS,
} from "./widgetDefaults";

// ── 設定型 ──
export type { AppSettings } from "./settings";
export { DEFAULT_SETTINGS } from "./settings";
