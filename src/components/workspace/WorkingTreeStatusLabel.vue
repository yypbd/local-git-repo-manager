<script setup lang="ts">
defineProps<{
  trackedChanges: number;
  untrackedFiles: number;
}>();
</script>

<template>
  <span class="status-badges">
    <span v-if="trackedChanges > 0" class="status-badge status-badge--tracked">
      M {{ trackedChanges }}
    </span>
    <span v-if="untrackedFiles > 0" class="status-badge status-badge--untracked">
      U {{ untrackedFiles }}
    </span>
  </span>
</template>

<style scoped>
.status-badges {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 4px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0 6px;
  min-height: 18px;
  font-size: 0.66rem;
  line-height: 1;
  font-weight: 600;
}

.status-badge--tracked {
  color: #fbbf24;
  border-color: rgb(251 191 36 / 35%);
  background: rgb(251 191 36 / 14%);
}

.status-badge--untracked {
  color: #60a5fa;
  border-color: rgb(96 165 250 / 35%);
  background: rgb(96 165 250 / 14%);
}
</style>
