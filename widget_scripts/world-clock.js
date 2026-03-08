// World Clock Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var tz = config.timezone || 'America/New_York';
  var labels = {
    'America/New_York': 'New York',
    'Europe/London': 'London',
    'Europe/Paris': 'Paris',
    'Asia/Shanghai': 'Shanghai',
    'Asia/Tokyo': 'Tokyo',
    'Australia/Sydney': 'Sydney'
  };
  var label = labels[tz] || tz;
  var timeStr;
  try {
    timeStr = data.now.toLocaleTimeString('ja-JP', { timeZone: tz, hour: '2-digit', minute: '2-digit', second: '2-digit' });
  } catch (e) {
    timeStr = data.now.toLocaleTimeString();
  }
  ctx.fillStyle = config.textColor || '#cdd6f4';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = 'bold ' + Math.max(8, w * 0.12) + 'px sans-serif';
  ctx.fillText(label, w / 2, h * 0.32);
  ctx.font = 'bold ' + Math.max(10, w * 0.2) + 'px monospace';
  ctx.fillText(timeStr, w / 2, h * 0.62);
}
