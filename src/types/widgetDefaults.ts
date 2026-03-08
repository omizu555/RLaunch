/* ============================================================
   MyLauncher - ウィジェットのデフォルト設定値・ラベル・ヘルパー
   ============================================================ */
import type {
  WidgetType,
  AnalogClockConfig,
  DigitalClockConfig,
  CountdownTimerConfig,
  SystemMonitorConfig,
  DateCalendarConfig,
} from "./widgets";

/* ── デフォルト値 ── */

export const DEFAULT_ANALOG_CLOCK: AnalogClockConfig = {
  backgroundColor: "#1e1e2e",
  dialStyle: "simple",
  hourHandColor: "#ffffff",
  minuteHandColor: "#ffffff",
  showSecondHand: true,
  secondHandColor: "#f38ba8",
  timezone: "local",
};

export const DEFAULT_DIGITAL_CLOCK: DigitalClockConfig = {
  format: "24h",
  fontStyle: "7segment",
  showDate: true,
  showWeekday: false,
  showSeconds: false,
  textColor: "#89b4fa",
  backgroundColor: "transparent",
  timezone: "local",
};

export const DEFAULT_COUNTDOWN_TIMER: CountdownTimerConfig = {
  targetDate: new Date(Date.now() + 7 * 24 * 3600 * 1000).toISOString(),
  targetLabel: "目標",
  textColor: "#f9e2af",
  backgroundColor: "transparent",
  showDays: true,
  showHours: true,
};

export const DEFAULT_CPU_MONITOR: SystemMonitorConfig = {
  target: "cpu",
  displayStyle: "gauge",
  textColor: "#ffffff",
  gaugeColor: "#a6e3a1",
  warningThreshold: 80,
  warningColor: "#f38ba8",
  backgroundColor: "transparent",
};

export const DEFAULT_MEMORY_MONITOR: SystemMonitorConfig = {
  target: "memory",
  displayStyle: "gauge",
  textColor: "#ffffff",
  gaugeColor: "#89b4fa",
  warningThreshold: 80,
  warningColor: "#f38ba8",
  backgroundColor: "transparent",
};

export const DEFAULT_DATE_CALENDAR: DateCalendarConfig = {
  showWeekday: true,
  showYear: false,
  textColor: "#ffffff",
  backgroundColor: "transparent",
  accentColor: "#f38ba8",
};

/* ── ヘルパー関数 ── */

export function getDefaultWidgetConfig(widgetType: WidgetType): Record<string, unknown> {
  switch (widgetType) {
    case "analog-clock": return { ...DEFAULT_ANALOG_CLOCK };
    case "digital-clock": return { ...DEFAULT_DIGITAL_CLOCK };
    case "countdown-timer": return { ...DEFAULT_COUNTDOWN_TIMER };
    case "cpu-monitor": return { ...DEFAULT_CPU_MONITOR };
    case "memory-monitor": return { ...DEFAULT_MEMORY_MONITOR };
    case "date-calendar": return { ...DEFAULT_DATE_CALENDAR };
    default: return {};
  }
}

export function getDefaultUpdateInterval(widgetType: WidgetType): number {
  switch (widgetType) {
    case "analog-clock": return 1000;
    case "digital-clock": return 1000;
    case "countdown-timer": return 1000;
    case "cpu-monitor": return 2000;
    case "memory-monitor": return 5000;
    case "date-calendar": return 60000;
    default: return 1000;
  }
}

export const WIDGET_LABELS: Record<string, string> = {
  "analog-clock": "アナログ時計",
  "digital-clock": "デジタル時計",
  "countdown-timer": "カウントダウン",
  "cpu-monitor": "CPU モニター",
  "memory-monitor": "メモリモニター",
  "date-calendar": "日付カレンダー",
};
