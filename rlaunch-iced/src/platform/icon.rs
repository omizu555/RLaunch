//! .exe/.lnk/フォルダ/ドキュメントからのアイコン抽出。
//! SHGetFileInfoW(SHGFI_ICON | SHGFI_LARGEICON) → HICON → GetIconInfo/GetDIBits → RGBA。
//! 高解像度が必要なら IShellItemImageFactory::GetImage(SIIGBF_ICONONLY) を第2候補に。
//!
//! COM: 呼び出し前に CoInitializeEx(COINIT_APARTMENTTHREADED) を行い S_FALSE は成功扱い。
//! どのスレッドから呼んでも安全なこと（内部で初期化を保証する）。
//! HICON/HBITMAP は DestroyIcon/DeleteObject で必ず解放する。

use windows::core::PCWSTR;
use windows::Win32::Foundation::{SIZE, S_OK};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, IBindCtx, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON,
    SHGFI_LARGEICON, SIIGBF_ICONONLY,
};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

// ---------------------------------------------------------------------------
// RAII ガード群 — 途中で Err/return しても解放が漏れないようにする
// ---------------------------------------------------------------------------

/// COM 初期化ガード。S_OK のときのみ対応する CoUninitialize を行う。
/// S_FALSE（同スレッドで既初期化）や RPC_E_CHANGED_MODE（別モードで既初期化）は
/// COM 自体は利用可能なので、そのまま続行しつつ Uninitialize はしない。
struct ComInit {
    need_uninit: bool,
}

impl ComInit {
    fn new() -> Self {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        ComInit {
            need_uninit: hr == S_OK,
        }
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        if self.need_uninit {
            unsafe { CoUninitialize() };
        }
    }
}

/// HICON を DestroyIcon で解放するガード。
struct IconGuard(HICON);

impl Drop for IconGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // 解放失敗はリカバリ不能なので無視（パニックさせない）
            let _ = unsafe { DestroyIcon(self.0) };
        }
    }
}

/// HBITMAP を DeleteObject で解放するガード。
struct BitmapGuard(HBITMAP);

impl Drop for BitmapGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            let _ = unsafe { DeleteObject(self.0.into()) };
        }
    }
}

/// 画面 DC（GetDC(None)）を ReleaseDC で解放するガード。
struct ScreenDc(HDC);

impl ScreenDc {
    fn get() -> Result<Self, String> {
        let hdc = unsafe { GetDC(None) };
        if hdc.is_invalid() {
            return Err("GetDC(None) が失敗しました".into());
        }
        Ok(ScreenDc(hdc))
    }
}

impl Drop for ScreenDc {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(None, self.0);
        }
    }
}

// ---------------------------------------------------------------------------
// 公開 API
// ---------------------------------------------------------------------------

/// アイコンを RGBA8 で返す（width, height, pixels）。
pub fn extract_icon_rgba(path: &str) -> Result<(u32, u32, Vec<u8>), String> {
    // SHGFI_USEFILEATTRIBUTES は使わない方針なので、実在しないパスは早期にエラーへ
    if !std::path::Path::new(path).exists() {
        return Err(format!("パスが存在しません: {}", path));
    }

    // どのスレッドから呼ばれても COM を使えるよう、関数内で初期化を保証する
    let _com = ComInit::new();

    // 第1候補: SHGetFileInfoW（.lnk はシェルがリンク解決込みでアイコンを返す）
    let first_err = match icon_via_shgetfileinfo(path) {
        Ok(v) => return Ok(v),
        Err(e) => e,
    };

    // 第2候補: IShellItemImageFactory::GetImage(SIIGBF_ICONONLY, 48x48)
    icon_via_image_factory(path).map_err(|e2| {
        format!(
            "アイコン抽出失敗: SHGetFileInfoW: {} / IShellItemImageFactory: {}",
            first_err, e2
        )
    })
}

/// 旧データ互換: PNG にエンコードして Base64 文字列（プレフィックス無しの生 Base64）で返す。
/// 旧版の iconBase64 は data URL ではなく生 Base64 の PNG。
pub fn extract_icon_png_base64(path: &str) -> Result<String, String> {
    let (w, h, rgba) = extract_icon_rgba(path)?;
    let img: image::RgbaImage =
        image::RgbaImage::from_raw(w, h, rgba).ok_or("RGBA バッファ長が不正")?;
    let mut png: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("PNG エンコード失敗: {}", e))?;
    use base64::Engine as _;
    Ok(base64::engine::general_purpose::STANDARD.encode(&png))
}

