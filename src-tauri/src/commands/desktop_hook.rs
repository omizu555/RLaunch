/* ============================================================
   desktop_hook - デスクトップダブルクリックでランチャー表示/非表示
   Windows 低レベルマウスフックを使用してデスクトップ上のダブルクリックを検出し、
   メインウィンドウの表示/非表示をトグルする
   ============================================================ */
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
mod win {
    use super::*;
    use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
    use std::sync::OnceLock;
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
    use windows::Win32::UI::WindowsAndMessaging::*;

    static SENDER: OnceLock<std::sync::mpsc::Sender<(i32, i32)>> = OnceLock::new();
    static LAST_CLICK_TIME: AtomicU32 = AtomicU32::new(0);
    static LAST_CLICK_X: AtomicI32 = AtomicI32::new(0);
    static LAST_CLICK_Y: AtomicI32 = AtomicI32::new(0);

    /// ウィンドウクラス名を取得
    fn get_class_name_str(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        if len == 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..len as usize])
    }

    /// 指定ウィンドウがデスクトップ(Progman/WorkerW)の子孫かどうか判定
    fn is_desktop_window(hwnd: HWND) -> bool {
        let mut current = hwnd;
        for _ in 0..10 {
            let class = get_class_name_str(current);
            if class == "Progman" || class == "WorkerW" {
                return true;
            }
            let parent = unsafe { GetParent(current) };
            if parent == HWND::default() || parent == current {
                break;
            }
            current = parent;
        }
        false
    }

    /// 低レベルマウスフックコールバック
    unsafe extern "system" fn mouse_hook_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 && w_param == WPARAM(WM_LBUTTONDOWN as usize) {
            let info = &*(l_param.0 as *const MSLLHOOKSTRUCT);
            let now = info.time;
            let prev_time = LAST_CLICK_TIME.load(Ordering::Relaxed);
            let prev_x = LAST_CLICK_X.load(Ordering::Relaxed);
            let prev_y = LAST_CLICK_Y.load(Ordering::Relaxed);

            let double_click_time = GetDoubleClickTime();
            let cx = GetSystemMetrics(SM_CXDOUBLECLK);
            let cy = GetSystemMetrics(SM_CYDOUBLECLK);

            let dt = now.wrapping_sub(prev_time);
            let dx = (info.pt.x - prev_x).abs();
            let dy = (info.pt.y - prev_y).abs();

            if prev_time != 0 && dt <= double_click_time && dx <= cx && dy <= cy {
                // ダブルクリック検出 → デスクトップかチェック
                let target = WindowFromPoint(info.pt);
                if is_desktop_window(target) {
                    if let Some(sender) = SENDER.get() {
                        let _ = sender.send((info.pt.x, info.pt.y));
                    }
                }
                // トリプルクリック防止用にリセット
                LAST_CLICK_TIME.store(0, Ordering::Relaxed);
            } else {
                LAST_CLICK_TIME.store(now, Ordering::Relaxed);
                LAST_CLICK_X.store(info.pt.x, Ordering::Relaxed);
                LAST_CLICK_Y.store(info.pt.y, Ordering::Relaxed);
            }
        }
        CallNextHookEx(HHOOK::default(), n_code, w_param, l_param)
    }

    /// デスクトップダブルクリックフックを開始
    pub fn start(app: &AppHandle) {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = SENDER.set(tx);

        // ウィンドウ表示スレッド: フックからの通知を処理
        let app_clone = app.clone();
        std::thread::spawn(move || {
            while let Ok((x, y)) = rx.recv() {
                if let Some(w) = app_clone.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        // カーソル位置を中心にウィンドウを配置（マルチモニター対応）
                        if let Ok(size) = w.outer_size() {
                            use windows::Win32::Foundation::POINT;
                            use windows::Win32::Graphics::Gdi::{
                                MonitorFromPoint, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
                            };
                            let point = POINT { x, y };
                            let hmon = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
                            let mut mi = MONITORINFO {
                                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                                ..Default::default()
                            };
                            let ok = unsafe { GetMonitorInfoW(hmon, &mut mi) };
                            let (mon_x, mon_y, mon_r, mon_b) = if ok.as_bool() {
                                (mi.rcWork.left, mi.rcWork.top, mi.rcWork.right, mi.rcWork.bottom)
                            } else {
                                (0, 0, i32::MAX, i32::MAX)
                            };
                            let mut wx = x - size.width as i32 / 2;
                            let mut wy = y - size.height as i32 / 2;
                            if wx + size.width as i32 > mon_r { wx = mon_r - size.width as i32; }
                            if wy + size.height as i32 > mon_b { wy = mon_b - size.height as i32; }
                            if wx < mon_x { wx = mon_x; }
                            if wy < mon_y { wy = mon_y; }
                            let _ = w.set_position(tauri::PhysicalPosition::new(wx, wy));
                        }
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        });

        // フックスレッド: マウスフック + メッセージポンプ
        std::thread::spawn(|| unsafe {
            match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), HINSTANCE::default(), 0) {
                Ok(hook) => {
                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    let _ = UnhookWindowsHookEx(hook);
                }
                Err(e) => {
                    eprintln!("Desktop hook failed: {e}");
                }
            }
        });
    }
}

/// アプリ起動時にデスクトップダブルクリックフックをセットアップ
pub fn setup_desktop_hook(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    win::start(app);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
    }
}
