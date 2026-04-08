<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";
import GitLogEntry from "@/components/git/GitLogEntry.vue";
import UiButton from "@/components/ui/UiButton.vue";

withDefaults(
  defineProps<{
    /** 사이드바(구버전) | 상세 패널 하단 */
    variant?: "sidebar" | "below";
  }>(),
  { variant: "below" },
);

const logs = ref<Array<{ id: string; command: string; cwd: string; exitCode: number; stdout: string; stderr: string }>>([]);
const width = ref(Number(sessionStorage.getItem("ui.logPanelWidth") || 280));

const refresh = async () => {
  try {
    logs.value = await invoke("logs_list");
  } catch {
    logs.value = [];
  }
};

const narrow = () => {
  width.value = Math.max(220, width.value - 20);
  sessionStorage.setItem("ui.logPanelWidth", String(width.value));
};
const widen = () => {
  width.value = Math.min(520, width.value + 20);
  sessionStorage.setItem("ui.logPanelWidth", String(width.value));
};

onMounted(() => {
  void refresh();
});
</script>

<template>
  <aside class="log-panel" :class="variant" :style="variant === 'sidebar' ? { width: `${width}px` } : undefined">
    <h3 v-if="variant === 'sidebar'">{{ $t("nav.logs") }}</h3>
    <div class="tools">
      <UiButton type="button" size="sm" variant="secondary" @click="refresh">새로고침</UiButton>
      <template v-if="variant === 'sidebar'">
        <UiButton type="button" size="sm" variant="secondary" @click="narrow">-</UiButton>
        <UiButton type="button" size="sm" variant="secondary" @click="widen">+</UiButton>
      </template>
    </div>
    <div class="entries">
      <GitLogEntry v-for="entry in logs" :key="entry.id" :entry="entry" />
    </div>
  </aside>
</template>

<style scoped>
.log-panel {
  padding: 12px;
  background: var(--color-surface-muted);
  overflow: auto;
}

.log-panel.sidebar {
  border-left: 1px solid var(--color-border);
  min-width: 220px;
}

.log-panel.below {
  max-height: min(40vh, 360px);
  border-top: 1px solid var(--color-border);
}

.tools {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.entries {
  display: grid;
  gap: 8px;
}

h3 {
  margin: 0 0 8px;
  font-size: 0.95rem;
}
</style>
