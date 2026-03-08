/* ============================================================
   useChildTheme - 子ウィンドウで現在のテーマを自動適用するフック
   Store から現在の themeId を読み取り、listThemes() で取得した
   テーマ一覧から CSS 変数を :root に適用する。
   ============================================================ */
import { useEffect, useCallback, useRef } from "react";
import { listThemes, applyThemeById } from "../utils/themeLoader";
import { applyWindowEffect } from "../utils/applyWindowEffect";
import { getSettings } from "../stores/launcherStore";
import type { ThemeInfo } from "../utils/themeLoader";

/**
 * 子ウィンドウマウント時に現在のテーマを適用する。
 * reusable ウィンドウ向けに refreshTheme() も返す。
 */
export function useChildTheme(): { refreshTheme: () => void } {
  const themesRef = useRef<ThemeInfo[]>([]);

  const refreshTheme = useCallback(async () => {
    try {
      let themes = themesRef.current;
      if (themes.length === 0) {
        themes = await listThemes();
        themesRef.current = themes;
      }
      const settings = await getSettings();
      if (themes.length > 0) {
        applyThemeById(themes, settings.theme);
        applyWindowEffect(themes, settings.theme);
      }
    } catch {
      // テーマ取得失敗時はデフォルト CSS 変数のまま
    }
  }, []);

  useEffect(() => {
    refreshTheme();
  }, [refreshTheme]);

  return { refreshTheme };
}
