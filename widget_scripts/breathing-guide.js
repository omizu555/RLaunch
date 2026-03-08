// Breathing Guide Widget - click to reset cycle
var _bg = _bg || { startTime: Date.now() };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  // Click handling: reset breathing cycle
  if (data && data.clicked) {
    _bg.startTime = Date.now();
  }
  var inhale = (config.inhaleSeconds || 4);
  var hold = (config.holdSeconds || 4);
  var exhale = (config.exhaleSeconds || 6);
  var total = inhale + hold + exhale;
  var circleColor = config.circleColor || '#89b4fa';
  var textColor = config.textColor || '#cdd6f4';
  var elapsed = ((Date.now() - _bg.startTime) / 1000) % total;
  var phase, progress, label;
  if (elapsed < inhale) {
    phase = 'inhale';
    progress = elapsed / inhale;
    label = '吸う';
  } else if (elapsed < inhale + hold) {
    phase = 'hold';
    progress = 1;
    label = '止める';
  } else {
    phase = 'exhale';
    progress = 1 - (elapsed - inhale - hold) / exhale;
    label = '吐く';
  }
  var cx = w / 2, cy = h * 0.45;
  var minR = Math.min(w, h) * 0.12;
  var maxR = Math.min(w, h) * 0.35;
  var r = minR + (maxR - minR) * progress;
  // Outer glow
  ctx.beginPath();
  ctx.arc(cx, cy, r + 4, 0, Math.PI * 2);
  ctx.fillStyle = circleColor;
  ctx.globalAlpha = 0.15;
  ctx.fill();
  ctx.globalAlpha = 1;
  // Main circle
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = circleColor;
  ctx.globalAlpha = 0.4 + progress * 0.4;
  ctx.fill();
  ctx.globalAlpha = 1;
  // Label
  ctx.fillStyle = textColor;
  ctx.font = 'bold ' + Math.max(8, w * 0.12) + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(label, cx, cy);
  // Timer
  ctx.font = Math.max(6, w * 0.08) + 'px sans-serif';
  var remaining;
  if (phase === 'inhale') remaining = Math.ceil(inhale - elapsed);
  else if (phase === 'hold') remaining = Math.ceil(inhale + hold - elapsed);
  else remaining = Math.ceil(total - elapsed);
  ctx.fillText(remaining + 's', cx, h * 0.85);
}
