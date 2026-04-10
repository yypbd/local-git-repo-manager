<script setup lang="ts">
import { ref } from "vue";
import SettingsDialog from "@/components/settings/SettingsDialog.vue";
import Button from "@/components/ui/Button.vue";

const showSettings = ref(false);
</script>

<template>
  <div class="shell">
    <header class="toolbar">
      <strong class="brand">
        <span class="ico" aria-hidden="true">🗂️</span>
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
      <div class="col-right">
        <div class="col-folders-area">
          <slot name="folders" />
        </div>
        <div class="col-detail-stack">
          <div class="detail-scroll">
            <slot name="detail" />
          </div>
        </div>
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

.col-right {
  min-height: 0;
  min-width: 0;
  display: grid;
  /* 아래 상세·액션 행은 내용 높이에 맞춤(불필요한 여백 제거) */
  grid-template-rows: minmax(120px, 1fr) auto;
  border-right: none;
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

.col-detail-stack {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.detail-scroll {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: visible;
  padding: 6px 10px 8px;
}
</style>
