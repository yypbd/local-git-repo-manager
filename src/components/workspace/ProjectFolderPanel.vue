<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { AlertDialogRoot } from "radix-vue";
import { DialogRoot } from "radix-vue";
import FolderDropZone from "@/components/FolderDropZone.vue";
import FolderSelectionDetail from "@/components/workspace/FolderSelectionDetail.vue";
import RootPathsList from "@/components/workspace/RootPathsList.vue";
import MoveRootToProjectModal from "@/components/tree/MoveRootToProjectModal.vue";
import RemoteManagerDialog from "@/components/git/RemoteManagerDialog.vue";
import ArchiveDialog from "@/components/git/ArchiveDialog.vue";
import GitignoreEditor from "@/components/git/GitignoreEditor.vue";
import TemplatePicker from "@/components/git/TemplatePicker.vue";
import Select from "@/components/ui/Select.vue";
import Button from "@/components/ui/Button.vue";
import AlertDialogContent from "@/components/ui/AlertDialogContent.vue";
import AlertDialogHeader from "@/components/ui/AlertDialogHeader.vue";
import AlertDialogTitle from "@/components/ui/AlertDialogTitle.vue";
import AlertDialogDescription from "@/components/ui/AlertDialogDescription.vue";
import AlertDialogFooter from "@/components/ui/AlertDialogFooter.vue";
import AlertDialogCancel from "@/components/ui/AlertDialogCancel.vue";
import AlertDialogAction from "@/components/ui/AlertDialogAction.vue";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import { pickDirectories } from "@/composables/pickFolder";
import {
  folderNameFromPath,
  lookupFolderRow,
  type FolderRootRow,
} from "@/composables/useFolderRootRows";
import { useProjectsStore } from "@/stores/projects";
import { useToastStore } from "@/stores/toast";
import type { Project } from "@/stores/projects";
import { repoPathArgs } from "@/utils/tauriRepoPath";

const props = withDefaults(
  defineProps<{
    project: Project | undefined;
    rows: Record<string, FolderRootRow>;
    loading: boolean;
    /** `useFolderRootRows` 진행률(이번 로드에서 처리한 폴더 수 / 전체) */
    folderRowsLoaded?: number;
    folderRowsTotal?: number;
    reload: () => void | Promise<void>;
  }>(),
  {
    folderRowsLoaded: 0,
    folderRowsTotal: 0,
  },
);

const emit = defineEmits<{
  dropped: [paths: string[]];
}>();

const selectedFolderPath = defineModel<string | null>("selectedFolderPath", { default: null });
const selectedFolderPaths = defineModel<string[]>("selectedFolderPaths", {
  default: () => [],
});

const { projects, syncFromBackend } = useProjectsStore();
const { push: toast } = useToastStore();
const { t } = useI18n();
type ExternalTool = { id: string; label: string; command: string; argsTemplate: string };
const externalTools = ref<ExternalTool[]>([]);
const showExternalPicker = ref(false);
const platformName = (typeof navigator !== "undefined" ? navigator.userAgent : "").toLowerCase();
const revealLabelKey = computed(() => {
  if (platformName.includes("mac")) return "workspace.openInFinder";
  if (platformName.includes("win")) return "workspace.openInExplorer";
  return "workspace.openInFileManager";
});

const rootPathsRef = computed(() => props.project?.rootPaths ?? []);
const rows = computed(() => props.rows);
const loading = computed(() => props.loading);
const reload = () => props.reload();
const confirmRemove = ref(false);
const confirmGitRemove = ref(false);
const moveModalOpen = ref(false);

const hasOtherProjects = computed(() => {
  if (!props.project) return false;
  return projects.value.some((p) => p.id !== props.project?.id);
});

/** Shift 범위 선택 앵커 */
const selectionAnchorPath = ref<string | null>(null);
type FolderFilterMode = "all" | "remote" | "git" | "non_git";
const filterMode = ref<FolderFilterMode>("all");
type FolderViewMode = "list" | "icon";
const FOLDER_VIEW_MODE_KEY = "workspace.folderViewMode";
const folderViewMode = ref<FolderViewMode>("list");

