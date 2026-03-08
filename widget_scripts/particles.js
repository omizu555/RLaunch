// Particles Widget
var _pt = _pt || { particles: [], initialized: false };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  var bg = config.backgroundColor || '#1e1e2e';
  if (bg !== 'transparent') {
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);
  }
  var color = config.particleColor || '#89b4fa';
  var count = config.count || 30;
  if (!_pt.initialized || _pt.particles.length !== count) {
    _pt.particles = [];
    for (var i = 0; i < count; i++) {
      _pt.particles.push({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * 1.2,
        vy: (Math.random() - 0.5) * 1.2,
        r: 1 + Math.random() * 2.5
      });
    }
    _pt.initialized = true;
  }
  // Update and draw
  _pt.particles.forEach(function(p) {
    p.x += p.vx;
    p.y += p.vy;
    if (p.x < 0 || p.x > w) p.vx *= -1;
    if (p.y < 0 || p.y > h) p.vy *= -1;
    p.x = Math.max(0, Math.min(w, p.x));
    p.y = Math.max(0, Math.min(h, p.y));
  });
  // Draw connections
  ctx.strokeStyle = color;
  var maxDist = Math.min(w, h) * 0.3;
  for (var i = 0; i < _pt.particles.length; i++) {
    for (var j = i + 1; j < _pt.particles.length; j++) {
      var a = _pt.particles[i], b = _pt.particles[j];
      var dx = a.x - b.x, dy = a.y - b.y;
      var dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < maxDist) {
        ctx.globalAlpha = 1 - dist / maxDist;
        ctx.lineWidth = 0.5;
        ctx.beginPath();
        ctx.moveTo(a.x, a.y);
        ctx.lineTo(b.x, b.y);
        ctx.stroke();
      }
    }
  }
  ctx.globalAlpha = 1;
  // Draw particles
  ctx.fillStyle = color;
  _pt.particles.forEach(function(p) {
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
    ctx.fill();
  });
}
