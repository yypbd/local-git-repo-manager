<script setup lang="ts">
defineProps<{ name: string }>();
const emit = defineEmits<{ close: []; confirm: [] }>();
</script>

<template>
  <div class="backdrop">
    <div class="dialog">
      <h3>프로젝트 삭제</h3>
      <p>"{{ name }}" 프로젝트를 삭제할까요?</p>
      <div class="actions">
        <button @click="emit('close')">취소</button>
        <button @click="emit('confirm')">삭제</button>
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
  width: 320px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: #111522;
  padding: 14px;
  display: grid;
  gap: 10px;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
