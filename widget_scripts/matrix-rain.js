// Matrix Rain Widget
var _mx = _mx || { columns: [], initialized: false };
function draw(ctx, w, h, config, data) {
  var bg = config.backgroundColor || '#000000';
  // Fade effect instead of full clear
  ctx.fillStyle = bg;
  ctx.globalAlpha = 0.1;
  ctx.fillRect(0, 0, w, h);
  ctx.globalAlpha = 1;
  var charColor = config.charColor || '#00ff00';
  var fontSize = Math.max(6, Math.min(w, h) * 0.06);
  var cols = Math.floor(w / fontSize);
  if (!_mx.initialized || _mx.columns.length !== cols) {
    _mx.columns = [];
    for (var i = 0; i < cols; i++) {
      _mx.columns.push(Math.random() * -50);
    }
    _mx.initialized = true;
    // Full clear on init
    ctx.globalAlpha = 1;
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);
  }
  ctx.fillStyle = charColor;
  ctx.font = fontSize + 'px monospace';
  var chars = 'ｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ0123456789';
  for (var c = 0; c < cols; c++) {
    var ch = chars[Math.floor(Math.random() * chars.length)];
    var x = c * fontSize;
    var y = _mx.columns[c] * fontSize;
    ctx.fillText(ch, x, y);
    if (y > h && Math.random() > 0.975) {
      _mx.columns[c] = 0;
    }
    _mx.columns[c]++;
  }
}
