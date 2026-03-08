// Daylight Info Widget - sunrise/sunset calculation
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var lat = config.latitude || 35.68;
  var lng = config.longitude || 139.77;
  var textColor = config.textColor || '#f9e2af';
  var now = data.now;
  // Simplified sunrise/sunset calculation
  var dayOfYear = Math.floor((now - new Date(now.getFullYear(), 0, 0)) / 86400000);
  var decl = 23.45 * Math.sin((360 / 365) * (dayOfYear - 81) * Math.PI / 180);
  var latRad = lat * Math.PI / 180;
  var declRad = decl * Math.PI / 180;
  var cosH = -Math.tan(latRad) * Math.tan(declRad);
  cosH = Math.max(-1, Math.min(1, cosH));
  var hourAngle = Math.acos(cosH) * 180 / Math.PI;
  var solarNoon = 12 - lng / 15 + now.getTimezoneOffset() / 60;
  var sunriseH = solarNoon - hourAngle / 15;
  var sunsetH = solarNoon + hourAngle / 15;
  var daylightH = (sunsetH - sunriseH);
  function formatTime(h) {
    var hh = Math.floor(h);
    var mm = Math.round((h - hh) * 60);
    if (mm === 60) { hh++; mm = 0; }
    return (hh < 10 ? '0' : '') + hh + ':' + (mm < 10 ? '0' : '') + mm;
  }
  ctx.fillStyle = textColor;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.font = Math.max(7, w * 0.1) + 'px sans-serif';
  ctx.fillText('🌅 ' + formatTime(sunriseH), w / 2, h * 0.25);
  ctx.fillText('🌇 ' + formatTime(sunsetH), w / 2, h * 0.5);
  ctx.font = Math.max(6, w * 0.08) + 'px sans-serif';
  var dlH = Math.floor(daylightH);
  var dlM = Math.round((daylightH - dlH) * 60);
  ctx.fillText('☀️ ' + dlH + '時間' + dlM + '分', w / 2, h * 0.78);
}
