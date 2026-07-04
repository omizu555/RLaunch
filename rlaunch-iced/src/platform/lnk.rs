//! .lnk ショートカット解決（IShellLinkW + IPersistFile、COM ネイティブ）。
//! 旧版の PowerShell 子プロセス方式を置き換える。
//!
//! COM: CoInitializeEx(COINIT_APARTMENTTHREADED)、S_FALSE は成功扱い。
//! IShellLinkW::Resolve は SLR_NO_UI で UI を出さない。

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, RPC_E_CHANGED_MODE};
use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::UI::Shell::{
    IShellLinkW, ShellLink, SLGP_UNCPRIORITY, SLR_NOUPDATE, SLR_NO_UI,
};

#[derive(Debug, Clone, Default)]
pub struct LnkInfo {
    pub target_path: String,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
    pub icon_location: Option<String>,
}

/// 受けバッファ長。MAX_PATH(260) の 2 倍で長めのパスにも備える。
const BUF_LEN: usize = 260 * 2;

/// CoInitializeEx の RAII ガード。
/// - S_OK / S_FALSE（既に同モデルで初期化済み）は成功 → Drop で CoUninitialize を対で呼ぶ。
/// - RPC_E_CHANGED_MODE（別モデルで初期化済み）は COM 自体は使えるので続行するが、
///   自分は初期化していないため CoUninitialize は呼ばない。
struct ComInit {
    needs_uninit: bool,
}

impl ComInit {
    fn new() -> Result<Self, String> {
        // 戻り値は生の HRESULT。is_ok() は S_FALSE(0x1) も真になる（成功扱い）。
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr.is_ok() {
            Ok(Self { needs_uninit: true })
        } else if hr == RPC_E_CHANGED_MODE {
            Ok(Self {
                needs_uninit: false,
            })
        } else {
            Err(format!("CoInitializeEx 失敗: {hr}"))
        }
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        if self.needs_uninit {
            // 成功した CoInitializeEx（S_FALSE 含む）と必ず対にする。
            unsafe { CoUninitialize() };
        }
    }
}

