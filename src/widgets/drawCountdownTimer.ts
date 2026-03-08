/* ============================================================
   カウントダウンタイマー Canvas 描画
   ============================================================ */
import type { CountdownTimerConfig } from "../types";

export function drawCountdownTimer(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: CountdownTimerConfig
) {
  ctx.clearRect(0, 0, w, h);

  // 背景
  if (config.backgroundColor && config.backgroundColor !== "transparent") {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }

  const now = Date.now();
  const target = new Date(config.targetDate).getTime();
  const diff = target - now;

  const isExpired = diff <= 0;
  const absDiff = Math.abs(diff);

  const totalSeconds = Math.floor(absDiff / 1000);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  ctx.save();

  // ラベル
  const labelFontSize = Math.max(7, w * 0.12);
  ctx.font = `${labelFontSize}px sans-serif`;
  ctx.fillStyle = config.textColor;
  ctx.globalAlpha = 0.6;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(config.targetLabel, w / 2, h * 0.18);
  ctx.globalAlpha = 1.0;

  // カウントダウン値
  let mainText = "";
  if (config.showDays && days > 0) {
    mainText = `${days}d`;
    if (config.showHours) {
      mainText += ` ${hours}h`;
    }
  } else if (config.showHours) {
    const totalHours = days * 24 + hours;
    mainText = `${totalHours}:${minutes.toString().padStart(2, "0")}`;
  } else {
    mainText = `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  const mainFontSize = Math.max(9, w * 0.22);
  ctx.font = `bold ${mainFontSize}px 'Consolas', monospace`;
  ctx.fillStyle = isExpired ? "#f38ba8" : config.textColor;
  ctx.fillText(mainText, w / 2, h * 0.48);

  // 秒の小さい表示
  if (config.showDays || config.showHours) {
    const subFontSize = Math.max(7, w * 0.14);
    ctx.font = `${subFontSize}px 'Consolas', monospace`;
    ctx.globalAlpha = 0.7;
    if (config.showDays && days > 0 && config.showHours) {
      ctx.fillText(`${minutes}m ${seconds}s`, w / 2, h * 0.72);
    } else {
      ctx.fillText(`:${seconds.toString().padStart(2, "0")}`, w / 2, h * 0.72);
    }
    ctx.globalAlpha = 1.0;
  }

  // 期限切れ表示
  if (isExpired) {
    const expFontSize = Math.max(6, w * 0.1);
    ctx.font = `${expFontSize}px sans-serif`;
    ctx.fillStyle = "#f38ba8";
    ctx.globalAlpha = 0.7;
    ctx.fillText("期限超過", w / 2, h * 0.88);
    ctx.globalAlpha = 1.0;
  }

  ctx.restore();
}
