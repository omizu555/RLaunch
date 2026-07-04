#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rlaunch::app::App;

fn main() -> iced::Result {
    // 二重起動なら既存インスタンスに表示要求を送って即終了
    let Some(_single_guard) = rlaunch::platform::single::acquire() else {
        rlaunch::platform::single::signal_show();
        return Ok(());
    };

    // 外部イベントチャネル（ホットキー/トレイ/フック → Subscription）を最初に初期化する。
    // これを忘れると external::send が全て捨てられ、常駐操作が一切効かなくなる。
    rlaunch::external::init();

    iced::daemon(App::boot, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title(App::title)
        .run()
}
