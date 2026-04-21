<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import type { FolderRootRow } from "@/composables/useFolderRootRows";
import {
  beginFolderMovePointer,
  folderMoveDraggingPaths,
  folderMovePointerActive,
  suppressFolderRowClick,
} from "@/composables/rootFolderProjectPointerMove";
import {
  folderNameFromPath,
  lookupFolderRow,
} from "@/composables/useFolderRootRows";
import { useProjectsStore } from "@/stores/projects";
import { useToastStore } from "@/stores/toast";
import WorkingTreeStatusLabel from "@/components/workspace/WorkingTreeStatusLabel.vue";

const router = useRouter();
const { t } = useI18n();
const { syncFromBackend } = useProjectsStore();
const { push: toast } = useToastStore();

function dropDetail(err: unknown): string {
  if (err instanceof Error && err.message) return err.message;
  return String(err);
}

const props = withDefaults(
  defineProps<{
    paths: string[];
    rows: Record<string, FolderRootRow>;
    loading: boolean;
    viewMode: "list" | "icon";
    /** 포커스(상세 패널 등) — 멀티 선택 시 마지막 클릭 */
    selectedPath: string | null;
    /** 멀티 선택된 경로(정렬된 목록 기준 Ctrl/Shift) */
    selectedPaths: string[];
    /** 있으면 행을 다른 프로젝트로 드래그해 이동 가능 */
    projectId?: string | null;
  }>(),
  { projectId: null },
);

const emit = defineEmits<{
  select: [path: string, event: MouseEvent];
}>();

/** `rows` 키와 목록 `path` 문자열(NFC/NFD) 불일치 시에도 행을 찾습니다. */
const rowByPath = computed(() => {
  const map: Record<string, FolderRootRow | undefined> = {};
  for (const path of props.paths) {
    map[path] = lookupFolderRow(props.rows, path);
  }
  return map;
});

/** 전역 로딩 중이어도, 이미 `folder_root_row`가 온 행은 원격·브랜치·상태를 바로 표시 */
function rowMetaPending(path: string): boolean {
  return props.loading && rowByPath.value[path] === undefined;
}

/** `folder_root_row` 조회 완료 후 `gitError`인 경우만 — 행 단위 비Git 시각 구분용 */
function isNonGitFolder(path: string): boolean {
  if (rowMetaPending(path)) return false;
  const row = rowByPath.value[path];
  return Boolean(row?.gitError);
}

/** Git 저장소로 확인됐으나 origin(대표 remote) URL 없음 — 행 왼쪽 강조·경로 아래 보조 문구용 */
function showRemoteMissingBadge(path: string): boolean {
  if (rowMetaPending(path)) return false;
  const row = rowByPath.value[path];
  if (!row || row.gitError) return false;
  return !row.remote?.trim();
}

/** 행 왼쪽 색상 바: 비Git 우선, 그다음 원격 없음 */
function folderRowAccentState(path: string): "non-git" | "no-remote" | undefined {
  if (isNonGitFolder(path)) return "non-git";
  if (showRemoteMissingBadge(path)) return "no-remote";
  return undefined;
}

/** listbox option용 스크린 리더 요약 (FOLDER-STATUS-05) */
function folderRowAriaLabel(path: string): string {
  const name = folderNameFromPath(path);
  if (rowMetaPending(path)) {
    return t("workspace.folderRowAriaLoading", { name });
  }
  const row = rowByPath.value[path];
  if (!row) {
    return t("workspace.folderRowAriaUnknown", { name, path });
  }
  if (row.gitError) {
    return t("workspace.folderRowAriaNotGit", { name, path });
  }
  const branch = row.branch?.trim() || "—";
  let statusSentence: string;
  if (row.clean) {
    statusSentence = t("workspace.folderRowAriaWorktreeClean");
  } else {
    const m = row.trackedChanges;
    const n = row.untrackedFiles;
    if (m > 0 && n > 0) statusSentence = t("workspace.statusDirtyBoth", { m, n });
    else if (m > 0) statusSentence = t("workspace.statusDirtyTrackedOnly", { m });
    else if (n > 0) statusSentence = t("workspace.statusDirtyUntrackedOnly", { n });
    else statusSentence = t("workspace.statusHasChangesLabel");
  }
  const remoteSentence = row.remote?.trim()
    ? t("workspace.folderRowAriaRemoteConnected")
    : t("workspace.remoteNoOrigin");
  return t("workspace.folderRowAriaGit", {
    name,
    path,
    branch,
    statusSentence,
    remoteSentence,
  });
}

function isRowSelected(path: string): boolean {
  return props.selectedPaths.length > 0
    ? props.selectedPaths.includes(path)
    : props.selectedPath === path;
}

function pathsForMove(path: string): string[] {
  if (props.selectedPaths.length > 0 && props.selectedPaths.includes(path)) {
    return [...props.selectedPaths];
  }
  return [path];
}

