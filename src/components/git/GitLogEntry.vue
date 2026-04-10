<script setup lang="ts">
import { ref } from "vue";
import Button from "@/components/ui/Button.vue";
defineProps<{ entry: { command: string; cwd: string; exitCode: number; stdout: string; stderr: string } }>();
const open = ref(false);
</script>

<template>
  <article class="entry">
    <Button type="button" size="sm" variant="secondary" @click="open = !open">
      {{ entry.command }} ({{ entry.exitCode }})
    </Button>
    <pre v-if="open">{{ entry.stdout || entry.stderr || "(empty)" }}</pre>
  </article>
</template>

<style scoped>
.entry { display: grid; gap: 6px; border-top: 1px solid var(--color-border); padding-top: 6px; }
pre { white-space: pre-wrap; margin: 0; }
</style>
