/** 로컬 파일 경로의 부모 디렉터리 (빈 문자열이면 판별 불가·루트만 있는 경우 등). */
export function parentDirectory(filePath: string): string {
  const n = filePath.trim().replace(/[/\\]+$/, "");
  if (!n) return "";
  const i = Math.max(n.lastIndexOf("/"), n.lastIndexOf("\\"));
  if (i <= 0) return "";
  return n.slice(0, i);
}
