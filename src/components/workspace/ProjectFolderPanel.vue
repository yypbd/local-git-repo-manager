<script setup lang="ts">
import { computed, ref, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import FolderDropZone from "@/components/FolderDropZone.vue";
import FolderSelectionDetail from "@/components/workspace/FolderSelectionDetail.vue";
import RootPathsList from "@/components/workspace/RootPathsList.vue";
import MoveRootToProjectModal from "@/components/tree/MoveRootToProjectModal.vue";
import UiSelect from "@/components/ui/UiSelect.vue";
import UiButton from "@/components/ui/UiButton.vue";
import { pickDirectories } from "@/composables/pickFolder";
import {
  lookupFolderRow,
  sortRootPathsForDisplay,
  type FolderRootRow,
} from "@/composables/useFolderRootRows";
import { useProjectsStore } from "@/stores/projects";
import { useToastStore } from "@/stores/toast";
import type { Project } from "@/stores/projects";
import { repoPathArgs } from "@/utils/tauriRepoPath";

const props = defineProps<{
  project: Project | undefined;
  rows: Record<string, FolderRootRow>;
  loading: boolean;
  reload: () => void | Promise<void>;
}>();

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
  const sorted = sortRootPathsForDisplay(props.project.rootPaths);
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

const moveRootToProject = async (toProjectId: string) => {
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

watchEffect((onCleanup) => {
  if (!confirmRemove.value) return;
  const h = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      confirmRemove.value = false;
      return;
    }
    if (e.key === "Enter") {
      const t = e.target as HTMLElement;
      if (t.tagName === "TEXTAREA" || t.tagName === "BUTTON") return;
      e.preventDefault();
      void confirmRemoveDo();
    }
  };
  window.addEventListener("keydown", h, true);
  onCleanup(() => window.removeEventListener("keydown", h, true));
});

watchEffect((onCleanup) => {
  if (!confirmGitRemove.value) return;
  const h = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      confirmGitRemove.value = false;
      return;
    }
    if (e.key === "Enter") {
      const t = e.target as HTMLElement;
      if (t.tagName === "TEXTAREA" || t.tagName === "BUTTON") return;
      e.preventDefault();
      void doGitRemoveDotGit();
    }
  };
  window.addEventListener("keydown", h, true);
  onCleanup(() => window.removeEventListener("keydown", h, true));
});

watchEffect((onCleanup) => {
  if (!showExternalPicker.value) return;
  const h = (e: KeyboardEvent) => {
    if (e.key !== "Escape") return;
    e.preventDefault();
    e.stopPropagation();
    showExternalPicker.value = false;
  };
  window.addEventListener("keydown", h, true);
  onCleanup(() => window.removeEventListener("keydown", h, true));
});
</script>