try {
  const stored = localStorage.getItem(FOLDER_VIEW_MODE_KEY);
  if (stored === "list" || stored === "icon") {
    folderViewMode.value = stored;
  }
} catch {}

type FolderSortMode = "name" | "path" | "status";
type FolderSortDir = "asc" | "desc";
const FOLDER_SORT_MODE_KEY = "workspace.folderSortMode";
const FOLDER_SORT_DIR_KEY = "workspace.folderSortDir";
const sortMode = ref<FolderSortMode>("name");
const sortDir = ref<FolderSortDir>("asc");

try {
  const sm = localStorage.getItem(FOLDER_SORT_MODE_KEY);
  if (sm === "name" || sm === "path" || sm === "status") sortMode.value = sm;
  const sd = localStorage.getItem(FOLDER_SORT_DIR_KEY);
  if (sd === "asc" || sd === "desc") sortDir.value = sd;
} catch {}

watch(sortMode, (v) => { try { localStorage.setItem(FOLDER_SORT_MODE_KEY, v); } catch {} });
watch(sortDir, (v) => { try { localStorage.setItem(FOLDER_SORT_DIR_KEY, v); } catch {} });

const filteredRootPaths = computed(() => {
  const all = props.project?.rootPaths ?? [];
  if (filterMode.value === "all") return all;
  return all.filter((p) => {
    const row = lookupFolderRow(props.rows, p);
    // 로딩 중/미조회 항목은 제외하지 않아 목록이 갑자기 비지 않게 유지
    if (!row) return true;
    if (filterMode.value === "remote") return Boolean(row.remote?.trim());
    if (filterMode.value === "git") return !row.gitError;
    if (filterMode.value === "non_git") return row.gitError;
    return true;
  });
});

const sortedFilteredPaths = computed(() => {
  const paths = [...filteredRootPaths.value];
  const mode = sortMode.value;
  const dir = sortDir.value;
  paths.sort((a, b) => {
    let cmp = 0;
    if (mode === "name") {
      const na = folderNameFromPath(a);
      const nb = folderNameFromPath(b);
      cmp = na.localeCompare(nb, undefined, { sensitivity: "base", numeric: true });
      if (cmp === 0) cmp = a.localeCompare(b, undefined, { sensitivity: "base", numeric: true });
    } else if (mode === "path") {
      cmp = a.localeCompare(b, undefined, { sensitivity: "base", numeric: true });
    } else {
      // status: clean(0) < dirty(1) < non-git(2)
      const statusRank = (p: string) => {
        const row = lookupFolderRow(props.rows, p);
        if (!row || row.gitError) return 2;
        if (!row.clean) return 1;
        return 0;
      };
      cmp = statusRank(a) - statusRank(b);
      if (cmp === 0) {
        const na = folderNameFromPath(a);
        const nb = folderNameFromPath(b);
        cmp = na.localeCompare(nb, undefined, { sensitivity: "base", numeric: true });
      }
    }
    return dir === "asc" ? cmp : -cmp;
  });
  return paths;
});

watch(filteredRootPaths, (visiblePaths) => {
  // 필터로 현재 선택이 목록에서 사라지면 선택 상태를 정리
  if (selectedFolderPath.value && !visiblePaths.includes(selectedFolderPath.value)) {
    selectedFolderPath.value = null;
  }
  selectedFolderPaths.value = selectedFolderPaths.value.filter((p) => visiblePaths.includes(p));
  if (selectedFolderPaths.value.length === 0) {
    selectionAnchorPath.value = null;
  }
});

watch(rootPathsRef, (paths) => {
  selectedFolderPaths.value = selectedFolderPaths.value.filter((p) => paths.includes(p));
  if (selectedFolderPath.value && !paths.includes(selectedFolderPath.value)) {
    selectedFolderPath.value = null;
    selectionAnchorPath.value = null;
  }
});

watch(folderViewMode, (next) => {
  try {
    localStorage.setItem(FOLDER_VIEW_MODE_KEY, next);
  } catch {}
});

