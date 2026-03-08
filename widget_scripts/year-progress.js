// Year Progress Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var now = data.now;
  var start = new Date(now.getFullYear(), 0, 1);
  var end = new Date(now.getFullYear() + 1, 0, 1);
  var progress = (now.getTime() - start.getTime()) / (end.getTime() - start.getTime());
  var pct = Math.floor(progress * 1000) / 10;
  var barColor = config.barColor || '#89b4fa';
  var textColor = config.textColor || '#cdd6f4';
  // Year label
  ctx.fillStyle = textColor;
  ctx.font = Math.max(7, w * 0.09) + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(now.getFullYear() + '年', w / 2, h * 0.22);
  // Progress bar background
  var barX = w * 0.1, barY = h * 0.42, barW = w * 0.8, barH = h * 0.16;
  ctx.fillStyle = '#313244';
  ctx.beginPath();
  ctx.roundRect(barX, barY, barW, barH, barH / 2);
  ctx.fill();
  // Progress bar fill
  ctx.fillStyle = barColor;
  ctx.beginPath();
  ctx.roundRect(barX, barY, barW * progress, barH, barH / 2);
  ctx.fill();
  // Percentage
  ctx.fillStyle = textColor;
  ctx.font = 'bold ' + Math.max(10, w * 0.15) + 'px monospace';
  ctx.fillText(pct.toFixed(1) + '%', w / 2, h * 0.76);
}
