// Weather Icon Animation Widget
var _wa = _wa || { t: 0, drops: [], flakes: [] };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  _wa.t++;
  var type = config.weatherType || 'sunny';
  var cx = w / 2, cy = h / 2;
  if (type === 'sunny') {
    // Sun
    var sr = Math.min(w, h) * 0.2;
    ctx.fillStyle = '#f9e2af';
    ctx.beginPath();
    ctx.arc(cx, cy, sr, 0, Math.PI * 2);
    ctx.fill();
    // Rays
    for (var i = 0; i < 8; i++) {
      var a = (i / 8) * Math.PI * 2 + _wa.t * 0.02;
      ctx.strokeStyle = '#f9e2af';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(cx + Math.cos(a) * sr * 1.3, cy + Math.sin(a) * sr * 1.3);
      ctx.lineTo(cx + Math.cos(a) * sr * 1.8, cy + Math.sin(a) * sr * 1.8);
      ctx.stroke();
    }
  } else if (type === 'cloudy') {
    var off = Math.sin(_wa.t * 0.03) * 5;
    drawCloud(ctx, cx + off, cy, Math.min(w, h) * 0.3, '#9399b2');
    drawCloud(ctx, cx - w * 0.15 + off * 0.5, cy + h * 0.08, Math.min(w, h) * 0.22, '#7f849c');
  } else if (type === 'rainy') {
    drawCloud(ctx, cx, cy - h * 0.15, Math.min(w, h) * 0.25, '#7f849c');
    if (_wa.drops.length < 12) {
      for (var d = 0; d < 12; d++) _wa.drops.push({ x: Math.random() * w, y: cy, speed: 2 + Math.random() * 3 });
    }
    ctx.strokeStyle = '#89b4fa';
    ctx.lineWidth = 1.5;
    _wa.drops.forEach(function(d) {
      ctx.beginPath(); ctx.moveTo(d.x, d.y); ctx.lineTo(d.x - 1, d.y + 6); ctx.stroke();
      d.y += d.speed;
      if (d.y > h) { d.y = cy; d.x = Math.random() * w; }
    });
  } else if (type === 'snowy') {
    drawCloud(ctx, cx, cy - h * 0.15, Math.min(w, h) * 0.25, '#9399b2');
    if (_wa.flakes.length < 15) {
      for (var f = 0; f < 15; f++) _wa.flakes.push({ x: Math.random() * w, y: cy, speed: 0.5 + Math.random() * 1.5, drift: Math.random() * 2 - 1 });
    }
    ctx.fillStyle = '#ffffff';
    _wa.flakes.forEach(function(f) {
      ctx.beginPath(); ctx.arc(f.x, f.y, 2, 0, Math.PI * 2); ctx.fill();
      f.y += f.speed; f.x += f.drift * 0.3;
      if (f.y > h) { f.y = cy; f.x = Math.random() * w; }
    });
  } else if (type === 'stormy') {
    drawCloud(ctx, cx, cy - h * 0.1, Math.min(w, h) * 0.28, '#585b70');
    if (_wa.t % 40 < 3) {
      ctx.strokeStyle = '#f9e2af';
      ctx.lineWidth = 2;
      ctx.beginPath();
      var lx = cx + (Math.random() - 0.5) * w * 0.3;
      ctx.moveTo(lx, cy); ctx.lineTo(lx - 5, cy + h * 0.15); ctx.lineTo(lx + 5, cy + h * 0.2); ctx.lineTo(lx - 2, cy + h * 0.35);
      ctx.stroke();
    }
  }
}
function drawCloud(ctx, x, y, size, color) {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(x, y, size * 0.5, 0, Math.PI * 2);
  ctx.arc(x - size * 0.4, y + size * 0.15, size * 0.35, 0, Math.PI * 2);
  ctx.arc(x + size * 0.4, y + size * 0.15, size * 0.35, 0, Math.PI * 2);
  ctx.fill();
}
