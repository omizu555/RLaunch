// Moon Phase Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  var bg = config.backgroundColor || '#1e1e2e';
  if (bg !== 'transparent') {
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);
  }
  // Calculate moon phase (simplified synodic month = 29.53 days)
  var now = data.now;
  var ref = new Date(2000, 0, 6, 18, 14, 0); // known new moon
  var diff = (now.getTime() - ref.getTime()) / 1000 / 86400;
  var cycle = 29.53058867;
  var phase = ((diff % cycle) + cycle) % cycle;
  var fraction = phase / cycle; // 0 = new, 0.5 = full
  var moonColor = config.moonColor || '#f9e2af';
  var cx = w / 2, cy = h * 0.42;
  var r = Math.min(w, h) * 0.3;
  // Draw full moon
  ctx.beginPath();
  ctx.arc(cx, cy, r, 0, Math.PI * 2);
  ctx.fillStyle = moonColor;
  ctx.fill();
  // Draw shadow
  ctx.beginPath();
  var sweep = Math.cos(fraction * 2 * Math.PI);
  ctx.moveTo(cx, cy - r);
  // Right semicircle
  ctx.arc(cx, cy, r, -Math.PI / 2, Math.PI / 2, fraction > 0.5);
  // Curved shadow edge
  ctx.ellipse(cx, cy, Math.abs(sweep) * r, r, 0, Math.PI / 2, -Math.PI / 2, sweep > 0 ? (fraction < 0.5) : (fraction >= 0.5));
  ctx.closePath();
  ctx.fillStyle = bg === 'transparent' ? '#1e1e2e' : bg;
  ctx.fill();
  // Phase names
  var names = ['🌑 新月', '🌒 三日月', '🌓 上弦', '🌔 十三夜', '🌕 満月', '🌖 十六夜', '🌗 下弦', '🌘 晦'];
  var idx = Math.round(fraction * 8) % 8;
  ctx.fillStyle = moonColor;
  ctx.font = Math.max(7, w * 0.1) + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(names[idx], w / 2, h * 0.82);
}
