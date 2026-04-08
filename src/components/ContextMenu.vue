<script setup lang="ts">
import UiButton from "@/components/ui/UiButton.vue";

type Action = { label: string; onClick: () => void };
defineProps<{ actions: Action[] }>();
</script>

<template>
  <div class="menu">
    <UiButton
      v-for="item in actions"
      :key="item.label"
      type="button"
      size="sm"
      variant="secondary"
      @click="item.onClick()"
    >
      {{ item.label }}
    </UiButton>
  </div>
</template>

<style scoped>
.menu { display: flex; gap: 6px; flex-wrap: wrap; }
</style>
