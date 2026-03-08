// Stopwatch Widget - uses global state, click to start/stop
var _sw = _sw || { running: false, elapsed: 0, lastTick: 0 };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  // Click handling: toggle start/stop
  if (data && data.clicked) {
    if (_sw.running) {
      _sw.running = false;
      _sw.lastTick = 0;
    } else {
      _sw.running = true;
      _sw.lastTick = Date.now();
    }
  }
  var now = Date.now();
  if (_sw.running) {
    if (_sw.lastTick) _sw.elapsed += now - _sw.lastTick;
    _sw.lastTick = now;
  }
  var total = Math.floor(_sw.elapsed / 1000);
  var ms = Math.floor((_sw.elapsed % 1000) / 10);
  var s = total % 60;
  var m = Math.floor(total / 60) % 60;
  var h2 = Math.floor(total / 3600);
  var text = (h2 > 0 ? h2 + ':' : '') +
    (m < 10 ? '0' : '') + m + ':' +
    (s < 10 ? '0' : '') + s + '.' +
    (ms < 10 ? '0' : '') + ms;
  ctx.fillStyle = config.textColor || '#a6e3a1';
  ctx.font = 'bold ' + Math.max(8, w * 0.18) + 'px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, w / 2, h / 2 - h * 0.08);
  // Status indicator
  ctx.font = Math.max(6, w * 0.08) + 'px sans-serif';
  ctx.fillStyle = _sw.running ? '#a6e3a1' : '#6c7086';
  ctx.fillText(_sw.running ? '● RUN' : '■ STOP', w / 2, h * 0.78);
  // Hint
  ctx.font = Math.max(5, w * 0.06) + 'px sans-serif';
  ctx.fillStyle = '#585b70';
  ctx.fillText('click: start/stop', w / 2, h * 0.92);
}
