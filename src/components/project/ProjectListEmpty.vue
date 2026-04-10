<script setup lang="ts">
import Button from "@/components/ui/Button.vue";
</script>

<template>
  <div class="empty">
    <h3>프로젝트가 없습니다</h3>
    <p>새 프로젝트를 만들거나 폴더를 추가해 시작하세요.</p>
    <Button type="button" size="sm" variant="default" @click="$emit('create')">프로젝트 만들기</Button>
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
