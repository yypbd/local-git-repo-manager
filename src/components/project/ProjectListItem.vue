<script setup lang="ts">
import { computed } from "vue";
import type { Project } from "@/stores/projects";

const props = defineProps<{
  project: Project;
}>();

const emit = defineEmits<{
  rename: [project: Project];
  delete: [project: Project];
  /** Tauri/WebView에서 HTML5 드롭이 불안정해 포인터로 순서 변경 */
  reorderPointerDown: [event: PointerEvent, project: Project];
}>();

const folderCount = computed(() => props.project.rootPaths?.length ?? 0);
</script>

<template>
  <div class="row">
    <button
      type="button"
      class="drag-handle"
      :title="$t('workspace.projectDragReorder')"
      @pointerdown="emit('reorderPointerDown', $event, project)"
      @click.prevent.stop
    >
      ⠿
    </button>
    <RouterLink :to="`/projects/${project.id}`" class="item" active-class="active" draggable="false">
      <div class="headline">
        <span class="title" :title="project.name">{{ project.name }}</span>
        <div class="right-tools">
          <span class="badge">
            <span aria-hidden="true">📁</span>
            {{ folderCount }}
          </span>
          <div class="actions">
            <button
              type="button"
              class="btn btn-sm btn-secondary icon-btn"
              :title="$t('workspace.projectRename')"
              :aria-label="$t('workspace.projectRename')"
              @click.stop="emit('rename', project)"
            >
              <span aria-hidden="true">✏️</span>
            </button>
            <button
              type="button"
              class="btn btn-sm btn-danger icon-btn"
              :title="$t('workspace.projectDelete')"
              :aria-label="$t('workspace.projectDelete')"
              @click.stop="emit('delete', project)"
            >
              <span aria-hidden="true">🗑️</span>
            </button>
          </div>
        </div>
      </div>
    </RouterLink>
  </div>
</template>

<style scoped>
.row {
  display: flex;
  align-items: stretch;
  gap: 4px;
  min-width: 0;
}

.drag-handle {
  flex-shrink: 0;
  width: 22px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface-muted);
  cursor: grab;
  font-size: 0.75rem;
  line-height: 1;
  padding: 0;
  color: inherit;
  opacity: 0.65;
  touch-action: none;
  user-select: none;
}

.drag-handle:hover {
  opacity: 1;
}

.drag-handle:active {
  cursor: grabbing;
}

.item {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 10px;
  display: grid;
  gap: 6px;
  text-decoration: none;
  color: inherit;
  transition: background 0.12s ease, border-color 0.12s ease;
}

.item:hover {
  background: var(--color-surface-muted);
}

.item.active {
  border-color: #7aa2ff;
  background: rgba(122, 162, 255, 0.12);
}

.title {
  display: inline-flex;
  align-items: center;
  font-weight: 600;
  line-height: 1;
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.headline {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  min-height: 26px;
  min-width: 0;
}

.right-tools {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-left: auto;
  flex-shrink: 0;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid var(--color-border);
  border-radius: 999px;
  font-size: 0.72rem;
  padding: 1px 7px;
  line-height: 1;
  background: rgb(255 255 255 / 4%);
  flex-shrink: 0;
}

.actions {
  display: flex;
  gap: 4px;
}

.actions button {
  display: inline-flex;
  align-items: center;
}

.icon-btn {
  width: 26px;
  padding: 0;
}
</style>
