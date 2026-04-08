<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { pickDirectory } from "@/composables/pickFolder";
import { useToastStore } from "@/stores/toast";
import { useDialogEnterConfirm, useDialogEscape } from "@/composables/useDialogShortcuts";
import { outputParentDirArgs, repoPathArgs } from "@/utils/tauriRepoPath";

const props = defineProps<{ repoPath: string | null }>();
const emit = defineEmits<{ close: [] }>();

const { t } = useI18n();
const { push: toast } = useToastStore();
const exporting = ref(false);

const canExport = computed(() => {
  const p = props.repoPath?.trim();
  return Boolean(p);
});

const pickAndExport = async () => {
  const repo = props.repoPath?.trim();
  if (!repo) {
    toast(t("workspace.archiveNoFolder"), "error");
    return;
  }
  const dest = await pickDirectory();
  if (!dest?.trim()) return;

  exporting.value = true;
  try {
    const outPath = await invoke<string>("git_archive_export", {
      ...repoPathArgs(repo),
      ...outputParentDirArgs(dest.trim()),
    });
    toast(t("workspace.archiveSuccess", { path: outPath }), "success");
    emit("close");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  } finally {
    exporting.value = false;
  }
};

useDialogEscape(() => emit("close"));
useDialogEnterConfirm(
  () => {
    void pickAndExport();
  },
  { enabled: () => canExport.value && !exporting.value },
);
</script>

<template>
  <Teleport to="body">
    <div class="backdrop modal-backdrop" @click.self="emit('close')">
      <div class="dialog" @click.stop>
        <h3 class="title">{{ $t("workspace.archiveTitle") }}</h3>
        <p class="body">{{ $t("workspace.archiveBody") }}</p>
        <p class="filename-hint">{{ $t("workspace.archiveFilenameHint") }}</p>
        <div class="actions">
          <button type="button" class="btn-cancel" @click="emit('close')">
            {{ $t("workspace.cancel") }}
          </button>
          <button
            type="button"
            class="btn-primary"
            :disabled="!canExport || exporting"
            @click="pickAndExport"
          >
            {{ exporting ? $t("workspace.archiveExporting") : $t("workspace.archivePickExport") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 20000;
  display: grid;
  place-items: center;
  background: rgb(0 0 0 / 45%);
  padding: 16px;
}

.dialog {
  width: min(420px, 100%);
  display: grid;
  gap: 10px;
  background: #161b29;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 16px;
}

.title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.body,
.filename-hint {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.45;
  color: #9ca3af;
}

.filename-hint {
  font-family: ui-monospace, monospace;
  font-size: 0.78rem;
  color: #6b7280;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 4px;
}

.btn-cancel {
  padding: 6px 12px;
  font-size: 0.85rem;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.btn-primary {
  padding: 6px 12px;
  font-size: 0.85rem;
  border-radius: 6px;
  border: 1px solid #5b7cff;
  background: rgb(91 124 255 / 22%);
  color: #e8ecff;
  cursor: pointer;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
