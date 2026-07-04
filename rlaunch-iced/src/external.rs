//! 外部イベント統合層。
//! コールバックベースの外部ソース（グローバルホットキー / トレイ / デスクトップフック /
//! 二重起動の表示要求）を単一の futures channel に集約し、iced の Subscription へ橋渡しする。
//!
//! 設計: main() で `init()` を呼んで channel を作り、送信側 (Sender) をグローバルに保持。
//! 各コールバックは `send()` で ExternalEvent を投げる。受信側 (Receiver) は
//! `subscription()` が一度だけ取り出してストリーム化する（daemon 生存中は購読が
//! 破棄されない前提。識別子は固定）。

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone)]
pub enum ExternalEvent {
    /// グローバルホットキー押下（HotKey::id）
    Hotkey(u32),
    /// トレイアイコン左クリック
    TrayToggle,
    /// トレイメニュー: 表示/非表示
    TrayMenuToggle,
    /// トレイメニュー: 設定
    TrayMenuSettings,
    /// トレイメニュー: 終了
    TrayMenuQuit,
    /// デスクトップダブルクリック
    DesktopDoubleClick,
    /// 二重起動プロセスからの表示要求
    ShowRequest,
}

static SENDER: OnceLock<UnboundedSender<ExternalEvent>> = OnceLock::new();
static RECEIVER: Mutex<Option<UnboundedReceiver<ExternalEvent>>> = Mutex::new(None);

/// main() の最初に一度だけ呼ぶ
pub fn init() {
    let (tx, rx) = unbounded();
    let _ = SENDER.set(tx);
    *RECEIVER.lock().unwrap() = Some(rx);
}

pub fn send(event: ExternalEvent) {
    if let Some(tx) = SENDER.get() {
        let _ = tx.unbounded_send(event);
    }
}

/// iced Subscription（外部イベント → Message 変換は呼び出し側で map）
pub fn subscription() -> iced::Subscription<ExternalEvent> {
    iced::Subscription::run(|| {
        iced::stream::channel(
            64,
            |mut output: futures::channel::mpsc::Sender<ExternalEvent>| async move {
                let rx = RECEIVER.lock().unwrap().take();
                if let Some(mut rx) = rx {
                    while let Some(ev) = rx.next().await {
                        use futures::SinkExt;
                        let _ = output.send(ev).await;
                    }
                }
                // channel が尽きたら何もしない（プロセス終了まで到達しない想定）
                futures::future::pending::<()>().await;
            },
        )
    })
}

/// GlobalHotKeyEvent のコールバックを channel へ配線（main() で一度だけ）
pub fn wire_hotkey_events() {
    GlobalHotKeyEvent::set_event_handler(Some(|ev: GlobalHotKeyEvent| {
        if ev.state == HotKeyState::Pressed {
            send(ExternalEvent::Hotkey(ev.id));
        }
    }));
}

// ------------------------------------------------------------------
// ホットキー文字列パーサ（旧版の "Ctrl+Space" / "Ctrl+Alt+A" 形式）
// ------------------------------------------------------------------

/// "Ctrl+Alt+A" のような文字列を HotKey に変換する。失敗時は Err に理由。
pub fn parse_hotkey(s: &str) -> Result<HotKey, String> {
    let mut mods = Modifiers::empty();
    let mut key: Option<Code> = None;
    for part in s.split('+') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "win" | "super" | "meta" | "cmd" => mods |= Modifiers::SUPER,
            _ => {
                if key.is_some() {
                    return Err(format!("キーが複数あります: {}", s));
                }
                key = Some(parse_key(part).ok_or_else(|| format!("不明なキー: {}", part))?);
            }
        }
    }
    let code = key.ok_or_else(|| format!("キーがありません: {}", s))?;
    let m = if mods.is_empty() { None } else { Some(mods) };
    Ok(HotKey::new(m, code))
}

