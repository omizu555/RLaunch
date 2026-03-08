// Pulse Animation Widget
var _pulse = _pulse || { t: 0 };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  var c1 = config.color1 || '#89b4fa';
  var c2 = config.color2 || '#f38ba8';
  var speed = config.speed || 2;
  _pulse.t += 0.02 * speed;
  var cx = w / 2, cy = h / 2;
  var maxR = Math.min(w, h) * 0.45;
  // Multiple rings
  for (var i = 0; i < 4; i++) {
    var phase = _pulse.t + i * 0.8;
    var scale = (Math.sin(phase) + 1) / 2; // 0-1
    var r = maxR * (0.3 + scale * 0.7);
    var alpha = 1 - scale * 0.8;
    ctx.beginPath();
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
    ctx.strokeStyle = i % 2 === 0 ? c1 : c2;
    ctx.globalAlpha = alpha;
    ctx.lineWidth = Math.max(1, maxR * 0.06);
    ctx.stroke();
  }
  ctx.globalAlpha = 1;
  // Center dot
  var dotR = maxR * 0.1 * (0.8 + Math.sin(_pulse.t * 2) * 0.2);
  ctx.beginPath();
  ctx.arc(cx, cy, dotR, 0, Math.PI * 2);
  ctx.fillStyle = c1;
  ctx.fill();
}
