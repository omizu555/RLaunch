//! アプリ・ファイル・URL の起動（ShellExecuteW）。
//! コンソールウィンドウのフラッシュを避けるため Command は使わない（旧版と同方針）。

use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_CANCELLED};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, SW_SHOWNORMAL,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Maximized,
    Minimized,
}

impl WindowState {
    pub fn from_setting(s: Option<&str>) -> Self {
        match s {
            Some("maximized") => WindowState::Maximized,
            Some("minimized") => WindowState::Minimized,
            _ => WindowState::Normal,
        }
    }
}

/// UTF-16 ヌル終端ワイド文字列に変換
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// ShellExecuteW の戻り値（HINSTANCE <= 32）を日本語エラーメッセージへ変換
fn shell_error_message(code: isize) -> String {
    let msg = match code {
        0 => "メモリまたはリソースが不足しています",
        2 => "ファイルが見つかりません", // SE_ERR_FNF
        3 => "パスが見つかりません",     // SE_ERR_PNF
        5 => "アクセスが拒否されました", // SE_ERR_ACCESSDENIED
        8 => "メモリが不足しています",   // SE_ERR_OOM
        26 => "共有違反が発生しました",  // SE_ERR_SHARE
        27 => "ファイルの関連付けが不完全または無効です", // SE_ERR_ASSOCINCOMPLETE
        28..=30 => "アプリケーションとの連携（DDE）に失敗しました", // SE_ERR_DDE*
        31 => "このファイルに関連付けられたアプリケーションがありません", // SE_ERR_NOASSOC
        32 => "必要な DLL が見つかりません", // SE_ERR_DLLNOTFOUND
        _ => return format!("起動に失敗しました (ShellExecute code {code})"),
    };
    format!("{msg} (code {code})")
}

/// ShellExecuteW 共通ラッパ。verb/file/params/dir をワイド文字列化して呼び、
/// 失敗（HINSTANCE <= 32）はエラーメッセージで返す。
fn shell_execute(
    verb: &str,
    file: &str,
    params: Option<&str>,
    dir: Option<&str>,
    show: SHOW_WINDOW_CMD,
) -> Result<(), String> {
    let verb_w = to_wide(verb);
    let file_w = to_wide(file);
    // Option はワイド文字列のバッファを呼び出し中生かすため、先に束縛してからポインタ化
    let params_w = params.map(to_wide);
    let dir_w = dir.map(to_wide);
    let params_ptr = params_w
        .as_ref()
        .map_or(PCWSTR::null(), |v| PCWSTR(v.as_ptr()));
    let dir_ptr = dir_w
        .as_ref()
        .map_or(PCWSTR::null(), |v| PCWSTR(v.as_ptr()));

    // ShellExecuteW は成功時 32 より大きい HINSTANCE（互換用の擬似値）を返す。
    // ハンドル等の資源は返らないので解放は不要。
    let (code, last_error) = unsafe {
        let hinst = ShellExecuteW(
            None,
            PCWSTR(verb_w.as_ptr()),
            PCWSTR(file_w.as_ptr()),
            params_ptr,
            dir_ptr,
            show,
        );
        // 失敗判定に使う GetLastError は ShellExecuteW 直後に取得する
        (hinst.0 as isize, GetLastError())
    };

    if code > 32 {
        return Ok(());
    }

    // "runas" で UAC ダイアログをユーザーがキャンセルすると
    // 戻り値 5 (SE_ERR_ACCESSDENIED) + GetLastError = ERROR_CANCELLED になる
    if last_error == ERROR_CANCELLED {
        return Err("キャンセルされました".to_string());
    }
    Err(shell_error_message(code))
}

/// WindowState → SW_* 定数
fn to_show_cmd(state: WindowState) -> SHOW_WINDOW_CMD {
    match state {
        WindowState::Normal => SW_SHOWNORMAL,
        WindowState::Maximized => SW_SHOWMAXIMIZED,
        WindowState::Minimized => SW_SHOWMINIMIZED,
    }
}

