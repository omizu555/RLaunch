/* ============================================================
   システムモニター (CPU/メモリ) Canvas 描画
   ============================================================ */
import type { SystemMonitorConfig } from "../types";

export function drawSystemMonitor(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: SystemMonitorConfig,
  value: number // 0-100
) {
  ctx.clearRect(0, 0, w, h);

  // 背景
  if (config.backgroundColor && config.backgroundColor !== "transparent") {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }

  const isWarning = value >= config.warningThreshold;
  const color = isWarning ? config.warningColor : config.gaugeColor;

  ctx.save();

  if (config.displayStyle === "gauge") {
    drawGauge(ctx, w, h, config, value, color);
  } else if (config.displayStyle === "bar") {
    drawBar(ctx, w, h, config, value, color);
  } else {
    drawText(ctx, w, h, config, value, color);
  }

  ctx.restore();
}

function drawGauge(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: SystemMonitorConfig,
  value: number,
  color: string
) {
  const cx = w / 2;
  const cy = h * 0.52;
  const r = Math.min(w, h) * 0.35;
  const lineWidth = Math.max(3, r * 0.2);

  const startAngle = Math.PI * 0.75;
  const endAngle = Math.PI * 2.25;

  // 背景アーク
  ctx.beginPath();
  ctx.arc(cx, cy, r, startAngle, endAngle);
  ctx.strokeStyle = "rgba(255,255,255,0.1)";
  ctx.lineWidth = lineWidth;
  ctx.lineCap = "round";
  ctx.stroke();

  // 値アーク
  const valueAngle = startAngle + (endAngle - startAngle) * (value / 100);
  ctx.beginPath();
  ctx.arc(cx, cy, r, startAngle, valueAngle);
  ctx.strokeStyle = color;
  ctx.lineWidth = lineWidth;
  ctx.lineCap = "round";
  ctx.stroke();

  // 中央の数値
  const fontSize = Math.max(9, w * 0.22);
  ctx.font = `bold ${fontSize}px 'Consolas', monospace`;
  ctx.fillStyle = config.textColor;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(`${Math.round(value)}%`, cx, cy);

  // ラベル
  const labelSize = Math.max(6, w * 0.11);
  ctx.font = `${labelSize}px sans-serif`;
  ctx.globalAlpha = 0.6;
  ctx.fillText(config.target.toUpperCase(), cx, h * 0.88);
  ctx.globalAlpha = 1.0;
}

function drawBar(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: SystemMonitorConfig,
  value: number,
  color: string
) {
  const pad = w * 0.12;
  const barW = w - pad * 2;
  const barH = Math.max(6, h * 0.15);
  const barY = h * 0.55;

  // 背景バー
  ctx.fillStyle = "rgba(255,255,255,0.1)";
  roundRect(ctx, pad, barY, barW, barH, 3);

  // 値バー
  const valW = barW * (value / 100);
  ctx.fillStyle = color;
  if (valW > 0) {
    roundRect(ctx, pad, barY, valW, barH, 3);
  }

  // ラベル
  const labelSize = Math.max(7, w * 0.12);
  ctx.font = `${labelSize}px sans-serif`;
  ctx.fillStyle = config.textColor;
  ctx.globalAlpha = 0.6;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(config.target.toUpperCase(), w / 2, h * 0.25);
  ctx.globalAlpha = 1.0;

  // 数値
  const numSize = Math.max(10, w * 0.2);
  ctx.font = `bold ${numSize}px 'Consolas', monospace`;
  ctx.fillStyle = config.textColor;
  ctx.fillText(`${Math.round(value)}%`, w / 2, h * 0.4);
}

function drawText(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: SystemMonitorConfig,
  value: number,
  color: string
) {
  const fontSize = Math.max(12, w * 0.3);
  ctx.font = `bold ${fontSize}px 'Consolas', monospace`;
  ctx.fillStyle = color;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(`${Math.round(value)}%`, w / 2, h * 0.5);

  const labelSize = Math.max(7, w * 0.12);
  ctx.font = `${labelSize}px sans-serif`;
  ctx.fillStyle = config.textColor;
  ctx.globalAlpha = 0.6;
  ctx.fillText(config.target.toUpperCase(), w / 2, h * 0.82);
  ctx.globalAlpha = 1.0;
}

function roundRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number
) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + w - r, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + r);
  ctx.lineTo(x + w, y + h - r);
  ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
  ctx.lineTo(x + r, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
  ctx.fill();
}
