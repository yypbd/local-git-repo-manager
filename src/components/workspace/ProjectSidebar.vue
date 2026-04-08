<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import CreateProjectModal from "@/components/project/CreateProjectModal.vue";
import ProjectDeleteDialog from "@/components/project/ProjectDeleteDialog.vue";
import FolderDropZone from "@/components/FolderDropZone.vue";
import ProjectListEmpty from "@/components/project/ProjectListEmpty.vue";
import ProjectListItem from "@/components/project/ProjectListItem.vue";
import RenameProjectDialog from "@/components/project/RenameProjectDialog.vue";
import MoveRootToProjectModal from "@/components/tree/MoveRootToProjectModal.vue";
import { useProjectsStore, type Project } from "@/stores/projects";
import { useToastStore } from "@/stores/toast";
import { folderMoveDropTargetId } from "@/composables/rootFolderProjectPointerMove";

const { t } = useI18n();
const router = useRouter();
const { push: pushToast } = useToastStore();

const {
  projects,
  hasProjects,
  syncFromBackend,
  createProject,
  renameProject,
  deleteProject,
  importFolderDrop,
  reorderProjects,
} = useProjectsStore();

const creating = ref(false);
const renaming = ref<Project | null>(null);

/** 삭제 플로우: dialog → (선택) pick 대상 프로젝트 */
const deleting = ref<Project | null>(null);
const deleteStep = ref<"dialog" | "pick" | null>(null);

/** 포인터 재정렬 중 하이라이트 */
const dragSourceId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);
/** 동시에 두 세션 방지 */
let reorderPointerSession: {
  fromId: string;
  pointerId: number;
  handle: HTMLElement;
  onMove: (ev: PointerEvent) => void;
  onEnd: (ev: PointerEvent) => void;
} | null = null;

function dropDetail(err: unknown): string {
  if (err instanceof Error && err.message) return err.message;
  return String(err);
}

function reorderIds(ids: string[], fromIdx: number, toIdx: number): string[] {
  const next = [...ids];
  const [item] = next.splice(fromIdx, 1);
  next.splice(toIdx, 0, item!);
  return next;
}

function teardownReorderListeners() {
  if (!reorderPointerSession) return;
  const { handle, pointerId, onMove, onEnd } = reorderPointerSession;
  window.removeEventListener("pointermove", onMove);
  window.removeEventListener("pointerup", onEnd);
  window.removeEventListener("pointercancel", onEnd);
  try {
    if (handle.hasPointerCapture(pointerId)) {
      handle.releasePointerCapture(pointerId);
    }
  } catch {
    /* noop */
  }
  reorderPointerSession = null;
}

function onReorderPointerDown(e: PointerEvent, project: Project) {
  if (e.pointerType === "mouse" && e.button !== 0) return;
  if (reorderPointerSession) return;

  e.preventDefault();
  e.stopPropagation();

  const handle = e.currentTarget as HTMLElement;
  handle.setPointerCapture(e.pointerId);

  const fromId = project.id;
  const pointerId = e.pointerId;

  dragSourceId.value = fromId;
  dragOverId.value = null;

  const onMove = (ev: PointerEvent) => {
    if (ev.pointerId !== pointerId) return;
    const el = document.elementFromPoint(ev.clientX, ev.clientY);
    const row = el?.closest("[data-project-id]");
    const id = row?.getAttribute("data-project-id");
    dragOverId.value = id ?? null;
  };

  const onEnd = (ev: PointerEvent) => {
    if (ev.pointerId !== pointerId) return;

    const el = document.elementFromPoint(ev.clientX, ev.clientY);
    const row = el?.closest("[data-project-id]");
    const targetId = row?.getAttribute("data-project-id");

    teardownReorderListeners();

    dragSourceId.value = null;
    dragOverId.value = null;

    if (!targetId || fromId === targetId) return;

    const ids = projects.value.map((p) => p.id);
    const fromIdx = ids.indexOf(fromId);
    const toIdx = ids.indexOf(targetId);
    if (fromIdx < 0 || toIdx < 0) return;
    const ordered = reorderIds(ids, fromIdx, toIdx);
    void (async () => {
      try {
        await reorderProjects(ordered);
      } catch (err) {
        pushToast(t("workspace.projectReorderFailed", { detail: dropDetail(err) }), "error");
      }
    })();
  };

  reorderPointerSession = { fromId, pointerId, handle, onMove, onEnd };
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onEnd);
  window.addEventListener("pointercancel", onEnd);
}

function openDelete(p: Project) {
  deleting.value = p;
  deleteStep.value = "dialog";
}

function closeDeleteFlow() {
  deleting.value = null;
  deleteStep.value = null;
}

