/* ============================================================
   themeLoader - 動的テーマ読み込み・適用ユーティリティ
   Rust の list_themes / init_themes と連携し、
   CSS 変数を :root に動的に適用する
   ============================================================ */
import { invoke } from "@tauri-apps/api/core";

/** テーマ情報（Rust の ThemeInfo と同じ構造） */
export interface ThemeInfo {
  id: string;
  label: string;
  author: string;
  variables: Record<string, string>;
}

/** テーマ一覧を取得 */
export async function listThemes(): Promise<ThemeInfo[]> {
  return await invoke<ThemeInfo[]>("list_themes");
}

/** テーマフォルダのパスを取得 */
export async function getThemesDirPath(): Promise<string> {
  return await invoke<string>("get_themes_dir_path");
}

/** サンプルテーマフォルダのパスを取得 */
export async function getSampleThemesDirPath(): Promise<string> {
  return await invoke<string>("get_sample_themes_dir_path");
}

/**
 * テーマの CSS 変数を :root に動的適用
 * 前回適用したカスタム変数をクリアしてから新しいものを設定する
 */
const THEME_VAR_ATTR = "data-theme-vars";

function applyThemeVariables(theme: ThemeInfo): void {
  const root = document.documentElement;

  // 前回のカスタムテーマ変数をクリア
  const prevVars = root.getAttribute(THEME_VAR_ATTR);
  if (prevVars) {
    for (const varName of prevVars.split(",")) {
      root.style.removeProperty(varName);
    }
  }

  // 新しいテーマ変数を適用
  const varNames: string[] = [];
  for (const [key, value] of Object.entries(theme.variables)) {
    root.style.setProperty(key, value);
    varNames.push(key);
  }

  // 適用した変数名を記録（次回クリア用）
  root.setAttribute(THEME_VAR_ATTR, varNames.join(","));
}

/**
 * テーマ ID からテーマをロードして適用
 * テーマ一覧キャッシュから検索し、見つかったら CSS 変数を適用
 */
export function applyThemeById(themes: ThemeInfo[], themeId: string): void {
  const theme = themes.find((t) => t.id === themeId);
  if (theme) {
    applyThemeVariables(theme);
  }
}