// ---------------------------------------------------------------------------
// 内部実装
// ---------------------------------------------------------------------------

/// &str → NUL 終端 UTF-16。
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 第1候補: SHGetFileInfoW で HICON を取得して RGBA 化。
fn icon_via_shgetfileinfo(path: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let wide = to_wide(path);
    let mut sfi = SHFILEINFOW::default();
    let ret = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut sfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if ret == 0 || sfi.hIcon.is_invalid() {
        return Err("SHGetFileInfoW がアイコンを返しませんでした".into());
    }
    // 以降どこで抜けても DestroyIcon される
    let _icon = IconGuard(sfi.hIcon);
    hicon_to_rgba(sfi.hIcon)
}

/// 第2候補: IShellItemImageFactory::GetImage(SIIGBF_ICONONLY, 48x48)。
fn icon_via_image_factory(path: &str) -> Result<(u32, u32, Vec<u8>), String> {
    let wide = to_wide(path);
    let factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None::<&IBindCtx>) }
            .map_err(|e| format!("SHCreateItemFromParsingName 失敗: {}", e))?;
    let hbm = unsafe { factory.GetImage(SIZE { cx: 48, cy: 48 }, SIIGBF_ICONONLY) }
        .map_err(|e| format!("GetImage 失敗: {}", e))?;
    let _bmp = BitmapGuard(hbm);

    let dc = ScreenDc::get()?;
    let (w, h) = bitmap_size(hbm)?;
    let mut bgra = get_dib_bgra32(dc.0, hbm, w, h)?;
    // GetImage の返す DIB は通常 α 付き 32bpp。マスクが無いので全 0 なら不透明扱いにする。
    if bgra.chunks_exact(4).all(|p| p[3] == 0) {
        for p in bgra.chunks_exact_mut(4) {
            p[3] = 255;
        }
    }
    bgra_to_rgba_inplace(&mut bgra);
    Ok((w, h, bgra))
}

/// HICON → RGBA8。カラービットマップが無いモノクロアイコンにも対応する。
fn hicon_to_rgba(hicon: HICON) -> Result<(u32, u32, Vec<u8>), String> {
    // ICONINFO は HBITMAP を含み Default 実装が無いため zeroed で初期化する
    let mut info: ICONINFO = unsafe { std::mem::zeroed() };
    unsafe { GetIconInfo(hicon, &mut info) }.map_err(|e| format!("GetIconInfo 失敗: {}", e))?;
    // GetIconInfo が返す HBITMAP は呼び出し側に解放義務がある
    let _color = BitmapGuard(info.hbmColor);
    let _mask = BitmapGuard(info.hbmMask);

    let dc = ScreenDc::get()?;

    if !info.hbmColor.is_invalid() {
        // カラーアイコン: 32bpp BGRA を取得
        let (w, h) = bitmap_size(info.hbmColor)?;
        let mut bgra = get_dib_bgra32(dc.0, info.hbmColor, w, h)?;

        // α が全 0 のアイコン（アルファ無しの古い形式）は hbmMask から α を再構成する。
        // AND マスクは黒(0)=不透明、白(≠0)=透明。
        if bgra.chunks_exact(4).all(|p| p[3] == 0) {
            if info.hbmMask.is_invalid() {
                // マスクも無ければ全ピクセル不透明とみなす
                for p in bgra.chunks_exact_mut(4) {
                    p[3] = 255;
                }
            } else {
                let mask = get_dib_bgra32(dc.0, info.hbmMask, w, h)?;
                for (px, m) in bgra.chunks_exact_mut(4).zip(mask.chunks_exact(4)) {
                    px[3] = if m[0] == 0 && m[1] == 0 && m[2] == 0 {
                        255
                    } else {
                        0
                    };
                }
            }
        }
        bgra_to_rgba_inplace(&mut bgra);
        Ok((w, h, bgra))
    } else {
        // モノクロアイコン: hbmMask が縦 2 倍で、上半分=AND マスク、下半分=XOR ビットマップ
        if info.hbmMask.is_invalid() {
            return Err("ICONINFO にカラーもマスクもありません".into());
        }
        let (w, h2) = bitmap_size(info.hbmMask)?;
        let h = h2 / 2;
        if h == 0 {
            return Err("モノクロアイコンのマスク高さが不正".into());
        }
        let bits = get_dib_bgra32(dc.0, info.hbmMask, w, h2)?;
        let stride = (w as usize) * 4;
        let (and_part, xor_part) = bits.split_at((h as usize) * stride);
        let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
        for (a, x) in and_part.chunks_exact(4).zip(xor_part.chunks_exact(4)) {
            let alpha = if a[0] == 0 && a[1] == 0 && a[2] == 0 {
                255
            } else {
                0
            };
            // BGRA → RGBA
            rgba.extend_from_slice(&[x[2], x[1], x[0], alpha]);
        }
        Ok((w, h, rgba))
    }
}

