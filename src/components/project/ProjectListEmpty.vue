<script setup lang="ts">
import UiButton from "@/components/ui/UiButton.vue";
</script>

<template>
  <div class="empty">
    <h3>프로젝트가 없습니다</h3>
    <p>새 프로젝트를 만들거나 폴더를 추가해 시작하세요.</p>
    <UiButton type="button" size="sm" variant="primary" @click="$emit('create')">프로젝트 만들기</UiButton>
  </div>
</template>

<style scoped>
.empty {
  border: 1px dashed var(--color-border);
  border-radius: 8px;
  padding: 16px;
  display: grid;
  gap: 8px;
}
</style>
