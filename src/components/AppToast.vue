<script setup lang="ts">
import { useToastStore } from "@/stores/toast";
import UiButton from "@/components/ui/UiButton.vue";

const { items, remove } = useToastStore();
</script>

<template>
  <div class="toast-wrap">
    <div v-for="item in items" :key="item.id" class="toast" :data-type="item.type">
      <span>{{ item.message }}</span>
      <UiButton type="button" size="sm" variant="secondary" @click="remove(item.id)">닫기</UiButton>
    </div>
  </div>
</template>

<style scoped>
.toast-wrap {
  position: fixed;
  right: 12px;
  bottom: 12px;
  display: grid;
  gap: 8px;
  z-index: 50;
}

.toast {
  min-width: 260px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: #1a2030;
  padding: 10px;
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.toast[data-type="success"] {
  border-color: #2e8b57;
}

.toast[data-type="error"] {
  border-color: #b34b4b;
}

.toast[data-type="warning"] {
  border-color: #b8860b;
}
</style>
