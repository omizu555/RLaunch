// Daily Quote Widget
function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);
  if (config.backgroundColor && config.backgroundColor !== 'transparent') {
    ctx.fillStyle = config.backgroundColor;
    ctx.fillRect(0, 0, w, h);
  }
  var quotes = [
    ['継続は力なり', ''],
    ['千里の道も一歩から', '老子'],
    ['為せば成る', '上杉鷹山'],
    ['初心忘るべからず', '世阿弥'],
    ['失敗は成功のもと', ''],
    ['七転び八起き', ''],
    ['塵も積もれば山となる', ''],
    ['急がば回れ', ''],
    ['笑う門には福来たる', ''],
    ['石の上にも三年', ''],
    ['一期一会', ''],
    ['雨降って地固まる', ''],
    ['案ずるより産むが易し', ''],
    ['思い立ったが吉日', ''],
    ['転ばぬ先の杖', ''],
    ['能ある鷹は爪を隠す', ''],
    ['習うより慣れろ', ''],
    ['良薬は口に苦し', ''],
    ['情けは人の為ならず', ''],
    ['明日は明日の風が吹く', ''],
    ['一日一生', '内村鑑三'],
    ['過去と他人は変えられない', ''],
    ['今日が人生で一番若い日', ''],
    ['小さなことを重ねることが\nとんでもないところに\n行くただ一つの道', 'イチロー'],
    ['夢は逃げない\n逃げるのはいつも自分', ''],
    ['やってみせ 言って聞かせて\nさせてみせ', '山本五十六'],
    ['心が変われば態度が変わる', ''],
    ['人生に失敗がないと\n人生を失敗する', '斎藤茂太'],
    ['どんな壁も扉である', ''],
    ['諦めたらそこで試合終了', '安西先生'],
    ['百聞は一見に如かず', ''],
  ];
  var dayOfYear = Math.floor((data.now - new Date(data.now.getFullYear(), 0, 0)) / 86400000);
  var idx = dayOfYear % quotes.length;
  var quote = quotes[idx][0];
  var author = quotes[idx][1];
  var textColor = config.textColor || '#cdd6f4';
  ctx.fillStyle = textColor;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  var lines = quote.split('\n');
  var fontSize = Math.max(7, Math.min(w * 0.09, h * 0.11));
  ctx.font = fontSize + 'px sans-serif';
  var totalH = lines.length * fontSize * 1.5;
  var startY = h * 0.45 - totalH / 2 + fontSize / 2;
  lines.forEach(function(line, i) {
    ctx.fillText(line, w / 2, startY + i * fontSize * 1.5);
  });
  if (author) {
    ctx.font = Math.max(6, fontSize * 0.7) + 'px sans-serif';
    ctx.fillStyle = '#6c7086';
    ctx.fillText('— ' + author, w / 2, h * 0.88);
  }
}
