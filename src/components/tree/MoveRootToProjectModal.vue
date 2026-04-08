<script setup lang="ts">
import { ref, watch } from "vue";
import type { Project } from "@/stores/projects";
import { useDialogEscape, useDialogInputFocus } from "@/composables/useDialogShortcuts";

const props = defineProps<{ projects: Project[] }>();
const emit = defineEmits<{ close: []; submit: [projectId: string] }>();
const targetId = ref(props.projects[0]?.id ?? "");
const selectRef = ref<HTMLSelectElement | null>(null);

useDialogInputFocus(selectRef);
useDialogEscape(() => emit("close"));

watch(
  () => props.projects,
  (list) => {
    if (!list.length) return;
    if (!list.some((p) => p.id === targetId.value)) {
      targetId.value = list[0]!.id;
    }
  },
  { deep: true },
);

const submit = () => {
  if (!targetId.value) return;
  emit("submit", targetId.value);
};
</script>

<template>
  <div class="backdrop modal-backdrop" @click.self="emit('close')">
    <div class="dialog" @click.stop>
      <h3>다른 프로젝트로 이동</h3>
      <form class="form" @submit.prevent="submit">
        <select ref="selectRef" v-model="targetId">
          <option v-for="item in projects" :key="item.id" :value="item.id">{{ item.name }}</option>
        </select>
        <div class="actions">
          <button type="button" class="btn btn-sm btn-secondary" @click="emit('close')">취소</button>
          <button type="submit" class="btn btn-sm btn-primary">이동</button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  background: rgb(0 0 0 / 40%);
}
.dialog {
  width: 320px;
  background: #161b29;
  border: 1px solid var(--color-border);
  padding: 12px;
  border-radius: 8px;
  display: grid;
  gap: 8px;
}
.form {
  display: grid;
  gap: 8px;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