/// GetObjectW で HBITMAP の寸法を得る。高さは符号を除いた絶対値を返す。
fn bitmap_size(hbm: HBITMAP) -> Result<(u32, u32), String> {
    let mut bm = BITMAP::default();
    let got = unsafe {
        GetObjectW(
            hbm.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut BITMAP as *mut _),
        )
    };
    if got == 0 {
        return Err("GetObjectW 失敗".into());
    }
    let w = bm.bmWidth.max(0) as u32;
    let h = bm.bmHeight.unsigned_abs();
    if w == 0 || h == 0 {
        return Err("ビットマップサイズが 0 です".into());
    }
    Ok((w, h))
}

/// GetDIBits(BI_RGB, 32bpp, top-down) で BGRA バッファを取得する。
/// モノクロビットマップも GDI が 32bpp（黒=0x000000 / 白=0xFFFFFF）へ変換して返す。
fn get_dib_bgra32(hdc: HDC, hbm: HBITMAP, w: u32, h: u32) -> Result<Vec<u8>, String> {
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: -(h as i32), // 負の高さ = top-down（上から下へのスキャンライン順）
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
    let scanned = unsafe {
        GetDIBits(
            hdc,
            hbm,
            0,
            h,
            Some(buf.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };
    if scanned == 0 {
        return Err("GetDIBits 失敗".into());
    }
    Ok(buf)
}

/// BGRA → RGBA（B と R を入れ替え）。
fn bgra_to_rgba_inplace(buf: &mut [u8]) {
    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
}

// ---------------------------------------------------------------------------
// テスト（GUI 不要。実ファイルのアイコンをシェルから取得する）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 抽出結果の共通検証: サイズ > 0、バッファ長 = w*h*4、少なくとも 1 ピクセルは不透明。
    fn assert_valid_icon(path: &str) {
        let (w, h, px) =
            extract_icon_rgba(path).unwrap_or_else(|e| panic!("{} の抽出に失敗: {}", path, e));
        assert!(w > 0 && h > 0, "サイズが 0: {}x{}", w, h);
        assert_eq!(
            px.len(),
            (w as usize) * (h as usize) * 4,
            "バッファ長が w*h*4 と不一致"
        );
        assert!(
            px.chunks_exact(4).any(|p| p[3] > 0),
            "全ピクセルが透明（α 再構成の失敗の疑い）"
        );
    }

    #[test]
    fn exe_のアイコンを抽出できる() {
        assert_valid_icon("C:\\Windows\\notepad.exe");
    }

    #[test]
    fn フォルダのアイコンを抽出できる() {
        assert_valid_icon("C:\\Windows");
    }

    #[test]
    fn 存在しないパスはエラーになる() {
        let r = extract_icon_rgba("C:\\__rlaunch_no_such_dir__\\no_such_file.exe");
        assert!(r.is_err(), "存在しないパスで Ok が返った");
    }

    #[test]
    fn png_base64_は_png_マジックを含む() {
        let b64 =
            extract_icon_png_base64("C:\\Windows\\notepad.exe").expect("PNG Base64 生成に失敗");
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&b64)
            .expect("Base64 デコード失敗");
        assert!(
            bytes.starts_with(&[0x89, b'P', b'N', b'G']),
            "PNG マジックが先頭にない"
        );
    }

    #[test]
    fn 別スレッドから呼んでも安全() {
        // COM 未初期化のワーカースレッドからでも内部で初期化して動くこと
        let handle = std::thread::spawn(|| extract_icon_rgba("C:\\Windows\\notepad.exe"));
        let res = handle.join().expect("スレッドがパニックした");
        assert!(res.is_ok(), "別スレッドからの抽出に失敗: {:?}", res.err());
    }
}