/// UTF-16（null 終端付き）へ変換する。
fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// null 終端までを String にする。空文字列なら None。
fn wide_buf_to_opt(buf: &[u16]) -> Option<String> {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    if len == 0 {
        None
    } else {
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}

/// .lnk ファイルのリンク先情報を返す。ターゲットが取れない場合は Err。
pub fn resolve_lnk(path: &str) -> Result<LnkInfo, String> {
    // ガードは最初に作る（宣言順の逆で Drop されるため、COM オブジェクト解放後に
    // CoUninitialize が走る）。
    let _com = ComInit::new()?;

    unsafe {
        // ShellLink コクラス → IShellLinkW → IPersistFile（QueryInterface 相当の cast）。
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("ShellLink の生成に失敗: {e}"))?;
        let persist: IPersistFile = link
            .cast()
            .map_err(|e| format!("IPersistFile の取得に失敗: {e}"))?;

        // .lnk を読み込む。存在しない・壊れている場合はここで Err。
        let wide_path = to_wide(path);
        persist
            .Load(PCWSTR(wide_path.as_ptr()), STGM_READ)
            .map_err(|e| format!(".lnk の読み込みに失敗 ({path}): {e}"))?;

        // リンク切れ時のターゲット再探索。SLR_NO_UI でダイアログ抑止、
        // SLR_NOUPDATE で .lnk 自体を書き換えない。失敗しても Load 済みの
        // 情報（古いパス等）は取得できるため、致命扱いにしない。
        let _ = link.Resolve(HWND::default(), (SLR_NO_UI.0 | SLR_NOUPDATE.0) as u32);

        // リンク先パス。SLGP_UNCPRIORITY でネットワークドライブなら UNC を優先。
        let mut buf = [0u16; BUF_LEN];
        let mut fd = WIN32_FIND_DATAW::default();
        link.GetPath(&mut buf, &mut fd, SLGP_UNCPRIORITY.0 as u32)
            .map_err(|e| format!("リンク先の取得に失敗 ({path}): {e}"))?;
        let target_path = match wide_buf_to_opt(&buf) {
            Some(t) => t,
            // GetPath は対象なしでも S_FALSE（成功扱い）で空を返すことがある。
            None => return Err(format!("リンク先を解決できません: {path}")),
        };

        // 引数・作業ディレクトリ・アイコン位置。空文字列は None に正規化。
        // 個別の取得失敗はターゲットが取れていれば致命ではないので None に落とす。
        let mut buf = [0u16; BUF_LEN];
        let arguments = link
            .GetArguments(&mut buf)
            .ok()
            .and_then(|_| wide_buf_to_opt(&buf));

        let mut buf = [0u16; BUF_LEN];
        let working_dir = link
            .GetWorkingDirectory(&mut buf)
            .ok()
            .and_then(|_| wide_buf_to_opt(&buf));

        // アイコンのインデックスは現状使わないためパスのみ返す。
        let mut buf = [0u16; BUF_LEN];
        let mut icon_index: i32 = 0;
        let icon_location = link
            .GetIconLocation(&mut buf, &mut icon_index)
            .ok()
            .and_then(|_| wide_buf_to_opt(&buf));

        Ok(LnkInfo {
            target_path,
            arguments,
            working_dir,
            icon_location,
        })
    }
    // link / persist はここで Drop（Release）→ その後 _com が CoUninitialize。
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用の一時 .lnk を IShellLinkW::SetPath + IPersistFile::Save で作る。
    /// WScript.Shell（子プロセス）は使わない。
    fn create_test_lnk(
        lnk_path: &str,
        target: &str,
        args: Option<&str>,
        workdir: Option<&str>,
    ) -> Result<(), String> {
        let _com = ComInit::new()?;
        unsafe {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("ShellLink の生成に失敗: {e}"))?;

            let wide_target = to_wide(target);
            link.SetPath(PCWSTR(wide_target.as_ptr()))
                .map_err(|e| format!("SetPath 失敗: {e}"))?;
            if let Some(a) = args {
                let w = to_wide(a);
                link.SetArguments(PCWSTR(w.as_ptr()))
                    .map_err(|e| format!("SetArguments 失敗: {e}"))?;
            }
            if let Some(d) = workdir {
                let w = to_wide(d);
                link.SetWorkingDirectory(PCWSTR(w.as_ptr()))
                    .map_err(|e| format!("SetWorkingDirectory 失敗: {e}"))?;
            }

            let persist: IPersistFile = link
                .cast()
                .map_err(|e| format!("IPersistFile の取得に失敗: {e}"))?;
            let wide_lnk = to_wide(lnk_path);
            persist
                .Save(PCWSTR(wide_lnk.as_ptr()), true)
                .map_err(|e| format!("Save 失敗: {e}"))?;
        }
        Ok(())
    }

    #[test]
    fn resolve_lnk_roundtrip() {
        // 確実に存在するファイルとして自分自身（テスト実行バイナリ）を使う。
        let target = std::env::current_exe()
            .expect("current_exe 取得失敗")
            .to_string_lossy()
            .into_owned();
        let workdir = std::env::temp_dir()
            .to_string_lossy()
            .trim_end_matches('\\')
            .to_string();
        let lnk_path = std::env::temp_dir()
            .join(format!("rlaunch_lnk_test_{}.lnk", std::process::id()))
            .to_string_lossy()
            .into_owned();

        create_test_lnk(&lnk_path, &target, Some("--foo bar"), Some(&workdir))
            .expect("テスト用 .lnk の作成に失敗");

        let result = resolve_lnk(&lnk_path);
        // テスト後に必ず削除（assert 前に消してもファイルは不要）。
        let _ = std::fs::remove_file(&lnk_path);

        let info = result.expect("resolve_lnk 失敗");
        // パスの大文字小文字は環境で揺れるため無視して比較。
        assert_eq!(
            info.target_path.to_lowercase(),
            target.to_lowercase(),
            "ターゲットが一致すること"
        );
        assert_eq!(info.arguments.as_deref(), Some("--foo bar"));
        assert_eq!(
            info.working_dir.map(|s| s.to_lowercase()),
            Some(workdir.to_lowercase())
        );
        // アイコン位置は未設定なので None（空文字列 → None 正規化の確認）。
        assert_eq!(info.icon_location, None);
    }

    #[test]
    fn resolve_lnk_missing_file_is_err() {
        let missing = std::env::temp_dir()
            .join("rlaunch_lnk_test_no_such_file_xyz.lnk")
            .to_string_lossy()
            .into_owned();
        assert!(resolve_lnk(&missing).is_err(), "存在しない .lnk は Err");
    }
}
