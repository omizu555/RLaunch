use tauri::{
    Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
};

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // --- Plugins ---
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 二重起動時: 既存ウィンドウを表示
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        // --- Commands ---
        .invoke_handler(tauri::generate_handler![
            commands::file_info::get_file_info,
            commands::icon_extractor::extract_icon,
            commands::lnk_resolver::resolve_lnk,
            commands::system_info::get_system_info,
            commands::app_launcher::launch_app,
            commands::app_launcher::get_store_path,
            commands::app_launcher::run_as_admin,
            commands::app_launcher::open_file_location,
            commands::app_launcher::set_window_effect,
            commands::app_launcher::get_cursor_position,
            commands::app_launcher::hide_webview_window,
            commands::app_launcher::get_cursor_monitor_info,
            commands::app_launcher::list_directory,
            commands::theme_manager::init_themes,
            commands::theme_manager::list_themes,
            commands::theme_manager::get_themes_dir_path,
            commands::widget_manager::init_widgets,
            commands::widget_manager::list_widgets,
            commands::widget_manager::get_widget_script,
            commands::widget_manager::get_widgets_dir_path,
            commands::audio::pick_sound_file,
            commands::audio::read_sound_file,
            commands::icon_library::init_icon_library,
            commands::icon_library::list_icon_library,
            commands::icon_library::get_icon_library_dir_path,
        ])
        // --- Setup ---
        .setup(|app| {
            // システムトレイの構築
            let show_i = MenuItem::with_id(app, "show", "表示/非表示", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "設定", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &settings_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("RLaunch")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                    "settings" => {
                        // TODO: 設定画面を開く
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左クリックで表示/非表示トグル
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // themes/ フォルダの初期化（ビルトイン + サンプルテーマ書き出し）
            let handle = app.handle().clone();
            let _ = commands::theme_manager::init_themes(handle);

            // widgets/ フォルダの初期化（ビルトイン + サンプルウィジェット書き出し）
            let handle2 = app.handle().clone();
            let _ = commands::widget_manager::init_widgets(handle2);

            // icons/ フォルダの初期化（デフォルトアイコン書き出し）
            let handle3 = app.handle().clone();
            let _ = commands::icon_library::init_icon_library(handle3);

            // デスクトップダブルクリックフックのセットアップ
            commands::desktop_hook::setup_desktop_hook(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
