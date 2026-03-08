/* ============================================================
   日付カレンダー Canvas 描画
   ============================================================ */
import type { DateCalendarConfig } from "../types";

export function drawDateCalendar(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: DateCalendarConfig
) {
  ctx.clearRect(0, 0, w, h);

  // 背景
  if (config.backgroundColor && config.backgroundColor !== "transparent") {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }

  const now = new Date();
  const month = now.getMonth() + 1;
  const day = now.getDate();
  const weekdays = ["日", "月", "火", "水", "木", "金", "土"];
  const weekday = weekdays[now.getDay()];

  ctx.save();

  // 月（上部バー風）
  const monthBarH = h * 0.28;
  ctx.fillStyle = config.accentColor;
  ctx.fillRect(0, 0, w, monthBarH);

  const monthFontSize = Math.max(7, w * 0.14);
  ctx.font = `bold ${monthFontSize}px sans-serif`;
  ctx.fillStyle = "#ffffff";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  let monthText = `${month}月`;
  if (config.showYear) {
    monthText = `${now.getFullYear()}年 ${monthText}`;
  }
  ctx.fillText(monthText, w / 2, monthBarH / 2);

  // 日付（大きく中央に）
  const dayFontSize = Math.max(14, w * 0.38);
  ctx.font = `bold ${dayFontSize}px sans-serif`;
  ctx.fillStyle = config.textColor;

  let dayY = h * 0.6;
  if (!config.showWeekday) {
    dayY = h * 0.65;
  }
  ctx.fillText(`${day}`, w / 2, dayY);

  // 曜日
  if (config.showWeekday) {
    const weekdayFontSize = Math.max(7, w * 0.13);
    ctx.font = `${weekdayFontSize}px sans-serif`;
    ctx.fillStyle = config.textColor;
    ctx.globalAlpha = 0.6;

    // 日曜日は赤、土曜日は青
    if (now.getDay() === 0) ctx.fillStyle = "#f38ba8";
    else if (now.getDay() === 6) ctx.fillStyle = "#89b4fa";

    ctx.globalAlpha = 0.8;
    ctx.fillText(weekday + "曜日", w / 2, h * 0.85);
    ctx.globalAlpha = 1.0;
  }

  ctx.restore();
}
