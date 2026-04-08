<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Project } from "@/stores/projects";
import { useDialogEnterConfirm, useDialogEscape } from "@/composables/useDialogShortcuts";
import UiButton from "@/components/ui/UiButton.vue";

const props = defineProps<{ project: Project; hasOtherProjects: boolean }>();
const emit = defineEmits<{
  close: [];
  deleteOnly: [];
  moveThenDelete: [];
}>();

const { t } = useI18n();
const hasRoots = computed(() => props.project.rootPaths.length > 0);
const rootCount = computed(() => props.project.rootPaths.length);

useDialogEscape(() => emit("close"));
useDialogEnterConfirm(() => {
  if (hasRoots.value && props.hasOtherProjects) emit("moveThenDelete");
  else emit("deleteOnly");
});
</script>

<template>
  <div class="backdrop modal-backdrop" @click.self="emit('close')">
    <div class="dialog">
      <h3>{{ $t("workspace.projectDeleteTitle") }}</h3>
      <p>{{ $t("workspace.projectDeleteMessage", { name: project.name }) }}</p>
      <p v-if="hasRoots" class="hint">
        {{ $t("workspace.projectDeleteRootsHint", { n: rootCount }) }}
      </p>
      <div class="actions">
        <UiButton type="button" size="sm" variant="secondary" class="btn-muted" @click="emit('close')">
          {{ $t("workspace.cancel") }}
        </UiButton>
        <template v-if="hasRoots">
          <UiButton
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!props.hasOtherProjects"
            @click="emit('moveThenDelete')"
          >
            {{ $t("workspace.projectDeleteMoveFirst") }}
          </UiButton>
          <UiButton type="button" size="sm" variant="danger" @click="emit('deleteOnly')">
            {{ $t("workspace.projectDeleteUnlinkOnly") }}
          </UiButton>
        </template>
        <UiButton v-else type="button" size="sm" variant="danger" @click="emit('deleteOnly')">
          {{ $t("workspace.projectDeleteConfirm") }}
        </UiButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 45%);
  display: grid;
  place-items: center;
}

.dialog {
  width: min(400px, calc(100vw - 32px));
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

.dialog p {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.4;
}

.hint {
  opacity: 0.9;
  font-size: 0.85rem;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.btn-muted {
  margin-right: auto;
}
</style>
