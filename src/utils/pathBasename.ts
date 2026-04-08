/** 경로에서 마지막 구성 요소(파일·폴더 이름)만 반환합니다. */
export function pathBasename(path: string): string {
  const normalized = path.replace(/\\/g, "/").replace(/\/+$/, "");
  const i = normalized.lastIndexOf("/");
  return i >= 0 ? normalized.slice(i + 1) : normalized;
}

/** 실행 파일 경로에서 표시용 짧은 이름(일부 확장자 제거). */
export function executableDisplayName(path: string): string {
  const base = pathBasename(path);
  return base.replace(/\.(exe|cmd|bat|msi|AppImage)$/i, "").replace(/\.app$/i, "") || base;
}
