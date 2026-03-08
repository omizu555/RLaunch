// Dice Widget - click to roll
var _dice = _dice || { value: Math.ceil(Math.random() * 6) };
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  // Click handling: re-roll
  if (data && data.clicked) {
    _dice.value = Math.ceil(Math.random() * 6);
  }
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var diceColor = config.diceColor || '#ffffff';
  var dotColor = config.dotColor || '#1e1e2e';
  var val = _dice.value;
  var size = Math.min(w, h) * 0.6;
  var cx = w / 2, cy = h / 2;
  var x = cx - size / 2, y = cy - size / 2;
  // Dice body
  ctx.fillStyle = diceColor;
  ctx.beginPath();
  ctx.roundRect(x, y, size, size, size * 0.12);
  ctx.fill();
  // Dot positions (relative to dice size)
  var dr = size * 0.08;
  var p = size * 0.25;
  var dots = {
    1: [[0.5, 0.5]],
    2: [[0.25, 0.25], [0.75, 0.75]],
    3: [[0.25, 0.25], [0.5, 0.5], [0.75, 0.75]],
    4: [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75], [0.75, 0.75]],
    5: [[0.25, 0.25], [0.75, 0.25], [0.5, 0.5], [0.25, 0.75], [0.75, 0.75]],
    6: [[0.25, 0.25], [0.75, 0.25], [0.25, 0.5], [0.75, 0.5], [0.25, 0.75], [0.75, 0.75]],
  };
  ctx.fillStyle = dotColor;
  (dots[val] || dots[1]).forEach(function(pos) {
    ctx.beginPath();
    ctx.arc(x + pos[0] * size, y + pos[1] * size, dr, 0, Math.PI * 2);
    ctx.fill();
  });
}
