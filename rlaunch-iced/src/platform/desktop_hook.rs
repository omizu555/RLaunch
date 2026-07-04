//! デスクトップ（Progman/WorkerW）上のダブルクリック検出。
//! WH_MOUSE_LL 低レベルマウスフックを専用スレッドで張り、GetMessageW でポンプする。
//! 判定ロジックは旧版 src-tauri/src/commands/desktop_hook.rs の移植:
//! - WM_LBUTTONDOWN を GetDoubleClickTime() と SM_CXDOUBLECLK/SM_CYDOUBLECLK でダブルクリック判定
//! - WindowFromPoint → GetAncestor(GA_ROOT) → GetClassNameW が "Progman" か "WorkerW" のときのみ発火
//! - コールバックはフックスレッドから直接呼ばず、mpsc チャネル経由の転送スレッドで呼ぶ

use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::OnceLock;

use windows::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetAncestor, GetClassNameW, GetMessageW, GetSystemMetrics,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT, MSG,
    MSLLHOOKSTRUCT, SM_CXDOUBLECLK, SM_CYDOUBLECLK, WH_MOUSE_LL, WM_LBUTTONDOWN,
};

/// フックプロシージャ → 転送スレッドへの通知チャネル送信側。
/// フックプロシージャはグローバル関数のため、static で保持するしかない。
static SENDER: OnceLock<std::sync::mpsc::Sender<()>> = OnceLock::new();
/// 直前の WM_LBUTTONDOWN の時刻（ms、MSLLHOOKSTRUCT.time）。0 は「直前クリックなし」を表す。
static LAST_CLICK_TIME: AtomicU32 = AtomicU32::new(0);
static LAST_CLICK_X: AtomicI32 = AtomicI32::new(0);
static LAST_CLICK_Y: AtomicI32 = AtomicI32::new(0);

/// ダブルクリック判定（純粋関数。テストのために Win32 呼び出しから分離）。
/// prev_time == 0 は「直前クリックなし」。時刻は u32 ミリ秒で約49日で巻き戻るため
/// wrapping_sub で差分を取る。
#[allow(clippy::too_many_arguments)] // Win32 の生パラメータをそのまま検証したいため
fn is_double_click(
    prev_time: u32,
    prev_x: i32,
    prev_y: i32,
    now: u32,
    x: i32,
    y: i32,
    max_ms: u32,
    max_dx: i32,
    max_dy: i32,
) -> bool {
    if prev_time == 0 {
        return false;
    }
    let dt = now.wrapping_sub(prev_time);
    dt <= max_ms && (x - prev_x).abs() <= max_dx && (y - prev_y).abs() <= max_dy
}

/// クリック座標のトップレベルウィンドウがデスクトップ（Progman / WorkerW）かどうか判定。
fn is_desktop_at(pt: POINT) -> bool {
    unsafe {
        let hwnd = WindowFromPoint(pt);
        if hwnd.is_invalid() {
            return false;
        }
        // 子ウィンドウ（SysListView32 等）に当たることがあるためルートまで遡る
        let root = GetAncestor(hwnd, GA_ROOT);
        let target = if root.is_invalid() { hwnd } else { root };
        let mut buf = [0u16; 64];
        let len = GetClassNameW(target, &mut buf);
        if len <= 0 {
            return false;
        }
        let class = String::from_utf16_lossy(&buf[..len as usize]);
        class == "Progman" || class == "WorkerW"
    }
}

/// 低レベルマウスフックプロシージャ。
/// ここでの処理は最小限に留める（重い処理はチャネルの先の転送スレッドで行う）。
unsafe extern "system" fn mouse_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 && w_param.0 == WM_LBUTTONDOWN as usize {
        let info = unsafe { &*(l_param.0 as *const MSLLHOOKSTRUCT) };
        let now = info.time;
        let prev_time = LAST_CLICK_TIME.load(Ordering::Relaxed);
        let prev_x = LAST_CLICK_X.load(Ordering::Relaxed);
        let prev_y = LAST_CLICK_Y.load(Ordering::Relaxed);

        // システム設定のダブルクリック時間・許容移動量を毎回取得（設定変更に追従）
        let (max_ms, max_dx, max_dy) = unsafe {
            (
                GetDoubleClickTime(),
                GetSystemMetrics(SM_CXDOUBLECLK),
                GetSystemMetrics(SM_CYDOUBLECLK),
            )
        };

        if is_double_click(
            prev_time, prev_x, prev_y, now, info.pt.x, info.pt.y, max_ms, max_dx, max_dy,
        ) {
            // ダブルクリック確定 → クリック先がデスクトップのときのみ通知
            if is_desktop_at(info.pt) {
                if let Some(tx) = SENDER.get() {
                    let _ = tx.send(());
                }
            }
            // トリプルクリックを「2連続のダブルクリック」と誤認しないようリセット
            LAST_CLICK_TIME.store(0, Ordering::Relaxed);
        } else {
            LAST_CLICK_TIME.store(now, Ordering::Relaxed);
            LAST_CLICK_X.store(info.pt.x, Ordering::Relaxed);
            LAST_CLICK_Y.store(info.pt.y, Ordering::Relaxed);
        }
    }
    // フックチェーンは必ず次へ渡す
    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}

