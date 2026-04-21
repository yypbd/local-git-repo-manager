<script setup lang="ts">
import { ref } from "vue";
import SettingsDialog from "@/components/settings/SettingsDialog.vue";
import Button from "@/components/ui/Button.vue";

const showSettings = ref(false);
const appIconSrc = new URL("../../src-tauri/icons/32x32.png", import.meta.url).href;
</script>

<template>
  <div class="shell">
    <header class="toolbar">
      <strong class="brand">
        <img class="app-icon" :src="appIconSrc" alt="" aria-hidden="true" />
        {{ $t("app.title") }}
      </strong>
      <div class="toolbar-actions">
        <Button type="button" size="sm" variant="secondary" @click="showSettings = true">
          <span class="ico" aria-hidden="true">⚙️</span>
          {{ $t("nav.settings") }}
        </Button>
      </div>
    </header>

    <div v-if="$slots.full" class="full-bleed">
      <slot name="full" />
    </div>
    <div v-else class="workspace">
      <aside class="col col-projects">
        <slot name="projects" />
      </aside>
      <div class="col-folders-area">
        <slot name="folders" />
      </div>
    </div>

    <SettingsDialog v-model="showSettings" />
  </div>
</template>

<style scoped>
.shell {
  display: grid;
  height: 100vh;
  grid-template-rows: auto 1fr;
  min-height: 0;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--color-border);
  padding: 10px 14px;
  flex-shrink: 0;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
}

.brand {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.toolbar-actions :deep(.btn) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.ico {
  line-height: 1;
  font-size: 0.9em;
}

.app-icon {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  object-fit: contain;
}

.full-bleed {
  min-height: 0;
  overflow: auto;
  padding: 16px;
}

.workspace {
  display: grid;
  grid-template-columns: minmax(240px, 320px) 1fr;
  min-height: 0;
  min-width: 0;
}

.col {
  min-height: 0;
  min-width: 0;
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.col-projects {
  overflow: auto;
  padding: 12px;
  background: var(--color-bg);
}

.col-folders-area {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  padding: 12px;
  background: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
}


</style>
