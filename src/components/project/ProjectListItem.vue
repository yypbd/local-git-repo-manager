<script setup lang="ts">
import { computed } from "vue";
import type { Project } from "@/stores/projects";
import Button from "@/components/ui/Button.vue";

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
  <!-- row -->
  <div class="flex items-stretch gap-1 min-w-0">
    <!--
      drag-handle: Tailwind handles flex-shrink, border, rounded, font, padding,
      color, user-select. The four drag-specific properties that have no direct
      Tailwind equivalent (cursor:grab, touch-action:none, opacity:0.65,
      width:22px) are kept in the scoped <style> block below.
    -->
    <Button
      type="button"
      size="sm"
      variant="secondary"
      class="drag-handle flex-shrink-0 border border-border rounded-sm bg-surface-muted text-xs leading-none !p-0 text-inherit select-none"
      :title="$t('workspace.projectDragReorder')"
      @pointerdown="emit('reorderPointerDown', $event, project)"
      @click.prevent.stop
    >
      ⠿
    </Button>

    <!-- item link -->
    <RouterLink
      :to="`/projects/${project.id}`"
      class="item flex-1 min-w-0 border border-border rounded-md p-[10px] grid gap-[6px] no-underline text-inherit transition-[background,border-color] duration-[120ms] ease-[ease] hover:bg-surface-muted"
      active-class="active"
      draggable="false"
    >
      <!-- headline -->
      <div class="flex items-center justify-start gap-2 min-h-[26px] min-w-0">
        <!-- title -->
        <span
          class="inline-flex items-center font-semibold leading-none flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap"
          :title="project.name"
        >
          {{ project.name }}
        </span>

        <!-- right-tools -->
        <div class="inline-flex items-center gap-[6px] ml-auto flex-shrink-0">
          <!-- badge -->
          <span class="inline-flex items-center gap-1 border border-border rounded-full text-[0.72rem] px-[7px] py-[1px] leading-none bg-white/[0.04] flex-shrink-0">
            <span aria-hidden="true">📁</span>
            {{ folderCount }}
          </span>

          <!-- actions -->
          <div class="flex gap-1">
            <Button
              type="button"
              size="sm"
              variant="secondary"
              class="icon-btn !w-[26px] !p-0 inline-flex items-center"
              :title="$t('workspace.projectRename')"
              :aria-label="$t('workspace.projectRename')"
              @click.prevent.stop="emit('rename', project)"
            >
              <span aria-hidden="true">✏️</span>
            </Button>
            <Button
              type="button"
              size="sm"
              variant="danger"
              class="icon-btn !w-[26px] !p-0 inline-flex items-center"
              :title="$t('workspace.projectDelete')"
              :aria-label="$t('workspace.projectDelete')"
              @click.prevent.stop="emit('delete', project)"
            >
              <span aria-hidden="true">🗑️</span>
            </Button>
          </div>
        </div>
      </div>
    </RouterLink>
  </div>
</template>

<style scoped>
/*
  Drag-handle properties with no direct Tailwind equivalent — kept here per
  TechLead directive (cursor:grab, touch-action:none, opacity:0.65, width:22px).
*/
.drag-handle {
  width: 22px;
  opacity: 0.65;
  cursor: grab;
  touch-action: none;
}

.drag-handle:hover {
  opacity: 1;
}

.drag-handle:active {
  cursor: grabbing;
}

/* Active state for the RouterLink — uses a colour not in the Tailwind token set */
.item.active {
  border-color: #7aa2ff;
  background: rgba(122, 162, 255, 0.12);
}
</style>
