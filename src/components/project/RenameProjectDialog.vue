<script setup lang="ts">
import { ref, watch } from "vue";
import { useDialogEscape, useDialogInputFocus } from "@/composables/useDialogShortcuts";
import UiInput from "@/components/ui/UiInput.vue";
import UiButton from "@/components/ui/UiButton.vue";

const props = defineProps<{ initialName: string }>();
const emit = defineEmits<{ close: []; submit: [name: string] }>();

const name = ref(props.initialName);
const inputRef = ref<{ focus?: () => void } | null>(null);
useDialogInputFocus(inputRef);
useDialogEscape(() => emit("close"));

watch(
  () => props.initialName,
  (next) => {
    name.value = next;
  },
);

const submit = () => {
  emit("submit", name.value.trim() || props.initialName);
};
</script>

<template>
  <div class="backdrop modal-backdrop" @click.self="emit('close')">
    <div class="dialog" @click.stop>
      <h3>프로젝트 이름 변경</h3>
      <form class="form" @submit.prevent="submit">
        <UiInput ref="inputRef" v-model="name" autocomplete="off" />
        <div class="actions">
          <UiButton type="button" size="sm" variant="secondary" @click="emit('close')">취소</UiButton>
          <UiButton type="submit" size="sm" variant="primary">저장</UiButton>
        </div>
      </form>
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
  width: 320px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: #111522;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.form {
  display: grid;
  gap: 10px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
