/**
 * 클립보드 문자열이 Git 원격 URL로 보이는지 보수적으로 판별합니다.
 * 확실하지 않으면 false (수동 입력 유도).
 */
export function looksLikeGitRemoteUrl(raw: string): boolean {
  const t = raw.trim();
  if (!t || /[\r\n]/.test(t)) return false;
  if (/^git@[^\s:]+:.+/i.test(t)) return true;
  if (/^ssh:\/\/[^\s]+/i.test(t)) return true;
  if (!/^https?:\/\//i.test(t)) return false;
  if (/\.git(?:\/|$|\?|#)/i.test(t)) return true;
  try {
    const u = new URL(t);
    const host = u.hostname.toLowerCase();
    const segs = u.pathname.split("/").filter(Boolean);
    if (segs.length < 2) return false;
    return (
      host === "github.com" ||
      host.endsWith(".github.com") ||
      host === "gitlab.com" ||
      host.endsWith(".gitlab.com") ||
      host === "bitbucket.org" ||
      host.endsWith(".bitbucket.org")
    );
  } catch {
    return false;
  }
}
