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

/** 경로별 `folder_root_row` 결과 캐시(프로젝트 전환·재방문 시 재호출 감소). `reload`(강제 새로고침) 시 해당 경로 키 무효화. */
const folderRowCache = new Map<string, FolderRootRow>();

export function invalidateFolderRowCache(): void {
  folderRowCache.clear();
}

async function fetchFolderRow(p: string): Promise<FolderRootRow> {
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

async function loadRow(p: string, skipCache: boolean): Promise<FolderRootRow> {
  const nk = normalizePathKey(p);
  if (!skipCache) {
    const hit = folderRowCache.get(nk);
    if (hit) return hit;
  }
  const row = await fetchFolderRow(p);
  folderRowCache.set(nk, row);
  return row;
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

/** 화면 상단에 먼저 보이는 폴더부터 채우기 위한 우선 구간 길이 */
const PRIORITY_FOLDER_COUNT = 24;

/** 프로젝트 루트 경로별 원격·브랜치·상태 (폴더 목록 + 하단 상세 공용) */
export function useFolderRootRows(paths: Ref<string[]>) {
  const rows = ref<Record<string, FolderRootRow>>({});
  const loading = ref(false);
  const loadToken = ref(0);
  const loadedCount = ref(0);
  const totalCount = ref(0);
  const MAX_CONCURRENCY = 8;

  const load = async (options?: { forceRefresh?: boolean }) => {
    const forceRefresh = options?.forceRefresh ?? false;
    const token = ++loadToken.value;
    const list = paths.value;
    const sorted = sortRootPathsForDisplay(list);
    loading.value = true;
    loadedCount.value = 0;
    totalCount.value = sorted.length;

    if (sorted.length === 0) {
      rows.value = {};
      loading.value = false;
      return;
    }

    if (forceRefresh) {
      rows.value = {};
      for (const p of sorted) {
        folderRowCache.delete(normalizePathKey(p));
      }
    } else {
      const initial: Record<string, FolderRootRow> = {};
      for (const p of sorted) {
        const hit = folderRowCache.get(normalizePathKey(p));
        if (hit) {
          for (const key of pathKeyVariants(p)) {
            initial[key] = hit;
          }
        }
      }
      rows.value = initial;
    }

    const skipCache = forceRefresh;

    const mergeOne = (p: string, row: FolderRootRow) => {
      const bag = rows.value;
      for (const key of pathKeyVariants(p)) {
        bag[key] = row;
      }
    };

    /**
     * `folder_root_row`는 워커 풀로 병렬 호출하고, **화면에 반영하는 병합만** 큐 + rAF로
     * **프레임당 최대 1행**씩 처리합니다. (그렇지 않으면 Vue/브라우저가 한 프레임에 몰아 그립니다.)
     */
    const processPathsWithPool = async (pathList: string[]) => {
      const mergeQueue: { p: string; row: FolderRootRow }[] = [];
      let mergeRafPending = false;

      const runMergeFrame = () => {
        const item = mergeQueue.shift();
        if (!item) {
          mergeRafPending = false;
          return;
        }
        if (token !== loadToken.value) {
          mergeQueue.length = 0;
          mergeRafPending = false;
          return;
        }
        mergeOne(item.p, item.row);
        loadedCount.value += 1;
        if (mergeQueue.length > 0) {
          requestAnimationFrame(runMergeFrame);
        } else {
          mergeRafPending = false;
        }
      };

      const enqueueMerge = (p: string, row: FolderRootRow) => {
        mergeQueue.push({ p, row });
        if (!mergeRafPending) {
          mergeRafPending = true;
          requestAnimationFrame(runMergeFrame);
        }
      };

      let next = 0;
      const worker = async () => {
        while (true) {
          const cur = next++;
          if (cur >= pathList.length) return;
          const p = pathList[cur]!;
          const row = await loadRow(p, skipCache);
          if (token !== loadToken.value) return;
          enqueueMerge(p, row);
        }
      };

      await Promise.all(Array.from({ length: MAX_CONCURRENCY }, () => worker()));

      while (mergeQueue.length > 0 || mergeRafPending) {
        await new Promise<void>((resolve) => {
          requestAnimationFrame(() => resolve());
        });
      }
    };

    try {
      const n = sorted.length;
      const split = Math.min(PRIORITY_FOLDER_COUNT, n);
      const priorityPaths = sorted.slice(0, split);
      const restPaths = sorted.slice(split);
      await processPathsWithPool(priorityPaths);
      if (token !== loadToken.value) return;
      await processPathsWithPool(restPaths);
    } finally {
      if (token === loadToken.value) {
        loading.value = false;
      }
    }
  };

  watch(
    paths,
    () => {
      void load({ forceRefresh: false });
    },
    { immediate: true, deep: true },
  );

  return {
    rows,
    loading,
    loadedCount,
    totalCount,
    reload: () => load({ forceRefresh: true }),
  };
}