fn parse_key(s: &str) -> Option<Code> {
    let u = s.to_ascii_uppercase();
    // A-Z / 0-9
    if u.len() == 1 {
        let c = u.chars().next().unwrap();
        return match c {
            'A' => Some(Code::KeyA),
            'B' => Some(Code::KeyB),
            'C' => Some(Code::KeyC),
            'D' => Some(Code::KeyD),
            'E' => Some(Code::KeyE),
            'F' => Some(Code::KeyF),
            'G' => Some(Code::KeyG),
            'H' => Some(Code::KeyH),
            'I' => Some(Code::KeyI),
            'J' => Some(Code::KeyJ),
            'K' => Some(Code::KeyK),
            'L' => Some(Code::KeyL),
            'M' => Some(Code::KeyM),
            'N' => Some(Code::KeyN),
            'O' => Some(Code::KeyO),
            'P' => Some(Code::KeyP),
            'Q' => Some(Code::KeyQ),
            'R' => Some(Code::KeyR),
            'S' => Some(Code::KeyS),
            'T' => Some(Code::KeyT),
            'U' => Some(Code::KeyU),
            'V' => Some(Code::KeyV),
            'W' => Some(Code::KeyW),
            'X' => Some(Code::KeyX),
            'Y' => Some(Code::KeyY),
            'Z' => Some(Code::KeyZ),
            '0' => Some(Code::Digit0),
            '1' => Some(Code::Digit1),
            '2' => Some(Code::Digit2),
            '3' => Some(Code::Digit3),
            '4' => Some(Code::Digit4),
            '5' => Some(Code::Digit5),
            '6' => Some(Code::Digit6),
            '7' => Some(Code::Digit7),
            '8' => Some(Code::Digit8),
            '9' => Some(Code::Digit9),
            _ => None,
        };
    }
    // F1-F24
    if let Some(n) = u.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()) {
        return match n {
            1 => Some(Code::F1),
            2 => Some(Code::F2),
            3 => Some(Code::F3),
            4 => Some(Code::F4),
            5 => Some(Code::F5),
            6 => Some(Code::F6),
            7 => Some(Code::F7),
            8 => Some(Code::F8),
            9 => Some(Code::F9),
            10 => Some(Code::F10),
            11 => Some(Code::F11),
            12 => Some(Code::F12),
            _ => None,
        };
    }
    match u.as_str() {
        "SPACE" => Some(Code::Space),
        "ENTER" | "RETURN" => Some(Code::Enter),
        "TAB" => Some(Code::Tab),
        "ESC" | "ESCAPE" => Some(Code::Escape),
        "DELETE" | "DEL" => Some(Code::Delete),
        "INSERT" | "INS" => Some(Code::Insert),
        "HOME" => Some(Code::Home),
        "END" => Some(Code::End),
        "PAGEUP" | "PGUP" => Some(Code::PageUp),
        "PAGEDOWN" | "PGDN" => Some(Code::PageDown),
        "UP" => Some(Code::ArrowUp),
        "DOWN" => Some(Code::ArrowDown),
        "LEFT" => Some(Code::ArrowLeft),
        "RIGHT" => Some(Code::ArrowRight),
        "BACKSPACE" => Some(Code::Backspace),
        "MINUS" | "-" => Some(Code::Minus),
        "EQUAL" | "=" => Some(Code::Equal),
        "COMMA" | "," => Some(Code::Comma),
        "PERIOD" | "." => Some(Code::Period),
        "SEMICOLON" | ";" => Some(Code::Semicolon),
        "SLASH" | "/" => Some(Code::Slash),
        "BACKQUOTE" | "`" => Some(Code::Backquote),
        _ => None,
    }
}

// ------------------------------------------------------------------
// ホットキー登録管理
// ------------------------------------------------------------------

/// 登録済みホットキーの管理。GlobalHotKeyManager は main スレッド（boot 内）で生成すること。
pub struct HotkeyRegistry {
    manager: GlobalHotKeyManager,
    /// 登録中: HotKey::id → 用途
    pub bindings: std::collections::HashMap<u32, HotkeyAction>,
    registered: Vec<HotKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyAction {
    /// メインウィンドウ表示トグル
    ToggleMain,
    /// アイテム直接起動（LauncherItem.id）
    LaunchItem(String),
}

impl HotkeyRegistry {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            manager: GlobalHotKeyManager::new().map_err(|e| e.to_string())?,
            bindings: std::collections::HashMap::new(),
            registered: Vec::new(),
        })
    }

    /// 全解除して登録し直す。(hotkey文字列, アクション) のリスト。
    /// 戻り値: 登録に失敗した (hotkey文字列, 理由) のリスト。
    pub fn rebind(&mut self, wanted: &[(String, HotkeyAction)]) -> Vec<(String, String)> {
        for hk in self.registered.drain(..) {
            let _ = self.manager.unregister(hk);
        }
        self.bindings.clear();
        let mut failures = Vec::new();
        for (spec, action) in wanted {
            match parse_hotkey(spec) {
                Ok(hk) => match self.manager.register(hk) {
                    Ok(()) => {
                        self.bindings.insert(hk.id(), action.clone());
                        self.registered.push(hk);
                    }
                    Err(e) => failures.push((spec.clone(), e.to_string())),
                },
                Err(e) => failures.push((spec.clone(), e)),
            }
        }
        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_parsing() {
        assert!(parse_hotkey("Ctrl+Space").is_ok());
        assert!(parse_hotkey("Ctrl+Alt+A").is_ok());
        assert!(parse_hotkey("F5").is_ok());
        assert!(parse_hotkey("Win+Z").is_ok());
        assert!(parse_hotkey("Ctrl+").is_err());
        assert!(parse_hotkey("Ctrl+Foo").is_err());
        assert!(parse_hotkey("").is_err());
    }
}
