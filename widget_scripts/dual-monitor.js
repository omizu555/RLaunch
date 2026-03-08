// Dual Monitor Widget - CPU + Memory
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var cpuColor = config.cpuColor || '#a6e3a1';
  var memColor = config.memColor || '#89b4fa';
  var textColor = config.textColor || '#ffffff';
  var cpu = data.systemInfo ? data.systemInfo.cpu_usage : 0;
  var mem = data.systemInfo ? data.systemInfo.memory_usage : 0;
  var r = Math.min(w / 4, h / 2) * 0.65;
  var lw = Math.max(3, r * 0.2);
  // CPU gauge (left)
  var cx1 = w * 0.3, cy = h * 0.45;
  drawGauge(ctx, cx1, cy, r, lw, cpu / 100, cpuColor);
  ctx.fillStyle = textColor;
  ctx.font = 'bold ' + Math.max(7, r * 0.55) + 'px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(Math.round(cpu) + '%', cx1, cy);
  ctx.font = Math.max(6, r * 0.4) + 'px sans-serif';
  ctx.fillText('CPU', cx1, cy + r + lw + 4);
  // Memory gauge (right)
  var cx2 = w * 0.7;
  drawGauge(ctx, cx2, cy, r, lw, mem / 100, memColor);
  ctx.fillStyle = textColor;
  ctx.font = 'bold ' + Math.max(7, r * 0.55) + 'px monospace';
  ctx.fillText(Math.round(mem) + '%', cx2, cy);
  ctx.font = Math.max(6, r * 0.4) + 'px sans-serif';
  ctx.fillText('MEM', cx2, cy + r + lw + 4);
}
function drawGauge(ctx, cx, cy, r, lw, pct, color) {
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.strokeStyle = '#313244';
  ctx.lineWidth = lw;
  ctx.stroke();
  ctx.beginPath();
  ctx.arc(cx, cy, r, -Math.PI / 2, -Math.PI / 2 + pct * Math.PI * 2);
  ctx.strokeStyle = color;
  ctx.lineWidth = lw;
  ctx.lineCap = 'round';
  ctx.stroke();
  ctx.lineCap = 'butt';
}
