/* ============================================================
   アナログ時計 Canvas 描画
   ============================================================ */
import type { AnalogClockConfig } from "../types";

export function drawAnalogClock(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  config: AnalogClockConfig
) {
  const size = Math.min(w, h);
  const cx = w / 2;
  const cy = h / 2;
  const r = size / 2 - 2;

  ctx.clearRect(0, 0, w, h);

  // 背景
  if (config.backgroundColor && config.backgroundColor !== "transparent") {
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.fillStyle = config.backgroundColor;
    ctx.fill();
  }

  // 文字盤の外枠
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.strokeStyle = "rgba(255,255,255,0.15)";
  ctx.lineWidth = 1;
  ctx.stroke();

  // 時刻を取得
  const now = new Date();

  // 文字盤
  if (config.dialStyle !== "none") {
    for (let i = 0; i < 12; i++) {
      const angle = (i * Math.PI) / 6 - Math.PI / 2;
      const markR = r * 0.85;

      if (config.dialStyle === "dots") {
        const dotR = i % 3 === 0 ? 2.5 : 1.5;
        ctx.beginPath();
        ctx.arc(cx + Math.cos(angle) * markR, cy + Math.sin(angle) * markR, dotR, 0, Math.PI * 2);
        ctx.fillStyle = "rgba(255,255,255,0.5)";
        ctx.fill();
      } else if (config.dialStyle === "roman") {
        const romans = ["XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
        const textR = r * 0.72;
        ctx.save();
        ctx.font = `bold ${Math.max(7, size * 0.09)}px sans-serif`;
        ctx.fillStyle = "rgba(255,255,255,0.7)";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(romans[i], cx + Math.cos(angle) * textR, cy + Math.sin(angle) * textR);
        ctx.restore();
      } else {
        // simple: 短い線
        const inner = i % 3 === 0 ? r * 0.73 : r * 0.8;
        const outer = r * 0.88;
        ctx.beginPath();
        ctx.moveTo(cx + Math.cos(angle) * inner, cy + Math.sin(angle) * inner);
        ctx.lineTo(cx + Math.cos(angle) * outer, cy + Math.sin(angle) * outer);
        ctx.strokeStyle = i % 3 === 0 ? "rgba(255,255,255,0.6)" : "rgba(255,255,255,0.3)";
        ctx.lineWidth = i % 3 === 0 ? 1.5 : 0.8;
        ctx.stroke();
      }
    }
  }

  const hours = now.getHours() % 12;
  const minutes = now.getMinutes();
  const seconds = now.getSeconds();

  // 時針
  const hAngle = ((hours + minutes / 60) * Math.PI) / 6 - Math.PI / 2;
  drawHand(ctx, cx, cy, hAngle, r * 0.5, 2.5, config.hourHandColor);

  // 分針
  const mAngle = ((minutes + seconds / 60) * Math.PI) / 30 - Math.PI / 2;
  drawHand(ctx, cx, cy, mAngle, r * 0.7, 1.8, config.minuteHandColor);

  // 秒針
  if (config.showSecondHand) {
    const sAngle = (seconds * Math.PI) / 30 - Math.PI / 2;
    drawHand(ctx, cx, cy, sAngle, r * 0.75, 0.8, config.secondHandColor);
  }

  // 中心点
  ctx.beginPath();
  ctx.arc(cx, cy, 2.5, 0, Math.PI * 2);
  ctx.fillStyle = config.secondHandColor || "#fff";
  ctx.fill();
}

function drawHand(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  angle: number,
  length: number,
  width: number,
  color: string
) {
  ctx.beginPath();
  ctx.moveTo(cx, cy);
  ctx.lineTo(cx + Math.cos(angle) * length, cy + Math.sin(angle) * length);
  ctx.strokeStyle = color;
  ctx.lineWidth = width;
  ctx.lineCap = "round";
  ctx.stroke();
}
