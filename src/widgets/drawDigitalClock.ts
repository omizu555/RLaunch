/* ============================================================
   デジタル時計 Canvas 描画
   ============================================================ */
import type { DigitalClockConfig } from "../types";

export function drawDigitalClock(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: DigitalClockConfig
) {
  ctx.clearRect(0, 0, w, h);

  // 背景
  if (config.backgroundColor && config.backgroundColor !== "transparent") {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }

  const now = new Date();
  let hours = now.getHours();
  let ampm = "";

  if (config.format === "12h") {
    ampm = hours >= 12 ? "PM" : "AM";
    hours = hours % 12 || 12;
  }

  const minutes = now.getMinutes().toString().padStart(2, "0");
  const seconds = now.getSeconds().toString().padStart(2, "0");
  const hoursStr = hours.toString().padStart(2, "0");

  let timeStr = `${hoursStr}:${minutes}`;
  if (config.showSeconds) {
    timeStr += `:${seconds}`;
  }

  // フォント選択
  let fontFamily = "monospace";
  if (config.fontStyle === "7segment") {
    fontFamily = "'Courier New', monospace";
  } else if (config.fontStyle === "digital") {
    fontFamily = "'Consolas', 'Courier New', monospace";
  }

  // 時刻描画
  const mainFontSize = Math.max(10, w * 0.28);
  ctx.save();
  ctx.font = `bold ${mainFontSize}px ${fontFamily}`;
  ctx.fillStyle = config.textColor;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  let y = h * 0.4;
  if (!config.showDate && !config.showWeekday) {
    y = h * 0.5;
  }

  ctx.fillText(timeStr, w / 2, y);

  // AM/PM
  if (config.format === "12h") {
    ctx.font = `bold ${mainFontSize * 0.35}px ${fontFamily}`;
    ctx.fillText(ampm, w / 2, y + mainFontSize * 0.55);
  }

  // 日付
  if (config.showDate || config.showWeekday) {
    const subFontSize = Math.max(7, w * 0.13);
    ctx.font = `${subFontSize}px sans-serif`;
    ctx.fillStyle = config.textColor;
    ctx.globalAlpha = 0.7;

    let dateStr = "";
    if (config.showDate) {
      const m = (now.getMonth() + 1).toString().padStart(2, "0");
      const d = now.getDate().toString().padStart(2, "0");
      dateStr = `${m}/${d}`;
    }
    if (config.showWeekday) {
      const days = ["日", "月", "火", "水", "木", "金", "土"];
      dateStr += (dateStr ? " " : "") + `(${days[now.getDay()]})`;
    }

    const dateY = config.format === "12h" ? y + mainFontSize * 0.8 : y + mainFontSize * 0.6;
    ctx.fillText(dateStr, w / 2, dateY);
    ctx.globalAlpha = 1.0;
  }

  ctx.restore();
}
