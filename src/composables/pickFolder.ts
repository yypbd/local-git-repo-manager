import { open } from "@tauri-apps/plugin-dialog";

/** Tauri 네이티브 폴더 선택(단일). 브라우저 단독 실행 시 null. */
export async function pickDirectory(): Promise<string | null> {
  try {
    const selected = await open({
      multiple: false,
      directory: true,
    });
    if (selected === null) return null;
    if (Array.isArray(selected)) return selected[0] ?? null;
    return selected;
  } catch {
    return null;
  }
}

/** Tauri 네이티브 폴더 선택(복수). 취소 시 빈 배열. */
export async function pickDirectories(): Promise<string[]> {
  try {
    const selected = await open({
      multiple: true,
      directory: true,
    });
    if (selected === null) return [];
    if (Array.isArray(selected)) return selected.filter((p): p is string => Boolean(p?.trim()));
    return selected.trim() ? [selected] : [];
  } catch {
    return [];
  }
}

/** 단일 파일 선택(실행 파일 등). 취소 시 null. */
export async function pickExecutableFile(): Promise<string | null> {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      /** 기본 `copy`는 샌드박스로 복사되어 실행 비트가 깨지거나 경로가 달라질 수 있음(macOS 등). */
      fileAccessMode: "scoped",
    });
    if (selected === null) return null;
    if (Array.isArray(selected)) return selected[0]?.trim() || null;
    return selected.trim() || null;
  } catch {
    return null;
  }
}
