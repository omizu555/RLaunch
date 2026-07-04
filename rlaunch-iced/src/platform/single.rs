//! 単一インスタンス化（名前付きミューテックス）と、二重起動時の「表示せよ」通知（名前付きイベント）。

use windows::core::HSTRING;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE,
    INFINITE,
};

pub const MUTEX_NAME: &str = "Local\\rlaunch-iced-single-instance";
pub const SHOW_EVENT_NAME: &str = "Local\\rlaunch-iced-show-request";

/// ミューテックス保持ハンドル。drop しても解放しない（プロセス生存中保持）想定でよい。
pub struct SingleInstanceGuard {
    #[allow(dead_code)]
    handle: isize,
}

/// CreateMutexW して ERROR_ALREADY_EXISTS なら None（=既に起動中）。
pub fn acquire() -> Option<SingleInstanceGuard> {
    acquire_named(MUTEX_NAME)
}

/// 名前を指定してミューテックスを取得する内部実装（テストから固有名で呼べるよう分離）。
fn acquire_named(name: &str) -> Option<SingleInstanceGuard> {
    unsafe {
        // CreateMutexW は同名ミューテックスが既に存在する場合も「有効なハンドル」を返し、
        // GetLastError() が ERROR_ALREADY_EXISTS になる（失敗ではない点に注意）。
        // windows クレートの Ok パスは追加の Win32 呼び出しをしないため、
        // 直後の GetLastError() の値は保存されている。
        let handle = CreateMutexW(None, false, &HSTRING::from(name)).ok()?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // 既に他インスタンスが起動中 → 取得したハンドルは閉じて None を返す
            let _ = CloseHandle(handle);
            return None;
        }
        Some(SingleInstanceGuard {
            handle: handle.0 as isize,
        })
    }
}

/// 既存インスタンスに表示要求を送る（二重起動プロセス側から呼び、その後即終了する）。
/// OpenEventW + SetEvent。イベントが無ければ何もしない。
pub fn signal_show() {
    signal_show_named(SHOW_EVENT_NAME);
}

/// 名前を指定してイベントをシグナルする内部実装。
fn signal_show_named(name: &str) {
    unsafe {
        // 常駐側が存在しない（イベント未作成）場合は OpenEventW が失敗する → 黙って成功扱い
        if let Ok(handle) = OpenEventW(EVENT_MODIFY_STATE, false, &HSTRING::from(name)) {
            let _ = SetEvent(handle);
            let _ = CloseHandle(handle);
        }
    }
}

/// 常駐側: 名前付きイベントを作成し、待機スレッドを起動。シグナルのたびに on_show を呼ぶ。
/// スレッドはプロセス終了まで生きてよい。
pub fn spawn_show_listener(on_show: impl Fn() + Send + 'static) -> Result<(), String> {
    spawn_show_listener_named(SHOW_EVENT_NAME, on_show)
}

/// 名前を指定してリスナーを起動する内部実装。
fn spawn_show_listener_named(
    name: &str,
    on_show: impl Fn() + Send + 'static,
) -> Result<(), String> {
    // 自動リセットイベント（bManualReset=false）: WaitForSingleObject で待機が解除された時点で
    // 自動的に非シグナル状態へ戻るため、ResetEvent は不要。
    let handle = unsafe {
        CreateEventW(None, false, false, &HSTRING::from(name))
            .map_err(|e| format!("CreateEventW({name}) に失敗: {e}"))?
    };
    // HANDLE は生ポインタを含み Send でないため、isize に変換してスレッドへ運ぶ
    // （カーネルハンドル自体はプロセス内のどのスレッドからでも使用可能）。
    let raw = handle.0 as isize;
    let spawned = std::thread::Builder::new()
        .name("rlaunch-show-listener".into())
        .spawn(move || {
            let handle = HANDLE(raw as *mut core::ffi::c_void);
            loop {
                let wait = unsafe { WaitForSingleObject(handle, INFINITE) };
                if wait != WAIT_OBJECT_0 {
                    // WAIT_FAILED 等 → ハンドル異常。ループを抜けて終了する
                    break;
                }
                on_show();
            }
            // 通常ここには到達しない（プロセス終了まで常駐）が、抜けた場合は解放する
            unsafe {
                let _ = CloseHandle(handle);
            }
        });
    if let Err(e) = spawned {
        // スレッドが起動できなかった場合はイベントハンドルを漏らさず閉じる
        unsafe {
            let _ = CloseHandle(HANDLE(raw as *mut core::ffi::c_void));
        }
        return Err(format!("リスナースレッド起動に失敗: {e}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[test]
    fn acquire_detects_existing_mutex() {
        // 実アプリと衝突しないよう、テスト専用のユニーク名を使う
        let name = format!("Local\\rlaunch-iced-test-mutex-{}", std::process::id());
        let first = acquire_named(&name);
        assert!(first.is_some(), "1回目の取得は成功するはず");
        // 保持したまま同名で2回目 → ERROR_ALREADY_EXISTS を検知して None
        let second = acquire_named(&name);
        assert!(second.is_none(), "2回目の取得は既存検知で None になるはず");
    }

    #[test]
    fn listener_receives_signal() {
        let name = format!("Local\\rlaunch-iced-test-event-{}", std::process::id());
        let count = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&count);
        spawn_show_listener_named(&name, move || {
            c.fetch_add(1, Ordering::SeqCst);
        })
        .expect("リスナー起動に失敗");

        signal_show_named(&name);

        // タイムアウト付きでコールバック発火を待つ
        let deadline = Instant::now() + Duration::from_secs(5);
        while count.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            count.load(Ordering::SeqCst) >= 1,
            "signal_show 後にコールバックが呼ばれるはず"
        );
    }

    #[test]
    fn signal_without_listener_is_silent_noop() {
        // 存在しないイベント名に対しては何もせず正常終了する（panic しない）
        signal_show_named(&format!(
            "Local\\rlaunch-iced-test-no-such-event-{}",
            std::process::id()
        ));
    }
}
