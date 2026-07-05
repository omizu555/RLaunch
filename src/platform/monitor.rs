//! カーソル位置とモニター情報（物理座標）。
//! iced の論理座標への変換は呼び出し側（scale factor を掛ける）。

use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

/// カーソルのあるモニターの作業領域（タスクバー除く、物理px）とカーソル位置
#[derive(Debug, Clone, Copy)]
pub struct CursorMonitor {
    pub cursor_x: i32,
    pub cursor_y: i32,
    /// 作業領域 (rcWork)
    pub work_x: i32,
    pub work_y: i32,
    pub work_w: i32,
    pub work_h: i32,
}

/// GetCursorPos の物理スクリーン座標
pub fn cursor_pos() -> Option<(i32, i32)> {
    let mut pt = POINT::default();
    // windows 0.62 では GetCursorPos は Result<()> を返す
    // （失敗はデスクトップにアクセスできない場合など稀）
    unsafe { GetCursorPos(&mut pt) }.ok()?;
    Some((pt.x, pt.y))
}

/// GetCursorPos + MonitorFromPoint(MONITOR_DEFAULTTONEAREST) + GetMonitorInfoW
pub fn cursor_monitor() -> Option<CursorMonitor> {
    let (cursor_x, cursor_y) = cursor_pos()?;
    let pt = POINT {
        x: cursor_x,
        y: cursor_y,
    };

    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let ok = unsafe {
        // MONITOR_DEFAULTTONEAREST なので必ず有効なモニターハンドルが返る
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        GetMonitorInfoW(hmon, &mut mi)
    };
    if !ok.as_bool() {
        return None;
    }

    // rcWork = タスクバー等を除いた作業領域（物理座標）
    let work = mi.rcWork;
    Some(CursorMonitor {
        cursor_x,
        cursor_y,
        work_x: work.left,
        work_y: work.top,
        work_w: work.right - work.left,
        work_h: work.bottom - work.top,
    })
}

/// 矩形 (x,y,w,h) をモニター作業領域内にクランプした左上座標を返す（物理px）
pub fn clamp_to_work_area(m: &CursorMonitor, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    let max_x = m.work_x + m.work_w - w;
    let max_y = m.work_y + m.work_h - h;
    (
        x.clamp(m.work_x, max_x.max(m.work_x)),
        y.clamp(m.work_y, max_y.max(m.work_y)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のモニター: 作業領域 (100, 50) 起点で 1920x1040
    fn mon() -> CursorMonitor {
        CursorMonitor {
            cursor_x: 500,
            cursor_y: 500,
            work_x: 100,
            work_y: 50,
            work_w: 1920,
            work_h: 1040,
        }
    }

    #[test]
    fn clamp_収まる場合はそのまま() {
        let m = mon();
        assert_eq!(clamp_to_work_area(&m, 200, 200, 400, 300), (200, 200));
    }

    #[test]
    fn clamp_左上にはみ出したら作業領域の左上へ() {
        let m = mon();
        assert_eq!(clamp_to_work_area(&m, -1000, -1000, 400, 300), (100, 50));
        // 境界ちょうど
        assert_eq!(clamp_to_work_area(&m, 100, 50, 400, 300), (100, 50));
    }

    #[test]
    fn clamp_右下にはみ出したら右下端に張り付く() {
        let m = mon();
        // 右端: work_x + work_w - w = 100 + 1920 - 400 = 1620
        // 下端: work_y + work_h - h = 50 + 1040 - 300 = 790
        assert_eq!(clamp_to_work_area(&m, 9999, 9999, 400, 300), (1620, 790));
        // 境界ちょうど（1ピクセルもはみ出さない）
        assert_eq!(clamp_to_work_area(&m, 1620, 790, 400, 300), (1620, 790));
        assert_eq!(clamp_to_work_area(&m, 1621, 791, 400, 300), (1620, 790));
    }

    #[test]
    fn clamp_ウィンドウが作業領域より大きい場合は左上を優先() {
        let m = mon();
        // max_x < work_x になっても panic せず左上に寄せる
        assert_eq!(clamp_to_work_area(&m, 500, 500, 3000, 2000), (100, 50));
    }

    #[test]
    fn cursor_pos_は取得できる() {
        // 対話デスクトップがあれば必ず成功する
        let pos = cursor_pos();
        assert!(pos.is_some(), "GetCursorPos が失敗した");
    }

    #[test]
    fn cursor_monitor_は妥当な作業領域を返す() {
        let m = cursor_monitor().expect("cursor_monitor が None を返した");
        // 作業領域のサイズは正であるはず
        assert!(m.work_w > 0, "work_w = {}", m.work_w);
        assert!(m.work_h > 0, "work_h = {}", m.work_h);
        // カーソルは cursor_pos と同じソースなので座標が一貫している
        // （移動中の可能性があるため厳密一致は確認しない）
    }
}
