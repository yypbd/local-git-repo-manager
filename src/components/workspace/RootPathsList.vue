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
  sortRootPathsForDisplay,
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

const sortedPaths = computed(() => sortRootPathsForDisplay(props.paths));

/** `rows` 키와 목록 `path` 문자열(NFC/NFD) 불일치 시에도 행을 찾습니다. */
const rowByPath = computed(() => {
  const map: Record<string, FolderRootRow | undefined> = {};
  for (const path of sortedPaths.value) {
    map[path] = lookupFolderRow(props.rows, path);
  }
  return map;
});

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
        toast(t("workspace.moveRootDropDone"), "success");
        void router.push(`/projects/${toId}`);
      },
      onError: (err) => {
        toast(t("workspace.projectDeleteMoveFailed", { detail: dropDetail(err) }), "error");
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
      v-for="path in sortedPaths"
      :key="path"
      role="option"
      :aria-selected="isRowSelected(path)"
      class="root-row"
      :class="{
        selected: isRowSelected(path),
        'root-row--draggable': projectId,
        'root-row--drag-moving': rowIsDragMoving(path),
      }"
      @pointerdown="onFolderPointerDown($event, path)"
      @click="onRowClick(path, $event)"
    >
      <div v-if="viewMode === 'list'" class="row">
        <span class="name">{{ folderNameFromPath(path) }}</span>
        <div class="paths-grid">
          <code class="cell local">{{ path }}</code>
          <code v-if="loading" class="cell remote muted">…</code>
          <span v-else-if="rowByPath[path]?.gitError" class="cell remote muted">{{ $t("workspace.remoteNotGit") }}</span>
          <span v-else-if="rowByPath[path]?.remote" class="cell remote">
            <code>{{ rowByPath[path]!.remote }}</code>
            <small
              v-if="(rowByPath[path]!.remoteCount ?? 0) > 1"
              class="remote-more"
            >
              {{ $t("workspace.remoteMoreCount", { count: rowByPath[path]!.remoteCount - 1 }) }}
            </small>
          </span>
          <span v-else class="cell remote muted">{{ $t("workspace.remoteNoOrigin") }}</span>
        </div>
        <div class="tail">
          <span class="branch">{{ loading ? "…" : (rowByPath[path]?.branch ?? "—") }}</span>
          <span class="dash" aria-hidden="true">-</span>
          <span class="status" :class="{ dirty: rowByPath[path] && !rowByPath[path]!.gitError && !rowByPath[path]!.clean }">
            <template v-if="loading">…</template>
            <template v-else-if="!rowByPath[path]">—</template>
            <template v-else-if="rowByPath[path]!.gitError">{{ $t("workspace.notGitRepo") }}</template>
            <template v-else-if="rowByPath[path]!.clean">{{ $t("workspace.statusClean") }}</template>
            <template v-else>
              <WorkingTreeStatusLabel
                :tracked-changes="rowByPath[path]!.trackedChanges"
                :untracked-files="rowByPath[path]!.untrackedFiles"
              />
            </template>
          </span>
        </div>
      </div>
      <div v-else class="icon-card">
        <div class="folder-emoji" aria-hidden="true">📁</div>
        <span class="name name--icon">{{ folderNameFromPath(path) }}</span>
        <code class="path-mini">{{ path }}</code>
        <span v-if="loading" class="mini muted">…</span>
        <span v-else-if="rowByPath[path]?.gitError" class="mini muted">{{ $t("workspace.notGitRepo") }}</span>
        <span v-else-if="rowByPath[path]?.clean" class="mini">{{ $t("workspace.statusClean") }}</span>
        <span v-else class="mini dirty">
          <WorkingTreeStatusLabel
            :tracked-changes="rowByPath[path]!.trackedChanges"
            :untracked-files="rowByPath[path]!.untrackedFiles"
          />
        </span>
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

.row {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 6px 10px;
  width: 100%;
  min-width: 0;
}

.name {
  font-weight: 600;
  font-size: 0.72rem;
  flex-shrink: 0;
  line-height: 1.25;
}

.paths-grid {
  flex: 1 1 10rem;
  min-width: 0;
  display: grid;
  grid-template-rows: auto auto;
  grid-template-columns: minmax(0, 1fr);
  gap: 2px;
  align-content: start;
  justify-items: stretch;
}

.cell {
  margin: 0;
  padding: 0;
  font-family: ui-monospace, monospace;
  font-size: 0.55rem;
  word-break: break-all;
  line-height: 1.25;
  text-align: left;
}

.local {
  color: var(--color-text);
  opacity: 0.9;
}

.remote {
  color: var(--color-text);
  opacity: 0.82;
}

.remote-more {
  margin-left: 6px;
  font-size: 0.62rem;
  opacity: 0.75;
}

.tail {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 6px;
  flex: 0 1 auto;
  margin-left: auto;
}

.branch {
  font-family: ui-monospace, monospace;
  font-size: 0.68rem;
}

.status {
  font-size: 0.68rem;
}

.status.dirty {
  color: #fbbf24;
}

.dash {
  opacity: 0.4;
  user-select: none;
  font-size: 0.68rem;
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
</style>
