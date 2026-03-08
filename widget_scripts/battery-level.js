// Battery Level Widget (simulated - uses time-based value since no real API)
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  // Simulated battery level based on seconds (for demo)
  var level = ((data.now.getMinutes() * 60 + data.now.getSeconds()) % 101);
  var fullColor = config.fullColor || '#a6e3a1';
  var lowColor = config.lowColor || '#f38ba8';
  var cx = w / 2, cy = h / 2;
  var bw = w * 0.5, bh = h * 0.3;
  var bx = cx - bw / 2, by = cy - bh / 2;
  // Battery outline
  ctx.strokeStyle = '#6c7086';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.roundRect(bx, by, bw, bh, 4);
  ctx.stroke();
  // Terminal
  ctx.fillStyle = '#6c7086';
  ctx.fillRect(bx + bw, cy - bh * 0.2, w * 0.03, bh * 0.4);
  // Fill
  var fillW = (bw - 4) * (level / 100);
  var color = level <= 20 ? lowColor : fullColor;
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.roundRect(bx + 2, by + 2, fillW, bh - 4, 2);
  ctx.fill();
  // Percentage text
  ctx.fillStyle = '#cdd6f4';
  ctx.font = 'bold ' + Math.max(8, w * 0.12) + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(level + '%', cx, cy + bh * 0.8);
}