watch(
  () => props.project?.id,
  () => {
    selectedFolderPath.value = null;
    selectedFolderPaths.value = [];
    selectionAnchorPath.value = null;
  },
);

function onRootRowClick(path: string, e: MouseEvent) {
  if (!props.project) return;
  const sorted = sortedFilteredPaths.value;
  const ctrl = e.ctrlKey || e.metaKey;
  const shift = e.shiftKey;

  if (!ctrl && !shift) {
    selectedFolderPaths.value = [path];
    selectedFolderPath.value = path;
    selectionAnchorPath.value = path;
    return;
  }
  if (ctrl) {
    const set = new Set(selectedFolderPaths.value);
    if (set.has(path)) set.delete(path);
    else set.add(path);
    selectedFolderPaths.value = sorted.filter((p) => set.has(p));
    selectedFolderPath.value = path;
    selectionAnchorPath.value = path;
    return;
  }
  if (shift) {
    const anchor = selectionAnchorPath.value ?? selectedFolderPath.value ?? path;
    const ia = sorted.indexOf(anchor);
    const ib = sorted.indexOf(path);
    if (ia < 0 || ib < 0) return;
    const lo = Math.min(ia, ib);
    const hi = Math.max(ia, ib);
    selectedFolderPaths.value = sorted.slice(lo, hi + 1);
    selectedFolderPath.value = path;
  }
}

const canRemove = computed(() => Boolean(props.project && selectedFolderPaths.value.length > 0));

const singleSelectedPath = computed(() =>
  selectedFolderPaths.value.length === 1 ? selectedFolderPaths.value[0]! : null,
);
const canPathActions = computed(() => Boolean(singleSelectedPath.value));

const loadExternalTools = async () => {
  try {
    const s = await invoke<{ externalTools?: ExternalTool[] }>("get_settings");
    externalTools.value = s.externalTools ?? [];
  } catch {
    externalTools.value = [];
  }
};

const usableExternalTools = computed(() => externalTools.value.filter((x) => x.command?.trim()));

