import { invoke } from "@tauri-apps/api/core";
import type { Ref } from "vue";
import { ref, watch } from "vue";
import { repoPathArgs } from "@/utils/tauriRepoPath";

/**
 * `rows` Record 키로 쓸 때 경로 문자열을 통일합니다.
 * macOS(APFS)에서 동일 폴더가 NFC/NFD 등으로만 달라 조회가 실패하는 경우를 줄입니다.
 */
export function normalizePathKey(p: string): string {
  const t = p.trim().replace(/[/\\]+$/, "");
  try {
    return t.normalize("NFC");
  } catch {
    return t;
  }
}

/** 동일 폴더를 가리키는 가능한 Record 키 문자열(원문·trim·NFC/NFD). */
function pathKeyVariants(p: string): string[] {
  const t = p.trim().replace(/[/\\]+$/, "");
  const out = new Set<string>();
  for (const x of [p, t, normalizePathKey(t)]) {
    if (x.length > 0) out.add(x);
  }
  try {
    out.add(t.normalize("NFC"));
    out.add(t.normalize("NFD"));
  } catch {
    /* ignore */
  }
  return [...out];
}

/** 목록·선택에 쓰인 경로로 `rows`에서 행을 찾습니다. */
export function lookupFolderRow(
  rows: Record<string, FolderRootRow>,
  path: string,
): FolderRootRow | undefined {
  for (const key of pathKeyVariants(path)) {
    const row = rows[key];
    if (row) return row;
  }
  return undefined;
}

export type FolderRootRow = {
  /** 이 경로에 `.git`이 직접 있음(상위 저장소 작업 트리와 무관) */
  hasDotGit: boolean;
  remote: string | null;
  remoteCount: number;
  branch: string;
  clean: boolean;
  /** 추적 파일 변경·스테이징 등( porcelain에서 `??` 제외) */
  trackedChanges: number;
  /** 미추적(`??`) 항목 수 */
  untrackedFiles: number;
  gitError: boolean;
};

type FolderRootRowPayload = {
  hasDotGit: boolean;
  remote: string | null;
  remoteCount: number;
  branch: string;
  clean: boolean;
  trackedChanges: number;
  untrackedFiles: number;
  gitError: boolean;
};

async function loadRow(p: string): Promise<FolderRootRow> {
  try {
    const r = await invoke<FolderRootRowPayload>("folder_root_row", repoPathArgs(p));
    return {
      hasDotGit: r.hasDotGit,
      remote: r.remote ?? null,
      remoteCount: r.remoteCount,
      branch: r.branch,
      clean: r.clean,
      trackedChanges: r.trackedChanges,
      untrackedFiles: r.untrackedFiles,
      gitError: r.gitError,
    };
  } catch {
    return {
      hasDotGit: false,
      remote: null,
      remoteCount: 0,
      branch: "—",
      clean: true,
      trackedChanges: 0,
      untrackedFiles: 0,
      gitError: true,
    };
  }
}

export function folderNameFromPath(path: string): string {
  const trimmed = path.replace(/[/\\]+$/, "");
  if (!trimmed) return path;
  const parts = trimmed.split(/[/\\]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1]! : path;
}

/** `RootPathsList` 표시·Shift 범위 선택과 동일한 순서 */
export function sortRootPathsForDisplay(paths: string[]): string[] {
  return [...paths].sort((a, b) => {
    const na = folderNameFromPath(a);
    const nb = folderNameFromPath(b);
    const byName = na.localeCompare(nb, undefined, {
      sensitivity: "base",
      numeric: true,
    });
    if (byName !== 0) return byName;
    return a.localeCompare(b, undefined, { sensitivity: "base", numeric: true });
  });
}

/** 프로젝트 루트 경로별 원격·브랜치·상태 (폴더 목록 + 하단 상세 공용) */
export function useFolderRootRows(paths: Ref<string[]>) {
  const rows = ref<Record<string, FolderRootRow>>({});
  const loading = ref(false);

  const load = async () => {
    const list = paths.value;
    loading.value = true;
    const next: Record<string, FolderRootRow> = {};
    try {
      const results = await Promise.all(
        list.map(async (p) => {
          const row = await loadRow(p);
          return [p, row] as const;
        }),
      );
      for (const [p, row] of results) {
        for (const key of pathKeyVariants(p)) {
          next[key] = row;
        }
      }
      rows.value = next;
    } finally {
      loading.value = false;
    }
  };

  watch(
    paths,
    () => {
      void load();
    },
    { immediate: true, deep: true },
  );

  return { rows, loading, reload: load };
}
