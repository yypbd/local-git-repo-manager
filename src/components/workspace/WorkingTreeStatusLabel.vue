<script setup lang="ts">
defineProps<{
  trackedChanges: number;
  untrackedFiles: number;
}>();
</script>

<template>
  <template v-if="trackedChanges > 0 && untrackedFiles > 0">
    {{ $t("workspace.statusDirtyBoth", { m: trackedChanges, n: untrackedFiles }) }}
  </template>
  <template v-else-if="trackedChanges > 0">
    {{ $t("workspace.statusDirtyTrackedOnly", { m: trackedChanges }) }}
  </template>
  <template v-else>
    {{ $t("workspace.statusDirtyUntrackedOnly", { n: untrackedFiles }) }}
  </template>
</template>