function onFolderPointerDown(e: PointerEvent, path: string) {
  const pid = props.projectId;
  if (!pid) return;
  beginFolderMovePointer(
    e,
    { fromProjectId: pid, paths: pathsForMove(path) },
    {
      afterMove: async (toId) => {
        await syncFromBackend();
        toast(t("workspace.moveFolderDropDone"), "success");
        void router.push(`/projects/${toId}`);
      },
      onError: (err) => {
        toast(t("workspace.projectDeleteFolderMoveFailed", { detail: dropDetail(err) }), "error");
      },
    },
  );
}

const isFolderMoveActive = computed(() => folderMovePointerActive.value);

function rowIsDragMoving(path: string): boolean {
  const paths = folderMoveDraggingPaths.value;
  return Boolean(paths?.includes(path));
}

function onRowClick(path: string, e: MouseEvent) {
  if (suppressFolderRowClick.value) {
    suppressFolderRowClick.value = false;
    e.preventDefault();
    e.stopPropagation();
    return;
  }
  emit("select", path, e);
}
</script>

<template>
  <ul
    class="roots"
    :class="{
      'roots--icon': viewMode === 'icon',
      'roots--folder-move': isFolderMoveActive,
    }"
    role="listbox"
    aria-multiselectable="true"
    :aria-label="$t('workspace.foldersPanel')"
  >
    <li
      v-for="path in props.paths"
      :key="path"
      role="option"
      :aria-label="folderRowAriaLabel(path)"
      :aria-selected="isRowSelected(path)"
      class="root-row"
      :data-state="folderRowAccentState(path)"
      :class="{
        selected: isRowSelected(path),
        'root-row--draggable': projectId,
        'root-row--drag-moving': rowIsDragMoving(path),
      }"
      @pointerdown="onFolderPointerDown($event, path)"
      @click="onRowClick(path, $event)"
    >
      <div v-if="viewMode === 'list'" class="row" aria-hidden="true">
        <!-- 1행: 폴더명 + 브랜치/상태 -->
        <div class="row-main">
          <span class="name">{{ folderNameFromPath(path) }}</span>
          <div class="tail">
            <span v-if="!rowByPath[path]?.gitError" class="branch">{{
              rowMetaPending(path) ? "…" : (rowByPath[path]?.branch ?? "—")
            }}</span>
            <span class="status">
              <template v-if="rowMetaPending(path)">…</template>
              <template v-else-if="!rowByPath[path]">—</template>
              <template v-else-if="rowByPath[path]!.gitError" />
              <template v-else-if="rowByPath[path]!.clean">
                <span class="status-tag status-tag--clean">{{ $t("workspace.statusClean") }}</span>
              </template>
              <template v-else>
                <WorkingTreeStatusLabel
                  :tracked-changes="rowByPath[path]!.trackedChanges"
                  :untracked-files="rowByPath[path]!.untrackedFiles"
                />
              </template>
            </span>
          </div>
        </div>
        <!-- 2행: 로컬 경로 + 리모트 경로 (보조 텍스트) -->
        <div class="row-sub">
          <code class="path-local">{{ path }}</code>
          <template v-if="rowMetaPending(path)">
            <span class="path-remote muted">…</span>
          </template>
          <span v-else-if="rowByPath[path]?.gitError" class="path-remote muted">{{ $t("workspace.remoteNotGit") }}</span>
          <span v-else-if="rowByPath[path]?.remote" class="path-remote">
            {{ rowByPath[path]!.remote
            }}<small v-if="(rowByPath[path]!.remoteCount ?? 0) > 1" class="remote-more">
              +{{ rowByPath[path]!.remoteCount - 1 }}
            </small>
          </span>
          <span
            v-else-if="showRemoteMissingBadge(path)"
            class="path-remote path-remote--aux"
          >{{ $t("workspace.remoteNoOrigin") }}</span>
        </div>
      </div>
      <div v-else class="icon-card" aria-hidden="true">
        <div class="folder-emoji" aria-hidden="true">📁</div>
        <span class="name name--icon">{{ folderNameFromPath(path) }}</span>
        <code class="path-mini">{{ path }}</code>
        <span v-if="rowMetaPending(path)" class="mini muted">…</span>
        <template v-else-if="rowByPath[path]?.gitError" />
        <template v-else>
          <span
            v-if="showRemoteMissingBadge(path)"
            class="mini path-remote-line"
          >{{ $t("workspace.remoteNoOrigin") }}</span>
          <div class="icon-card-status">
          <span v-if="rowByPath[path]?.clean" class="mini">{{ $t("workspace.statusClean") }}</span>
          <span v-else class="mini dirty">
            <WorkingTreeStatusLabel
              :tracked-changes="rowByPath[path]!.trackedChanges"
              :untracked-files="rowByPath[path]!.untrackedFiles"
            />
          </span>
          </div>
        </template>
      </div>
    </li>
  </ul>
</template>

<style scoped>
.roots {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 5px;
  /* 전역 `code { user-select: text }`보다 우선 — 행 드래그 시 경로가 잡히지 않게 */
  user-select: none;
  -webkit-user-select: none;
}

.roots code {
  user-select: none;
  -webkit-user-select: none;
}

