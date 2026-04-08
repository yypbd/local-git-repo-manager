<script setup lang="ts">
type Action = { label: string; onClick: () => void };
defineProps<{ actions: Action[] }>();
</script>

<template>
  <div class="menu">
    <button v-for="item in actions" :key="item.label" @click="item.onClick()">{{ item.label }}</button>
  </div>
</template>

<style scoped>
.menu { display: flex; gap: 6px; flex-wrap: wrap; }
</style>