const runExternalWithTool = async (tool: ExternalTool) => {
  const p = singleSelectedPath.value;
  if (!p) return;
  const cmd = tool.command.trim();
  if (!cmd) {
    toast(t("workspace.externalOpenBadTool"), "error");
    return;
  }
  try {
    const tpl = tool.argsTemplate?.trim();
    await invoke("open_external", {
      path: p,
      command: cmd,
      argsTemplate: tpl ? tpl : null,
    });
    showExternalPicker.value = false;
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const openExternal = async () => {
  const p = singleSelectedPath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  await loadExternalTools();
  const tools = usableExternalTools.value;
  if (tools.length === 0) {
    toast(t("workspace.externalOpenNoTools"), "error");
    return;
  }
  if (tools.length === 1) {
    await runExternalWithTool(tools[0]!);
    return;
  }
  showExternalPicker.value = true;
};

const copyPathInvoke = async () => {
  const p = singleSelectedPath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  try {
    await navigator.clipboard.writeText(p);
    toast(t("workspace.copyDone"), "success");
  } catch {
    toast(t("workspace.clipboardCopyFailed"), "error");
  }
};

const copyRemote = async () => {
  const row = selectedRow.value;
  const remote = row?.remote?.trim();
  if (!remote || row?.gitError) {
    toast(t("workspace.copyRemoteUnavailable"), "error");
    return;
  }
  try {
    await navigator.clipboard.writeText(remote);
    toast(t("workspace.copyRemoteSuccess"), "success");
  } catch {
    toast(t("workspace.clipboardCopyFailed"), "error");
  }
};

const openRemoteInBrowser = async () => {
  const row = selectedRow.value;
  const remote = row?.remote?.trim();
  if (!remote || row?.gitError) {
    toast(t("workspace.copyRemoteUnavailable"), "error");
    return;
  }
  try {
    await invoke("open_remote_in_browser", { remote });
  } catch (e) {
    const detail = e instanceof Error ? e.message : String(e);
    toast(t("workspace.openRemoteInBrowserFailed", { detail }), "error");
  }
};

const revealPath = async () => {
  const p = singleSelectedPath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  await invoke("reveal_path", { path: p });
};

const addFolder = async () => {
  const paths = await pickDirectories();
  if (paths.length) emit("dropped", paths);
};

const removeSelected = () => {
  if (!canRemove.value) return;
  confirmRemove.value = true;
};

const pathsToRemove = computed(() =>
  selectedFolderPaths.value.length > 0 ? [...selectedFolderPaths.value] : [],
);

const confirmRemoveDo = async () => {
  if (!props.project || pathsToRemove.value.length === 0) return;
  confirmRemove.value = false;
  const pid = props.project.id;
  for (const path of pathsToRemove.value) {
    await invoke("projects_remove_root", {
      projectId: pid,
      path,
    });
  }
  selectedFolderPath.value = null;
  selectedFolderPaths.value = [];
  selectionAnchorPath.value = null;
  await syncFromBackend();
};

const selectedRow = computed(() => {
  const sp = selectedFolderPath.value;
  if (!sp) return undefined;
  return lookupFolderRow(rows.value, sp);
});

/** 이 경로에 `.git`이 없을 때만 git init (상위만 Git이어도 init 가능) */
const canGitInit = computed(() => {
  if (!singleSelectedPath.value || selectedFolderPaths.value.length !== 1) return false;
  // 목록(rows) 캐시는 선택 직후 불일치할 수 있으니,
  // 선택된 경로에 대해 별도 재검증한 상태를 우선 사용한다.
  if (selectedHasDotGit.value === null) return true; // 확인 전이면 일단 활성(클릭 시점에 서버 재검증)
  return !selectedHasDotGit.value;
});

/** 이 경로에 `.git`이 있을 때만 삭제 */
const canGitRemoveRepo = computed(() => {
  if (!singleSelectedPath.value || selectedFolderPaths.value.length !== 1) return false;
  if (selectedHasDotGit.value === null) return false;
  return selectedHasDotGit.value;
});

// 선택 경로에 대해 `.git` 존재 여부를 즉시 재검증 (버튼 활성 판단 정확도 향상)
const selectedHasDotGit = ref<boolean | null>(null);
const selectedHasDotGitLoading = ref(false);

watch(
  selectedFolderPath,
  async (p) => {
    if (!p) {
      selectedHasDotGit.value = null;
      return;
    }
    selectedHasDotGitLoading.value = true;
    try {
      selectedHasDotGit.value = await invoke<boolean>("path_is_git_repo_root", repoPathArgs(p));
    } catch {
      // `.git` 판별 실패 시 init은 허용하고(클릭 시 재검증),
      // 삭제는 비활성 처리.
      selectedHasDotGit.value = false;
    } finally {
      selectedHasDotGitLoading.value = false;
    }
  },
  { immediate: true },
);

const refreshSelectedHasDotGit = async () => {
  if (!selectedFolderPath.value) {
    selectedHasDotGit.value = null;
    return;
  }
  selectedHasDotGitLoading.value = true;
  try {
    selectedHasDotGit.value = await invoke<boolean>(
      "path_is_git_repo_root",
      repoPathArgs(selectedFolderPath.value),
    );
  } catch {
    // 판별 실패는 "Git repo 아님"으로 가정(버튼 비활성에 영향을 주기 위해)
    selectedHasDotGit.value = false;
  } finally {
    selectedHasDotGitLoading.value = false;
  }
};

const runGitInit = async () => {
  const p = singleSelectedPath.value;
  if (!p) return;
  try {
    // 버튼이 켜졌더라도 로컬 cached 상태와 서버 상태가 어긵 않을 수 있으니
    // 클릭 시점에 `.git` 존재 여부를 한 번 더 확인한다.
    const isGitRepo = await invoke<boolean>("path_is_git_repo_root", repoPathArgs(p));
    if (isGitRepo) {
      // 이미 git 저장소 메타데이터가 존재하는 경우: 조용히 종료 (또는 toast)
      return;
    }
    await invoke("git_init", repoPathArgs(p));
    await reload();
    await refreshSelectedHasDotGit();
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const openGitRemoveConfirm = () => {
  if (!canGitRemoveRepo.value) return;
  confirmGitRemove.value = true;
};

const doGitRemoveDotGit = async () => {
  const p = singleSelectedPath.value;
  if (!p) return;
  confirmGitRemove.value = false;
  try {
    await invoke("git_remove_dot_git", repoPathArgs(p));
    await reload();
    await refreshSelectedHasDotGit();
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const openMoveProjectModal = () => {
  if (!props.project || !singleSelectedPath.value) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  if (!hasOtherProjects.value) return;
  moveModalOpen.value = true;
};

const moveFolderToProject = async (toProjectId: string) => {
  const path = singleSelectedPath.value;
  if (!props.project || !path) return;
  await invoke("move_root_to_project", {
    fromProjectId: props.project.id,
    toProjectId,
    path,
  });
  moveModalOpen.value = false;
  selectedFolderPath.value = null;
  selectedFolderPaths.value = [];
  selectionAnchorPath.value = null;
  await syncFromBackend();
};

// ── Remote / Archive / Gitignore 다이얼로그 ──────────────────────────────
const showRemote = ref(false);
const showArchive = ref(false);
const showGitignoreModal = ref(false);
const gitignoreModalContent = ref("");
const gitignoreTemplates = ref<Array<{ name: string; content: string }>>([]);
const templateSyncing = ref(false);

const openGitignoreModal = async () => {
  const p = singleSelectedPath.value;
  if (!p) { toast(t("workspace.actionNeedsFolder"), "error"); return; }
  try {
    const gi = await invoke<{ exists: boolean; content: string }>("read_gitignore", repoPathArgs(p));
    gitignoreModalContent.value = gi.content ?? "";
    gitignoreTemplates.value = await invoke("template_list");
    showGitignoreModal.value = true;
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const saveGitignoreModal = async (content: string) => {
  const p = singleSelectedPath.value;
  if (!p) return;
  try {
    await invoke("write_gitignore", { ...repoPathArgs(p), content });
    gitignoreModalContent.value = content;
    toast(t("workspace.gitignoreSaveSuccess"), "success");
    showGitignoreModal.value = false;
    await props.reload();
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const syncTemplateInModal = async () => {
  templateSyncing.value = true;
  try {
    const r = await invoke<{ count: number }>("template_sync");
    gitignoreTemplates.value = await invoke("template_list");
    toast(t("workspace.templateSyncSuccess", { count: r.count }), "success");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  } finally {
    templateSyncing.value = false;
  }
};

</script>

<template>
  <div class="folder-panel">
    <header class="folder-head">
      <h2 class="title">{{ $t("workspace.foldersPanel") }}</h2>
    </header>

    <div
      v-if="project && loading && folderRowsTotal > 0"
      class="folder-load-banner"
      role="status"
      :aria-busy="true"
      :aria-label="
        $t('workspace.folderRowsLoadProgress', {
          loaded: folderRowsLoaded,
          total: folderRowsTotal,
        })
      "
    >
      <div
        class="folder-load-banner__track"
        :class="{ 'folder-load-banner__track--indeterminate': folderRowsLoaded === 0 }"
      >
        <div
          class="folder-load-banner__bar"
          :style="
            folderRowsLoaded === 0
              ? undefined
              : {
                  width: `${Math.min(100, (folderRowsLoaded / folderRowsTotal) * 100)}%`,
                }
          "
        />
      </div>
      <span class="folder-load-banner__text">{{
        $t("workspace.folderRowsLoadProgress", {
          loaded: folderRowsLoaded,
          total: folderRowsTotal,
        })
      }}</span>
    </div>

    <div v-if="!project" class="empty">
      {{ $t("workspace.selectProjectHint") }}
    </div>

    <template v-else>
      <!-- 한 줄: 좌측 액션 버튼 · 우측 표시·필터·정렬 (좁으면 줄바꿈, 필터는 우측 정렬 유지) -->
      <div class="folder-top-bar">
        <div class="folder-toolbar">
          <div class="toolbar-actions">
            <!-- 그룹 1: 목록 전체 -->
            <Button type="button" size="sm" variant="default" @click="addFolder">
              <span aria-hidden="true">➕</span>
              {{ $t("workspace.addFolder") }}
            </Button>
            <Button
              type="button"
              size="sm"
              variant="secondary"
              :disabled="!project.rootPaths.length || loading"
              @click="reload()"
            >
              <span aria-hidden="true">🔄</span>
              {{ $t("workspace.refreshFolderInfo") }}
            </Button>

            <div class="toolbar-divider" aria-hidden="true" />

            <!-- 그룹 2: 멀티 선택 가능 -->
            <Button
              type="button"
              size="sm"
              variant="destructive"
              :disabled="!canRemove"
              @click="removeSelected"
            >
              <span aria-hidden="true">🗑️</span>
              {{ $t("workspace.deleteSelected") }}
            </Button>
          </div>
        </div>

        <div
          class="folder-filter-bar"
          role="group"
          :aria-label="$t('workspace.folderFilterBarGroup')"
        >
          <label class="filter-label" for="folder-view-mode">{{ $t("workspace.viewModeLabel") }}</label>
          <Select id="folder-view-mode" v-model="folderViewMode" class="filter-select" size="sm">
            <option value="list">{{ $t("workspace.viewModeList") }}</option>
            <option value="icon">{{ $t("workspace.viewModeIcon") }}</option>
          </Select>
          <label class="filter-label" for="folder-filter">{{ $t("workspace.filterLabel") }}</label>
          <Select id="folder-filter" v-model="filterMode" class="filter-select" size="sm">
            <option value="all">{{ $t("workspace.filterAll") }}</option>
            <option value="remote">{{ $t("workspace.filterRemoteConnected") }}</option>
            <option value="git">{{ $t("workspace.filterGitRepo") }}</option>
            <option value="non_git">{{ $t("workspace.filterNotGitRepo") }}</option>
          </Select>
          <label class="filter-label" for="folder-sort">{{ $t("workspace.sortLabel") }}</label>
          <Select id="folder-sort" v-model="sortMode" class="filter-select" size="sm">
            <option value="name">{{ $t("workspace.sortByName") }}</option>
            <option value="path">{{ $t("workspace.sortByPath") }}</option>
            <option value="status">{{ $t("workspace.sortByStatus") }}</option>
          </Select>
          <Button
            type="button"
            size="sm"
            variant="secondary"
            class="sort-dir-btn"
            :aria-label="sortDir === 'asc' ? '오름차순' : '내림차순'"
            @click="sortDir = sortDir === 'asc' ? 'desc' : 'asc'"
          >{{ sortDir === "asc" ? "↑" : "↓" }}</Button>
        </div>
      </div>

      <div class="folder-body">
        <div class="list-section">
          <FolderDropZone class="folder-drop-wrap" @dropped="emit('dropped', $event)">
            <div v-if="project.rootPaths.length" class="list-block">
              <RootPathsList
                :paths="sortedFilteredPaths"
                :rows="rows"
                :loading="loading"
                :view-mode="folderViewMode"
                :selected-path="selectedFolderPath"
                :selected-paths="selectedFolderPaths"
                :project-id="project.id"
                @select="onRootRowClick"
              />
            </div>
            <p v-else class="hint">{{ $t("workspace.noFoldersYet") }}</p>
          </FolderDropZone>
        </div>
        <div v-if="selectedFolderPath" class="folder-detail-pane">
          <FolderSelectionDetail
            class="selection-detail"
            :path="selectedFolderPath"
            :row="selectedRow"
            :loading="loading"
            :can-path-actions="canPathActions"
            :reveal-label-key="revealLabelKey"
            :can-move="Boolean(singleSelectedPath && !loading && hasOtherProjects)"
            :can-git-init="canGitInit"
            :can-git-remove-repo="canGitRemoveRepo"
            @copy-path="copyPathInvoke"
            @copy-remote="copyRemote"
            @open-remote-in-browser="openRemoteInBrowser"
            @reveal-path="revealPath"
            @open-external="openExternal"
            @open-move-modal="openMoveProjectModal"
            @git-init="() => void runGitInit()"
            @git-remove="openGitRemoveConfirm"
            @open-remote-manager="showRemote = true"
            @open-archive="showArchive = true"
            @open-gitignore="() => void openGitignoreModal()"
          />
        </div>
      </div>
    </template>

    <!-- 폴더 링크 제거 확인 AlertDialog -->
    <AlertDialogRoot :open="confirmRemove" @update:open="(v) => { if (!v) confirmRemove = false }">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{{ $t("workspace.removeFolderLinkTitle") }}</AlertDialogTitle>
          <AlertDialogDescription>
            {{
              pathsToRemove.length > 1
                ? $t("workspace.removeFolderLinksBulkMessage", { count: pathsToRemove.length })
                : $t("workspace.removeFolderLinkMessage")
            }}
          </AlertDialogDescription>
          <ul v-if="pathsToRemove.length" class="path-list mt-2">
            <li v-for="rp in pathsToRemove" :key="rp"><code>{{ rp }}</code></li>
          </ul>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="confirmRemove = false">{{ $t("workspace.cancel") }}</AlertDialogCancel>
          <AlertDialogAction @click="confirmRemoveDo">{{ $t("workspace.remove") }}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialogRoot>

    <!-- Git 저장소 삭제 확인 AlertDialog -->
    <AlertDialogRoot :open="confirmGitRemove" @update:open="(v) => { if (!v) confirmGitRemove = false }">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{{ $t("workspace.gitRemoveRepoTitle") }}</AlertDialogTitle>
          <AlertDialogDescription>{{ $t("workspace.gitRemoveRepoMessage") }}</AlertDialogDescription>
          <p v-if="singleSelectedPath" class="text-xs text-[var(--muted-foreground)] break-all mt-1">
            <code>{{ singleSelectedPath }}</code>
          </p>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="confirmGitRemove = false">{{ $t("workspace.cancel") }}</AlertDialogCancel>
          <AlertDialogAction @click="doGitRemoveDotGit">{{ $t("workspace.gitRemoveRepoConfirm") }}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialogRoot>

    <MoveRootToProjectModal
      v-if="moveModalOpen && project"
      :projects="projects.filter((p) => p.id !== project!.id)"
      @close="moveModalOpen = false"
      @submit="moveFolderToProject"
    />

    <!-- Remote 관리 Dialog -->
    <RemoteManagerDialog
      v-if="showRemote"
      :repo-path="singleSelectedPath"
      @updated="() => void props.reload()"
      @close="showRemote = false"
    />

    <!-- 아카이브 Dialog -->
    <ArchiveDialog
      v-if="showArchive"
      :repo-path="singleSelectedPath"
      @close="showArchive = false"
    />

    <!-- Gitignore 편집 Dialog -->
    <DialogRoot :open="showGitignoreModal" @update:open="(v) => { if (!v) showGitignoreModal = false }">
      <DialogContent class="w-[min(720px,96vw)] max-h-[min(90vh,720px)] max-w-none flex flex-col">
        <DialogHeader>
          <DialogTitle>{{ $t("workspace.editGitignore") }}</DialogTitle>
        </DialogHeader>
        <div class="min-h-0 flex-1 overflow-auto px-4 py-3 grid gap-3">
          <TemplatePicker
            compact
            :items="gitignoreTemplates"
            :syncing="templateSyncing"
            @apply="(c) => { gitignoreModalContent = c }"
            @sync="() => void syncTemplateInModal()"
          />
          <GitignoreEditor
            hide-heading
            :content="gitignoreModalContent"
            @save="saveGitignoreModal"
            @cancel="showGitignoreModal = false"
          />
        </div>
      </DialogContent>
    </DialogRoot>

    <!-- 외부 도구 선택 Dialog -->
    <DialogRoot :open="showExternalPicker" @update:open="(v) => { if (!v) showExternalPicker = false }">
      <DialogContent class="max-w-xs">
        <DialogHeader>
          <DialogTitle>{{ $t("workspace.externalOpenPickTitle") }}</DialogTitle>
        </DialogHeader>
        <ul class="grid gap-2 px-4 py-3">
          <li v-for="tool in usableExternalTools" :key="tool.id">
            <Button type="button" size="sm" variant="secondary" class="w-full justify-start" @click="runExternalWithTool(tool)">
              {{ tool.label?.trim() || tool.command }}
            </Button>
          </li>
        </ul>
        <DialogFooter>
          <Button type="button" size="sm" variant="secondary" @click="showExternalPicker = false">
            {{ $t("workspace.cancel") }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </DialogRoot>
  </div>
</template>

<style scoped>
.folder-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.folder-head {
  flex-shrink: 0;
  margin-bottom: 4px;
}

.title {
  margin: 0;
  font-size: 0.88rem;
  font-weight: 600;
}

.folder-load-banner {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0 0 8px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid rgb(59 130 246 / 35%);
  background: rgb(15 23 42 / 95%);
  box-shadow: 0 0 0 1px rgb(0 0 0 / 20%);
}

.folder-load-banner__track {
  flex: 1;
  min-width: 120px;
  height: 8px;
  border-radius: 999px;
  background: rgb(55 65 81 / 85%);
  overflow: hidden;
  position: relative;
}

.folder-load-banner__track--indeterminate .folder-load-banner__bar {
  position: absolute;
  left: 0;
  top: 0;
  width: 38%;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #22c55e, #3b82f6);
  animation: folder-load-indeterminate 1.05s ease-in-out infinite;
}

@keyframes folder-load-indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(320%);
  }
}

.folder-load-banner__bar {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #22c55e, #3b82f6);
  transition: width 0.12s ease-out;
  min-width: 4px;
}

.folder-load-banner__text {
  flex-shrink: 0;
  font-size: 0.75rem;
  font-weight: 500;
  opacity: 0.95;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

.empty {
  color: var(--muted-foreground);
  font-size: 0.9rem;
  line-height: 1.45;
}

.folder-top-bar {
  flex-shrink: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px 12px;
  padding: 2px 0 8px;
  margin-bottom: 4px;
  border-bottom: 1px solid var(--color-border);
}

.folder-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  flex: 0 1 auto;
  min-width: 0;
}

.folder-filter-bar {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  flex: 0 1 auto;
  justify-content: flex-end;
  margin-left: auto;
  min-width: 0;
}

.toolbar-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
}

.toolbar-divider {
  width: 1px;
  height: 18px;
  background: var(--color-border);
  flex-shrink: 0;
  margin: 0 2px;
}

.filter-label {
  font-size: 0.72rem;
  opacity: 0.8;
  white-space: nowrap;
}

.filter-select {
  height: 24px;
  font-size: 0.74rem;
  border-radius: 6px;
  width: auto;
  min-width: 92px;
  border: 1px solid var(--border);
  background: var(--background);
  color: inherit;
}

.sort-dir-btn {
  height: 24px;
  min-width: 28px;
  padding: 0 6px;
  font-size: 0.8rem;
  flex-shrink: 0;
}

.toolbar-actions :deep(button) {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

/* 목록만 flex로 늘어나고, 폴더 정보는 flex-shrink:0으로 높이 고정(내용은 패널 내부 스크롤) */
.folder-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow: hidden;
}

.list-section {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.folder-drop-wrap {
  flex: 1;
  min-height: 0;
}

.list-block {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.folder-detail-pane {
  flex: 0 0 260px;
  width: 100%;
  min-height: 0;
  max-height: 230px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.folder-detail-pane :deep(.detail) {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.hint {
  margin: 0;
  padding: 8px 0;
  font-size: 0.88rem;
  color: var(--muted-foreground);
}


.path-list {
  margin: 0;
  padding-left: 1.1rem;
  max-height: 12rem;
  overflow: auto;
  font-size: 0.78rem;
}

.path-list li {
  margin: 0.2rem 0;
  word-break: break-all;
}
</style>