function onDeleteMoveFirst() {
  const others = projects.value.filter((p) => p.id !== deleting.value?.id);
  if (others.length === 0) {
    pushToast(t("workspace.projectDeleteNoOtherProject"), "error");
    return;
  }
  deleteStep.value = "pick";
}

async function onDeleteOnly() {
  const id = deleting.value?.id;
  if (!id) return;
  deleteStep.value = null;
  deleting.value = null;
  await deleteProject(id);
}

async function onMoveAllRootsThenDelete(toProjectId: string) {
  const p = deleting.value;
  if (!p) return;
  try {
    for (const path of p.rootPaths) {
      await invoke("move_root_to_project", {
        fromProjectId: p.id,
        toProjectId,
        path,
      });
    }
    await syncFromBackend();
    deleteStep.value = null;
    deleting.value = null;
    await deleteProject(p.id);
  } catch (err) {
    pushToast(t("workspace.projectDeleteMoveFailed", { detail: dropDetail(err) }), "error");
  }
}

async function onCreateProject(name: string) {
  try {
    await createProject(name);
    creating.value = false;
  } catch (e) {
    pushToast(dropDetail(e), "error");
  }
}

async function onRenameProject(name: string) {
  if (!renaming.value) return;
  try {
    await renameProject(renaming.value.id, name);
    renaming.value = null;
  } catch (e) {
    pushToast(dropDetail(e), "error");
  }
}

const movePickerProjects = () =>
  projects.value.filter((p) => p.id !== deleting.value?.id);

async function onProjectPanelDrop(paths: string[]) {
  let lastId: string | null = null;
  for (const path of paths) {
    try {
      const p = await importFolderDrop(path);
      lastId = p.id;
    } catch (e) {
      const detail = dropDetail(e);
      if (detail.includes("no subfolders to import")) {
        pushToast(t("workspace.importNoSubfolders"), "error");
      } else {
        pushToast(t("workspace.importFolderDropFailed", { detail }), "error");
      }
    }
  }
  await syncFromBackend();
  if (lastId) {
    void router.push(`/projects/${lastId}`);
  }
}

onMounted(() => {
  void syncFromBackend();
});

onUnmounted(() => {
  teardownReorderListeners();
});
</script>

<template>
  <div class="project-drop">
    <FolderDropZone @dropped="onProjectPanelDrop">
      <div class="sidebar">
        <div class="sidebar-head">
          <h2 class="title">{{ $t("workspace.projectsPanel") }}</h2>
          <UiButton type="button" size="sm" variant="secondary" @click="creating = true">+</UiButton>
        </div>

        <ProjectListEmpty v-if="!hasProjects" @create="creating = true" />

        <div v-else class="list">
          <div
            v-for="p in projects"
            :key="p.id"
            class="project-row"
            :data-project-id="p.id"
            :class="{
              'drag-over': dragOverId === p.id && dragSourceId && dragSourceId !== p.id,
              'root-folder-drop-over': folderMoveDropTargetId === p.id,
            }"
          >
            <ProjectListItem
              :project="p"
              @rename="renaming = $event"
              @delete="openDelete($event)"
              @reorder-pointer-down="onReorderPointerDown"
            />
          </div>
        </div>

        <CreateProjectModal
          v-if="creating"
          @close="creating = false"
          @submit="(name) => void onCreateProject(name)"
        />

        <RenameProjectDialog
          v-if="renaming"
          :initial-name="renaming.name"
          @close="renaming = null"
          @submit="(name) => void onRenameProject(name)"
        />

        <ProjectDeleteDialog
          v-if="deleteStep === 'dialog' && deleting"
          :project="deleting"
          @close="closeDeleteFlow"
          @delete-only="onDeleteOnly"
          @move-then-delete="onDeleteMoveFirst"
        />

        <MoveRootToProjectModal
          v-if="deleteStep === 'pick' && deleting"
          :key="`pick-${deleting.id}`"
          :projects="movePickerProjects()"
          @close="deleteStep = 'dialog'"
          @submit="onMoveAllRootsThenDelete"
        />
      </div>
    </FolderDropZone>
  </div>
</template>

<style scoped>
.project-drop {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.sidebar {
  display: grid;
  gap: 10px;
  align-content: start;
}

.sidebar-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.list {
  display: grid;
  gap: 8px;
}

.project-row {
  border-radius: 10px;
  transition: box-shadow 0.12s ease;
}

.project-row.drag-over {
  box-shadow: 0 0 0 2px rgba(122, 162, 255, 0.55);
}

.project-row.root-folder-drop-over {
  box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.75);
  background: rgba(74, 222, 128, 0.14);
}
</style>