/// フックを開始する。成功したらプロセス終了まで常駐（明示的な解除 API は不要）。
pub fn spawn_desktop_double_click_hook(
    on_double_click: impl Fn() + Send + 'static,
) -> Result<(), String> {
    // 通知チャネルを用意。SENDER が既に設定済みなら二重開始なのでエラー。
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    SENDER
        .set(tx)
        .map_err(|_| "デスクトップフックは既に開始済み".to_string())?;

    // 転送スレッド: フックプロシージャから届いた通知ごとにコールバックを呼ぶ。
    // フックプロシージャ内で任意のユーザーコードを実行しないための分離。
    std::thread::Builder::new()
        .name("rlaunch-desktop-dblclick".into())
        .spawn(move || {
            while rx.recv().is_ok() {
                on_double_click();
            }
        })
        .map_err(|e| format!("転送スレッド起動に失敗: {e}"))?;

    // フック設置の成否をフックスレッドから同期的に受け取るためのチャネル
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    // フックスレッド: WH_MOUSE_LL はフックを張ったスレッドがメッセージポンプを
    // 回し続ける必要がある（ポンプが止まるとシステム全体のマウス応答が劣化する）。
    std::thread::Builder::new()
        .name("rlaunch-mouse-hook".into())
        .spawn(move || {
            // WH_MOUSE_LL はフックプロシージャが自プロセス内にあるため hmod は不要（None）
            let hook =
                match unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0) } {
                    Ok(h) => {
                        let _ = ready_tx.send(Ok(()));
                        h
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(format!("SetWindowsHookExW に失敗: {e}")));
                        return;
                    }
                };
            // メッセージポンプ（プロセス終了まで回り続ける）
            let mut msg = MSG::default();
            loop {
                let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if ret.0 <= 0 {
                    // 0 = WM_QUIT, -1 = エラー → ポンプ終了
                    break;
                }
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            // 通常ここには到達しないが、ポンプを抜けた場合はフックを解除する
            unsafe {
                let _ = UnhookWindowsHookEx(hook);
            }
        })
        .map_err(|e| format!("フックスレッド起動に失敗: {e}"))?;

    // フック設置の結果を待ってから返す（失敗を呼び出し側へ伝えるため）
    ready_rx
        .recv()
        .map_err(|_| "フックスレッドが応答せず終了".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_installs_and_unhooks() {
        // 実クリックのシミュレーションはせず、設置と即時解除のみ確認（CI 安全）。
        // このスレッドはメッセージポンプを回さないため、フックプロシージャは呼ばれない。
        unsafe {
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)
                .expect("WH_MOUSE_LL の設置に失敗");
            UnhookWindowsHookEx(hook).expect("フック解除に失敗");
        }
    }

    #[test]
    fn spawn_hook_reports_success_and_rejects_double_start() {
        // 設置成功が Result で返ること（フック/転送スレッドはプロセス終了まで残るが無害）
        spawn_desktop_double_click_hook(|| {}).expect("フック開始に失敗");
        // 二重開始は SENDER 設定済みのためエラーになる
        assert!(spawn_desktop_double_click_hook(|| {}).is_err());
    }

    #[test]
    fn double_click_detection_logic() {
        // 通常のダブルクリック（500ms 以内・4px 以内）
        assert!(is_double_click(1000, 100, 100, 1300, 102, 101, 500, 4, 4));
        // 境界ちょうどは許容（dt == max_ms, dx == max_dx）
        assert!(is_double_click(1000, 100, 100, 1500, 104, 100, 500, 4, 4));
        // 時間超過
        assert!(!is_double_click(1000, 100, 100, 1600, 100, 100, 500, 4, 4));
        // 距離超過（X）
        assert!(!is_double_click(1000, 100, 100, 1200, 110, 100, 500, 4, 4));
        // 距離超過（Y）
        assert!(!is_double_click(1000, 100, 100, 1200, 100, 110, 500, 4, 4));
        // 直前クリックなし（prev_time == 0）
        assert!(!is_double_click(0, 0, 0, 100, 0, 0, 500, 4, 4));
        // タイマー巻き戻り（u32 ラップ）でも wrapping_sub で正しく判定できる
        assert!(is_double_click(
            u32::MAX - 100,
            50,
            50,
            200,
            50,
            50,
            500,
            4,
            4
        ));
    }
}
