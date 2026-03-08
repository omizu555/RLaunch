// Pomodoro Timer Widget - click to start/stop, with optional sound notification
var _pom = _pom || {
  state: 'idle', startTime: 0, isWork: true, total: 0, soundPlayed: false,
  audioElement: null, audioLoading: false, lastSoundFile: '',
  isPlaying: false, playTimer: null
};

/** Web Audio API で通知音を鳴らす (外部ファイル不要) */
function _pomPlaySound(volume) {
  try {
    var AudioCtx = window.AudioContext || window.webkitAudioContext;
    if (!AudioCtx) return;
    var ac = new AudioCtx();
    var gain = ac.createGain();
    gain.connect(ac.destination);
    var vol = Math.max(0, Math.min(1, (volume || 50) / 100));
    // 3 連ビープ: work 終了は高め、break 終了は低め
    var freqs = _pom.isWork ? [880, 988, 1047] : [523, 587, 659];
    freqs.forEach(function (freq, i) {
      var osc = ac.createOscillator();
      var env = ac.createGain();
      osc.type = 'sine';
      osc.frequency.value = freq;
      env.gain.setValueAtTime(vol * 0.6, ac.currentTime + i * 0.18);
      env.gain.exponentialRampToValueAtTime(0.001, ac.currentTime + i * 0.18 + 0.16);
      osc.connect(env);
      env.connect(gain);
      osc.start(ac.currentTime + i * 0.18);
      osc.stop(ac.currentTime + i * 0.18 + 0.18);
    });
    // AudioContext を自動クリーンアップ
    setTimeout(function () { ac.close(); }, 1500);
  } catch (e) { /* 音声再生失敗は無視 */ }
}

/** カスタム音声ファイルを Rust 経由で読み込みキャッシュ */
function _pomLoadAudioFile(filePath, invokeFn) {
  if (!filePath || !invokeFn || _pom.audioLoading) return;
  // パスが変わったら再ロード
  if (_pom.lastSoundFile === filePath && _pom.audioElement) return;
  _pom.audioLoading = true;
  _pom.lastSoundFile = filePath;
  _pom.audioElement = null;
  try {
    invokeFn('read_sound_file', { path: filePath })
      .then(function (dataUrl) {
        _pom.audioElement = new Audio(dataUrl);
        _pom.audioElement.preload = 'auto';
        _pom.audioLoading = false;
      })
      .catch(function (err) {
        console.warn('Sound file load failed:', err);
        _pom.audioLoading = false;
        _pom.audioElement = null;
      });
  } catch (e) {
    _pom.audioLoading = false;
  }
}

/** 通知音の再生（ビープ or カスタム音声） */
function _pomNotify(config) {
  var soundType = config.soundType || 'beep';
  var volume = config.soundVolume !== undefined ? config.soundVolume : 50;
  var maxSec = config.soundMaxSeconds !== undefined ? config.soundMaxSeconds : 10;
  if (soundType === 'custom' && _pom.audioElement) {
    try {
      _pom.audioElement.currentTime = 0;
      _pom.audioElement.volume = Math.max(0, Math.min(1, volume / 100));
      _pom.audioElement.play();
      _pom.isPlaying = true;
      // 最大再生秒数で自動停止
      if (_pom.playTimer) clearTimeout(_pom.playTimer);
      if (maxSec > 0) {
        _pom.playTimer = setTimeout(function () { _pomStopSound(); }, maxSec * 1000);
      }
    } catch (e) {
      _pomPlaySound(volume);
    }
  } else {
    _pomPlaySound(volume);
  }
}

/** 音楽を停止 */
function _pomStopSound() {
  if (_pom.playTimer) { clearTimeout(_pom.playTimer); _pom.playTimer = null; }
  if (_pom.audioElement && _pom.isPlaying) {
    try {
      _pom.audioElement.pause();
      _pom.audioElement.currentTime = 0;
    } catch (e) { /* ignore */ }
  }
  _pom.isPlaying = false;
}

function draw(ctx, w, h, config, data) {
  ctx.clearRect(0, 0, w, h);

  // カスタム音声ファイルの事前読み込み (data.invoke 経由)
  var invokeFn = data && data.invoke;
  var soundType = config.soundType || 'beep';
  if (soundType === 'custom' && config.soundFile && invokeFn) {
    _pomLoadAudioFile(config.soundFile, invokeFn);
  }

  // Click handling: toggle idle/running、音楽再生中なら停止
  if (data && data.clicked) {
    // 音楽が再生中ならまず停止
    if (_pom.isPlaying) {
      _pomStopSound();
    } else if (_pom.state === 'idle') {
      _pom.state = 'running';
      _pom.startTime = Date.now();
      _pom.isWork = true;
      _pom.soundPlayed = false;
    } else {
      _pom.state = 'idle';
      _pom.startTime = 0;
    }
  }
  var workMin = config.workMinutes || 25;
  var breakMin = config.breakMinutes || 5;
  var workColor = config.workColor || '#f38ba8';
  var breakColor = config.breakColor || '#a6e3a1';
  var textColor = config.textColor || '#ffffff';
  var soundEnabled = config.soundEnabled !== undefined ? config.soundEnabled : true;
  var cx = w / 2, cy = h / 2;
  var r = Math.min(w, h) * 0.32;
  var now = Date.now();
  var remaining, total;
  if (_pom.state === 'running') {
    total = (_pom.isWork ? workMin : breakMin) * 60;
    var elapsed = Math.floor((now - _pom.startTime) / 1000);
    remaining = Math.max(0, total - elapsed);
    if (remaining === 0) {
      // サウンド通知（1回だけ鳴らす）
      if (soundEnabled && !_pom.soundPlayed) {
        _pomNotify(config);
        _pom.soundPlayed = true;
      }
      _pom.isWork = !_pom.isWork;
      _pom.startTime = now;
      _pom.soundPlayed = false; // 次の区間用にリセット
    }
  } else {
    total = workMin * 60;
    remaining = total;
  }
  var progress = 1 - remaining / total;
  var activeColor = _pom.isWork ? workColor : breakColor;
  // Background circle
  ctx.beginPath();
  ctx.arc(cx, cy - h * 0.05, r, 0, Math.PI * 2);
  ctx.strokeStyle = '#313244';
  ctx.lineWidth = Math.max(3, r * 0.15);
  ctx.stroke();
  // Progress arc
  ctx.beginPath();
  ctx.arc(cx, cy - h * 0.05, r, -Math.PI / 2, -Math.PI / 2 + progress * Math.PI * 2);
  ctx.strokeStyle = activeColor;
  ctx.lineWidth = Math.max(3, r * 0.15);
  ctx.stroke();
  // Time text
  var m = Math.floor(remaining / 60);
  var s = remaining % 60;
  ctx.fillStyle = textColor;
  ctx.font = 'bold ' + Math.max(8, r * 0.6) + 'px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText((m < 10 ? '0' : '') + m + ':' + (s < 10 ? '0' : '') + s, cx, cy - h * 0.05);
  // Label
  ctx.font = Math.max(6, w * 0.08) + 'px sans-serif';
  var label;
  if (_pom.isPlaying) {
    label = '🔇 STOP';
  } else if (_pom.state === 'idle') {
    label = '🍅 START';
  } else {
    label = _pom.isWork ? '🍅 WORK' : '☕ BREAK';
  }
  ctx.fillText(label, cx, h * 0.88);
}
