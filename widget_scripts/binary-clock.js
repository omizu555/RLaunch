// Binary Clock Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var now = data.now;
  var hours = now.getHours();
  var mins = now.getMinutes();
  var secs = now.getSeconds();
  var cols = [
    Math.floor(hours / 10), hours % 10,
    Math.floor(mins / 10), mins % 10,
    Math.floor(secs / 10), secs % 10
  ];
  var rows = 4;
  var dotR = Math.min(w / (cols.length * 3), h / (rows * 3)) * 0.9;
  var gapX = w / (cols.length + 1);
  var gapY = h / (rows + 1);
  for (var c = 0; c < cols.length; c++) {
    for (var r = 0; r < rows; r++) {
      var bit = (cols[c] >> (3 - r)) & 1;
      ctx.beginPath();
      ctx.arc(gapX * (c + 1), gapY * (r + 1), dotR, 0, Math.PI * 2);
      ctx.fillStyle = bit ? (config.dotColor || '#89b4fa') : (config.offColor || '#313244');
      ctx.fill();
    }
  }
}
