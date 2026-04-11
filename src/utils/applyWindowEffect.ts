/* ============================================================
   applyWindowEffect - テーマの透過効果をウィンドウに適用
   --window-opacity CSS 変数と set_window_effect コマンドを共通化。

   useChildTheme / SettingsWindow / WidgetSelectWindow /
   WidgetSettingsWindow で共通利用。
   ============================================================ */
import { invoke } from "@tauri-apps/api/core";
import type { ThemeInfo } from "./themeLoader";

/**
 * テーマ一覧と themeId から、現在のウィンドウに透過効果を適用する。
 * CSS 変数 --window-opacity を設定し、Rust 側の set_window_effect を呼ぶ。
 */
export function applyWindowEffect(themes: ThemeInfo[], themeId: string): void {
  const theme = themes.find((t) => t.id === themeId);
  const opacity = theme?.variables["--window-opacity"] ?? "1";
  document.documentElement.style.setProperty("--window-opacity", opacity);
  const effect = theme?.variables["--window-effect"] ?? "none";
  invoke("set_window_effect", { effect }).catch((e) => console.warn("Failed to apply window effect:", e));
}
