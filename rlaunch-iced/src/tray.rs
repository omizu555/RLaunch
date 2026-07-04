//! システムトレイ常駐。
//! 旧版同等: 左クリック=表示/非表示トグル、右クリックメニュー（表示/非表示・設定・終了）。
//! TrayIcon インスタンスは drop するとトレイから消えるため、呼び出し側（App）が保持し続けること。

use crate::external::{send, ExternalEvent};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

const ICON_PNG: &[u8] = include_bytes!("../assets/icon-32.png");

const MENU_ID_TOGGLE: &str = "rlaunch-toggle";
const MENU_ID_SETTINGS: &str = "rlaunch-settings";
const MENU_ID_QUIT: &str = "rlaunch-quit";

/// トレイアイコンを構築する。main スレッド（boot 内）で呼ぶこと。
pub fn build_tray(tooltip: &str) -> Result<TrayIcon, String> {
    let img = image::load_from_memory(ICON_PNG)
        .map_err(|e| format!("トレイアイコン画像の読み込み失敗: {}", e))?
        .into_rgba8();
    let (w, h) = img.dimensions();
    let icon = tray_icon::Icon::from_rgba(img.into_raw(), w, h)
        .map_err(|e| format!("トレイアイコン生成失敗: {}", e))?;

    let menu = Menu::new();
    let toggle = MenuItem::with_id(MENU_ID_TOGGLE, "表示/非表示", true, None);
    let settings = MenuItem::with_id(MENU_ID_SETTINGS, "設定", true, None);
    let quit = MenuItem::with_id(MENU_ID_QUIT, "終了", true, None);
    menu.append_items(&[&toggle, &settings, &quit])
        .map_err(|e| format!("トレイメニュー構築失敗: {}", e))?;

    // 左クリックはメニューを出さずトグルに使う。
    // Down 基準にする理由: ダブルクリック時のシーケンス（DOWN/UP/DBLCLK/UP）では
    // UP が2回届くため、Up 基準だとトグルが2回走って元に戻ってしまう。
    TrayIconEvent::set_event_handler(Some(|ev: TrayIconEvent| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Down,
            ..
        } = ev
        {
            send(ExternalEvent::TrayToggle);
        }
    }));
    MenuEvent::set_event_handler(Some(|ev: MenuEvent| match ev.id.0.as_str() {
        MENU_ID_TOGGLE => send(ExternalEvent::TrayMenuToggle),
        MENU_ID_SETTINGS => send(ExternalEvent::TrayMenuSettings),
        MENU_ID_QUIT => send(ExternalEvent::TrayMenuQuit),
        _ => {}
    }));

    TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip(tooltip)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()
        .map_err(|e| format!("トレイアイコン構築失敗: {}", e))
}
