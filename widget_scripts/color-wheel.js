// Color Wheel Widget
var _cw = _cw || { angle: 0 };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var speed = config.speed || 1;
  _cw.angle += speed * 0.5;
  var cx = w / 2, cy = h / 2;
  var outerR = Math.min(w, h) * 0.42;
  var innerR = outerR * 0.6;
  var segments = 60;
  for (var i = 0; i < segments; i++) {
    var a1 = (i / segments) * Math.PI * 2 + _cw.angle * Math.PI / 180;
    var a2 = ((i + 1) / segments) * Math.PI * 2 + _cw.angle * Math.PI / 180;
    var hue = (i / segments) * 360;
    ctx.beginPath();
    ctx.moveTo(cx + Math.cos(a1) * innerR, cy + Math.sin(a1) * innerR);
    ctx.lineTo(cx + Math.cos(a1) * outerR, cy + Math.sin(a1) * outerR);
    ctx.lineTo(cx + Math.cos(a2) * outerR, cy + Math.sin(a2) * outerR);
    ctx.lineTo(cx + Math.cos(a2) * innerR, cy + Math.sin(a2) * innerR);
    ctx.closePath();
    ctx.fillStyle = 'hsl(' + hue + ', 80%, 60%)';
    ctx.fill();
  }
}
