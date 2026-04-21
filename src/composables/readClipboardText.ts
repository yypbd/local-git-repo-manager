import { readText as readTextTauri } from "@tauri-apps/plugin-clipboard-manager";

/**
 * Tauri 네이티브 클립보드 우선(WebKit `navigator.clipboard` Paste 시트 회피).
 * 브라우저 단독 미리보기 시에는 `navigator.clipboard`로 폴백합니다.
 */
export async function readClipboardText(): Promise<string> {
  try {
    return await readTextTauri();
  } catch {
    try {
      return await navigator.clipboard.readText();
    } catch {
      return "";
    }
  }
}
