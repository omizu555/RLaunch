/* ============================================================
   MyLauncher - ウィジェット関連の型定義
   ============================================================ */

/** ビルトインウィジェット ID（互換維持用） */
export type BuiltinWidgetType =
  | "analog-clock"
  | "digital-clock"
  | "countdown-timer"
  | "cpu-monitor"
  | "memory-monitor"
  | "date-calendar";

/** ウィジェットタイプ: ビルトイン or プラグイン(任意の文字列) */
export type WidgetType = string;

export const BUILTIN_WIDGET_IDS: readonly string[] = [
  "analog-clock", "digital-clock", "countdown-timer",
  "cpu-monitor", "memory-monitor", "date-calendar",
];

export function isBuiltinWidget(id: string): id is BuiltinWidgetType {
  return BUILTIN_WIDGET_IDS.includes(id);
}

/* ── ウィジェットマニフェスト (Rust と同じ構造) ── */

export interface SelectOption {
  value: string;
  label: string;
}

export interface ConfigSchemaField {
  key: string;
  type: "color" | "checkbox" | "select" | "text" | "number" | "datetime" | "file";
  label: string;
  default: unknown;
  options?: SelectOption[];
  min?: number;
  max?: number;
  step?: number;
}

export interface WidgetManifest {
  id: string;
  label: string;
  author: string;
  description: string;
  emoji: string;
  version: string;
  updateInterval: number;
  needsSystemInfo: boolean;
  configSchema: ConfigSchemaField[];
}

/* ── ウィジェット個別設定 (ビルトイン用型付き) ── */

export interface AnalogClockConfig {
  backgroundColor: string;
  dialStyle: "simple" | "roman" | "dots" | "none";
  hourHandColor: string;
  minuteHandColor: string;
  showSecondHand: boolean;
  secondHandColor: string;
  timezone: string;
}

export interface DigitalClockConfig {
  format: "12h" | "24h";
  fontStyle: "7segment" | "digital" | "monospace";
  showDate: boolean;
  showWeekday: boolean;
  showSeconds: boolean;
  textColor: string;
  backgroundColor: string;
  timezone: string;
}

export interface CountdownTimerConfig {
  targetDate: string;
  targetLabel: string;
  textColor: string;
  backgroundColor: string;
  showDays: boolean;
  showHours: boolean;
}

export interface SystemMonitorConfig {
  target: "cpu" | "memory";
  displayStyle: "gauge" | "bar" | "text";
  textColor: string;
  gaugeColor: string;
  warningThreshold: number;
  warningColor: string;
  backgroundColor: string;
}

export interface DateCalendarConfig {
  showWeekday: boolean;
  showYear: boolean;
  textColor: string;
  backgroundColor: string;
  accentColor: string;
}

export type WidgetConfig =
  | AnalogClockConfig
  | DigitalClockConfig
  | CountdownTimerConfig
  | SystemMonitorConfig
  | DateCalendarConfig;
