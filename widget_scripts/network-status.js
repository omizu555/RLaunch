// Network Status Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var online = typeof navigator !== 'undefined' ? navigator.onLine : true;
  var color = online ? (config.onlineColor || '#a6e3a1') : (config.offlineColor || '#f38ba8');
  var cx = w / 2, cy = h * 0.4;
  // Signal bars
  var barW = w * 0.06;
  var gap = barW * 0.8;
  var bars = 4;
  var totalW = bars * barW + (bars - 1) * gap;
  var startX = cx - totalW / 2;
  for (var i = 0; i < bars; i++) {
    var barH = h * 0.08 * (i + 1);
    var x = startX + i * (barW + gap);
    var y = cy + h * 0.15 - barH;
    ctx.fillStyle = online ? color : '#45475a';
    ctx.beginPath();
    ctx.roundRect(x, y, barW, barH, 2);
    ctx.fill();
  }
  // Status text
  ctx.fillStyle = color;
  ctx.font = 'bold ' + Math.max(7, w * 0.1) + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(online ? 'ONLINE' : 'OFFLINE', cx, h * 0.78);
  // Dot indicator
  ctx.beginPath();
  ctx.arc(cx - w * 0.18, h * 0.78, 3, 0, Math.PI * 2);
  ctx.fillStyle = color;
  ctx.fill();
}