.roots--icon {
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 8px;
}

.root-row {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 5px 7px;
  background: var(--color-surface-muted);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.12s ease, background 0.12s ease;
}

.root-row:hover {
  background: #1a2130;
}

.root-row.selected {
  border-color: #7aa2ff;
  background: rgba(122, 162, 255, 0.1);
}

.root-row--draggable {
  cursor: grab;
}

.root-row--draggable:active {
  cursor: grabbing;
}

/* 포인터로 프로젝트까지 끄는 중 */
.roots--folder-move {
  outline: 1px dashed rgba(122, 162, 255, 0.45);
  outline-offset: 3px;
  border-radius: 10px;
}

.root-row--drag-moving {
  outline: 2px solid rgba(122, 162, 255, 0.9);
  background: rgba(122, 162, 255, 0.18);
  box-shadow: inset 0 0 0 1px rgba(122, 162, 255, 0.35);
}

/* FOLDER-STATUS-01: Git 저장소 아님 — 목록·아이콘 공통(행 `li`) */
.root-row[data-state="non-git"] {
  background: rgb(148 163 184 / 0.1);
  box-shadow: inset 3px 0 0 0 rgb(100 116 139 / 0.92);
}

.root-row[data-state="non-git"]:hover {
  background: rgb(148 163 184 / 0.16);
}

.root-row[data-state="non-git"].selected {
  border-color: #7aa2ff;
  background: rgba(122, 162, 255, 0.11);
  box-shadow: inset 3px 0 0 0 rgb(100 116 139 / 0.92);
}

.root-row[data-state="non-git"].selected:hover {
  background: rgba(122, 162, 255, 0.16);
}

.root-row[data-state="non-git"].root-row--drag-moving {
  box-shadow:
    inset 3px 0 0 0 rgb(100 116 139 / 0.92),
    inset 0 0 0 1px rgba(122, 162, 255, 0.35);
}

/* Git인데 원격 URL 없음 — 비Git(slate)과 구분되는 왼쪽 앰버 바 */
.root-row[data-state="no-remote"] {
  background: rgb(251 191 36 / 0.09);
  box-shadow: inset 3px 0 0 0 rgb(217 119 6 / 0.9);
}

.root-row[data-state="no-remote"]:hover {
  background: rgb(251 191 36 / 0.14);
}

.root-row[data-state="no-remote"].selected {
  border-color: #7aa2ff;
  background: rgba(122, 162, 255, 0.11);
  box-shadow: inset 3px 0 0 0 rgb(217 119 6 / 0.9);
}

.root-row[data-state="no-remote"].selected:hover {
  background: rgba(122, 162, 255, 0.16);
}

.root-row[data-state="no-remote"].root-row--drag-moving {
  box-shadow:
    inset 3px 0 0 0 rgb(217 119 6 / 0.9),
    inset 0 0 0 1px rgba(122, 162, 255, 0.35);
}

.row {
  display: flex;
  flex-direction: column;
  gap: 3px;
  width: 100%;
  min-width: 0;
}

/* 1행: 폴더명 + 우측 브랜치/상태 */
.row-main {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.name {
  flex: 1 1 0;
  min-width: 0;
  font-weight: 600;
  font-size: 0.82rem;
  line-height: 1.25;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tail {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 5px;
}

.branch {
  font-family: ui-monospace, monospace;
  font-size: 0.68rem;
  opacity: 0.8;
}

.status {
  font-size: 0.68rem;
}

.status-tag {
  display: inline-flex;
  align-items: center;
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0 6px;
  min-height: 18px;
  line-height: 1;
  font-size: 0.64rem;
  font-weight: 600;
}

.status-tag--clean {
  color: var(--folder-status-clean-fg);
  border-color: var(--folder-status-clean-border);
  background: var(--folder-status-clean-bg);
}

.icon-card-status {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 4px;
  width: 100%;
}

/* 2행: 경로 텍스트들 (보조) */
.row-sub {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.path-local {
  font-family: ui-monospace, monospace;
  font-size: 0.62rem;
  opacity: 0.65;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  line-height: 1.3;
}

.path-remote {
  font-family: ui-monospace, monospace;
  font-size: 0.58rem;
  opacity: 0.5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  line-height: 1.3;
  font-style: normal;
}

.path-remote--aux {
  opacity: 0.72;
  font-style: normal;
}

.remote-more {
  margin-left: 4px;
  font-size: 0.58rem;
  opacity: 0.8;
}

.muted {
  opacity: 0.55;
  font-style: italic;
}

.icon-card {
  display: grid;
  justify-items: center;
  gap: 4px;
  text-align: center;
}

.folder-emoji {
  font-size: 1.5rem;
  line-height: 1;
}

.name--icon {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
  width: 100%;
}

.path-mini {
  width: 100%;
  font-size: 0.58rem;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini {
  font-size: 0.62rem;
}

.path-remote-line {
  display: block;
  width: 100%;
  font-size: 0.58rem;
  opacity: 0.72;
  line-height: 1.3;
  text-align: center;
}
</style>
