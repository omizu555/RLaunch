//! Win32 プラットフォーム層。
//! ここは UI(iced) 非依存に保つ — 戻り値は std 型のみ（iced 型を持ち込まない）。

pub mod desktop_hook;
pub mod icon;
pub mod launch;
pub mod lnk;
pub mod monitor;
pub mod single;