/// ShellExecuteW verb "open"。URL/ドキュメント/フォルダも関連付けで開ける。
/// window_state は SW_SHOWNORMAL / SW_SHOWMAXIMIZED / SW_SHOWMINIMIZED に対応。
pub fn shell_open(
    path: &str,
    args: Option<&str>,
    working_dir: Option<&str>,
    window_state: WindowState,
) -> Result<(), String> {
    shell_execute("open", path, args, working_dir, to_show_cmd(window_state))
}

/// ShellExecuteW verb "runas"（UAC 昇格）。ユーザーが UAC をキャンセルした場合は
/// ERROR_CANCELLED をエラーとして返す。
pub fn shell_runas(path: &str, args: Option<&str>) -> Result<(), String> {
    shell_execute("runas", path, args, None, SW_SHOWNORMAL)
}

/// explorer.exe /select,<path> で親フォルダを開いてファイルを選択状態にする
pub fn open_file_location(path: &str) -> Result<(), String> {
    // 存在しないパスを渡すと explorer が無関係のフォルダ（ドキュメント等）を
    // 開いてしまうため、事前に存在チェックしてエラーにする
    if !std::path::Path::new(path).exists() {
        return Err(format!("パスが見つかりません: {path}"));
    }
    // 空白を含むパスに備えて引用符で囲む
    let params = format!("/select,\"{path}\"");
    shell_execute("open", "explorer.exe", Some(&params), None, SW_SHOWNORMAL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_state_from_str_の変換() {
        assert_eq!(
            WindowState::from_setting(Some("maximized")),
            WindowState::Maximized
        );
        assert_eq!(
            WindowState::from_setting(Some("minimized")),
            WindowState::Minimized
        );
        assert_eq!(
            WindowState::from_setting(Some("normal")),
            WindowState::Normal
        );
        assert_eq!(
            WindowState::from_setting(Some("unknown")),
            WindowState::Normal
        );
        assert_eq!(WindowState::from_setting(None), WindowState::Normal);
    }

    #[test]
    fn to_wide_はヌル終端() {
        let w = to_wide("ab");
        assert_eq!(w, vec![97, 98, 0]);
        assert_eq!(to_wide(""), vec![0]);
    }

    #[test]
    fn shell_open_存在しないパスはファイルが見つからないエラー() {
        // 存在しない exe パス → ShellExecuteW が 2 (SE_ERR_FNF) を返す
        // （何も起動されないのでテストから実行して安全）
        let err = shell_open(
            r"C:\__rlaunch_test_no_such_dir__\no_such_app.exe",
            None,
            None,
            WindowState::Normal,
        )
        .expect_err("存在しないパスで Ok が返った");
        assert!(
            err.contains("見つかりません"),
            "エラーメッセージが想定外: {err}"
        );
    }

    #[test]
    fn shell_error_message_のコード別メッセージ() {
        assert!(shell_error_message(2).contains("ファイルが見つかりません"));
        assert!(shell_error_message(3).contains("パスが見つかりません"));
        assert!(shell_error_message(5).contains("アクセスが拒否"));
        assert!(shell_error_message(31).contains("関連付けられたアプリケーション"));
        assert!(shell_error_message(32).contains("DLL"));
        // 未知コードもコード番号入りで返る
        assert!(shell_error_message(1).contains("code 1"));
    }

    #[test]
    fn open_file_location_存在しないパスは起動せずエラー() {
        // 事前チェックで弾かれるため explorer は起動しない
        let err = open_file_location(r"C:\__rlaunch_test_no_such_dir__\missing.txt")
            .expect_err("存在しないパスで Ok が返った");
        assert!(
            err.contains("パスが見つかりません"),
            "エラーメッセージが想定外: {err}"
        );
    }
}
