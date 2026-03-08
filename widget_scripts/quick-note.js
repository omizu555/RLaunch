// Quick Note Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  var bg = config.backgroundColor || '#313244';
  if (bg !== 'transparent') {
    ctx.fillStyle = bg;
    ctx.beginPath();
    ctx.roundRect(2, 2, w - 4, h - 4, 6);
    ctx.fill();
  }
  var note = config.note || 'メモ';
  var textColor = config.textColor || '#cdd6f4';
  var fontSize = Math.max(7, Math.min(w * 0.1, h * 0.12));
  ctx.fillStyle = textColor;
  ctx.font = fontSize + 'px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  // Simple word wrap
  var lines = [];
  var maxW = w * 0.85;
  var words = note.split('');
  var line = '';
  for (var i = 0; i < words.length; i++) {
    var test = line + words[i];
    if (ctx.measureText(test).width > maxW && line.length > 0) {
      lines.push(line);
      line = words[i];
    } else {
      line = test;
    }
  }
  if (line) lines.push(line);
  var lineH = fontSize * 1.4;
  var startY = h / 2 - (lines.length - 1) * lineH / 2;
  lines.forEach(function(l, idx) {
    ctx.fillText(l, w / 2, startY + idx * lineH);
  });
}