<template>
  <div class="folder-panel">
    <header class="folder-head">
      <h2 class="title">{{ $t("workspace.foldersPanel") }}</h2>
    </header>

    <div v-if="!project" class="empty">
      {{ $t("workspace.selectProjectHint") }}
    </div>

    <template v-else>
      <div class="folder-toolbar">
        <div class="toolbar-actions">
          <UiButton type="button" size="sm" variant="primary" @click="addFolder">
            <span aria-hidden="true">➕</span>
            {{ $t("workspace.addFolder") }}
          </UiButton>
          <UiButton
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!project.rootPaths.length || loading"
            @click="reload()"
          >
            <span aria-hidden="true">🔄</span>
            {{ $t("workspace.refreshFolderInfo") }}
          </UiButton>
          <UiButton
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!singleSelectedPath || loading || !hasOtherProjects"
            @click="openMoveProjectModal"
          >
            <span aria-hidden="true">📦</span>
            {{ $t("workspace.moveRootToOtherProject") }}
          </UiButton>
          <UiButton
            type="button"
            size="sm"
            variant="danger"
            :disabled="!canRemove"
            @click="removeSelected"
          >
            <span aria-hidden="true">🗑️</span>
            {{ $t("workspace.deleteSelected") }}
          </UiButton>
          <UiButton
            type="button"
            size="sm"
            variant="success"
            :disabled="!canGitInit"
            @click="runGitInit"
          >
            <span aria-hidden="true">🌱</span>
            {{ $t("workspace.gitInit") }}
          </UiButton>
          <UiButton
            type="button"
            size="sm"
            variant="danger"
            :disabled="!canGitRemoveRepo"
            @click="openGitRemoveConfirm"
          >
            <span aria-hidden="true">⚠️</span>
            {{ $t("workspace.gitRemoveRepo") }}
          </UiButton>
        </div>
        <div class="toolbar-filter">
          <label class="filter-label" for="folder-view-mode">{{ $t("workspace.viewModeLabel") }}</label>
          <UiSelect id="folder-view-mode" v-model="folderViewMode" class="filter-select" size="sm">
            <option value="list">{{ $t("workspace.viewModeList") }}</option>
            <option value="icon">{{ $t("workspace.viewModeIcon") }}</option>
          </UiSelect>
          <label class="filter-label" for="folder-filter">{{ $t("workspace.filterLabel") }}</label>
          <UiSelect id="folder-filter" v-model="filterMode" class="filter-select" size="sm">
            <option value="all">{{ $t("workspace.filterAll") }}</option>
            <option value="remote">{{ $t("workspace.filterRemoteConnected") }}</option>
            <option value="git">{{ $t("workspace.filterGitRepo") }}</option>
            <option value="non_git">{{ $t("workspace.filterNotGitRepo") }}</option>
          </UiSelect>
        </div>
      </div>

      <div class="folder-body">
        <div class="list-section">
          <FolderDropZone class="folder-drop-wrap" @dropped="emit('dropped', $event)">
            <div v-if="project.rootPaths.length" class="list-block">
              <RootPathsList
                :paths="filteredRootPaths"
                :rows="rows"
                :loading="loading"
                :view-mode="folderViewMode"
                :selected-path="selectedFolderPath"
                :selected-paths="selectedFolderPaths"
                :project-id="project.id"
                @select="onRootRowClick"
              />
            </div>
            <p v-else class="hint">{{ $t("workspace.noRootsYet") }}</p>
          </FolderDropZone>
        </div>
        <div class="folder-detail-pane">
          <FolderSelectionDetail
            class="selection-detail"
            :path="selectedFolderPath"
            :row="selectedRow"
            :loading="loading"
            :can-path-actions="canPathActions"
            :reveal-label-key="revealLabelKey"
            @copy-path="copyPathInvoke"
            @copy-remote="copyRemote"
            @reveal-path="revealPath"
            @open-external="openExternal"
          />
        </div>
      </div>
    </template>

    <div v-if="confirmRemove" class="backdrop modal-backdrop" @click.self="confirmRemove = false">
      <div class="dialog">
        <h3>{{ $t("workspace.removeRootTitle") }}</h3>
        <p class="msg">
          {{
            pathsToRemove.length > 1
              ? $t("workspace.removeRootsBulkMessage", { count: pathsToRemove.length })
              : $t("workspace.removeRootMessage")
          }}
        </p>
        <ul v-if="pathsToRemove.length" class="path-list">
          <li v-for="rp in pathsToRemove" :key="rp"><code>{{ rp }}</code></li>
        </ul>
        <div class="dialog-actions">
          <UiButton type="button" size="sm" variant="secondary" @click="confirmRemove = false">{{ $t("workspace.cancel") }}</UiButton>
          <UiButton type="button" size="sm" variant="danger" @click="confirmRemoveDo">{{ $t("workspace.remove") }}</UiButton>
        </div>
      </div>
    </div>

    <div v-if="confirmGitRemove" class="backdrop modal-backdrop" @click.self="confirmGitRemove = false">
      <div class="dialog">
        <h3>{{ $t("workspace.gitRemoveRepoTitle") }}</h3>
        <p class="msg">{{ $t("workspace.gitRemoveRepoMessage") }}</p>
        <p v-if="singleSelectedPath" class="path-preview"><code>{{ singleSelectedPath }}</code></p>
        <div class="dialog-actions">
          <UiButton type="button" size="sm" variant="secondary" @click="confirmGitRemove = false">{{ $t("workspace.cancel") }}</UiButton>
          <UiButton type="button" size="sm" variant="danger" @click="doGitRemoveDotGit">{{ $t("workspace.gitRemoveRepoConfirm") }}</UiButton>
        </div>
      </div>
    </div>

    <MoveRootToProjectModal
      v-if="moveModalOpen && project"
      :projects="projects.filter((p) => p.id !== project!.id)"
      @close="moveModalOpen = false"
      @submit="moveRootToProject"
    />

    <div
      v-if="showExternalPicker"
      class="backdrop modal-backdrop"
      @click.self="showExternalPicker = false"
    >
      <div class="dialog external-picker-dialog" @click.stop>
        <h3>{{ $t("workspace.externalOpenPickTitle") }}</h3>
        <ul class="path-list">
          <li v-for="tool in usableExternalTools" :key="tool.id">
            <UiButton type="button" size="sm" variant="secondary" @click="runExternalWithTool(tool)">
              {{ tool.label?.trim() || tool.command }}
            </UiButton>
          </li>
        </ul>
        <div class="dialog-actions">
          <UiButton type="button" size="sm" variant="secondary" @click="showExternalPicker = false">
            {{ $t("workspace.cancel") }}
          </UiButton>
        </div>
      </div>
    </div>
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

.empty {
  color: #9ca3af;
  font-size: 0.9rem;
  line-height: 1.45;
}

.folder-toolbar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 4px;
  padding: 2px 0 4px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 4px;
}

.toolbar-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
}

.toolbar-filter {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
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
  border: 1px solid var(--color-border);
  background: #111827;
  color: inherit;
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
  flex: 0 0 280px;
  width: 100%;
  min-height: 0;
  max-height: 280px;
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
  color: #9ca3af;
}


.backdrop {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 45%);
  display: grid;
  place-items: center;
  z-index: 10050;
}

.dialog {
  width: min(400px, 92vw);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: #111522;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.dialog h3 {
  margin: 0;
  font-size: 1rem;
}

.msg {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.4;
}

.path-preview {
  margin: 0;
  word-break: break-all;
}

.path-preview code {
  font-size: 0.78rem;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
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
