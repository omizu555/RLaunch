// FPS Counter Widget
var _fps = _fps || { frames: 0, lastTime: Date.now(), current: 0 };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  _fps.frames++;
  var now = Date.now();
  var delta = now - _fps.lastTime;
  if (delta >= 1000) {
    _fps.current = Math.round(_fps.frames * 1000 / delta);
    _fps.frames = 0;
    _fps.lastTime = now;
  }
  var textColor = config.textColor || '#a6e3a1';
  var cx = w / 2, cy = h / 2;
  ctx.fillStyle = textColor;
  ctx.font = 'bold ' + Math.max(12, Math.min(w, h) * 0.35) + 'px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(_fps.current, cx, cy - h * 0.05);
  ctx.font = Math.max(7, w * 0.1) + 'px sans-serif';
  ctx.fillStyle = '#6c7086';
  ctx.fillText('FPS', cx, h * 0.8);
}
