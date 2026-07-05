#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rlaunch::app::App;

/// フォント設定が無いときの既定（日本語UIの可読性優先）
const DEFAULT_FONT: &str = "Yu Gothic UI";

fn main() -> iced::Result {
    // 二重起動なら既存インスタンスに表示要求を送って即終了
    let Some(_single_guard) = rlaunch::platform::single::acquire() else {
        rlaunch::platform::single::signal_show();
        return Ok(());
    };

    // 外部イベントチャネル（ホットキー/トレイ/フック → Subscription）を最初に初期化する。
    // これを忘れると external::send が全て捨てられ、常駐操作が一切効かなくなる。
    rlaunch::external::init();

    // UIフォント（Font::with_name は 'static を要求するため一度だけ leak する）
    let font_name: &'static str = Box::leak(
        rlaunch::model::store::peek_font_family()
            .unwrap_or_else(|| DEFAULT_FONT.to_string())
            .into_boxed_str(),
    );

    iced::daemon(App::boot, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title(App::title)
        .default_font(iced::Font::with_name(font_name))
        .run()
}
