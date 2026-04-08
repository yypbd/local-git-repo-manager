<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useToastStore } from "@/stores/toast";
import type { Project } from "@/stores/projects";
import {
  lookupFolderRow,
  type FolderRootRow,
} from "@/composables/useFolderRootRows";
import { repoPathArgs } from "@/utils/tauriRepoPath";
import RemoteManagerDialog from "@/components/git/RemoteManagerDialog.vue";
import ArchiveDialog from "@/components/git/ArchiveDialog.vue";
import GitignoreEditor from "@/components/git/GitignoreEditor.vue";
import TemplatePicker from "@/components/git/TemplatePicker.vue";

type ExternalTool = { id: string; label: string; command: string; argsTemplate: string };

const props = defineProps<{
  project: Project | undefined;
  folderRows: Record<string, FolderRootRow>;
  selectedFolderPath: string | null;
  reloadFolderRows: () => void | Promise<void>;
}>();

const { t } = useI18n();
const { push: toast } = useToastStore();

/** 상세 액션은 "명시적으로 선택된 폴더" 기준으로 동작 */
const activePath = computed(() => props.selectedFolderPath?.trim() || null);

const activeRow = computed(() => {
  const p = activePath.value;
  if (!p) return undefined;
  return lookupFolderRow(props.folderRows, p);
});

const hasSelectedFolder = computed(() => Boolean(activePath.value));
const isSelectedGitRepo = computed(() => {
  if (!hasSelectedFolder.value) return false;
  const row = activeRow.value;
  if (!row) return false;
  return !row.gitError;
});

const showRemote = ref(false);
const showArchive = ref(false);

/** 원격 URL 복사 가능: 폴더 선택됨, Git 저장소, origin URL 있음 */
const canCopyRemoteUrl = computed(() => {
  const row = activeRow.value;
  const p = activePath.value;
  if (!p || !row || row.gitError) return false;
  return Boolean(row.remote?.trim());
});

/** 폴더 경로만 있으면 시도 가능. `folderRows`에 행이 아직 없을 수 있어 `!row`로 막지 않음. */
const canArchive = computed(() => isSelectedGitRepo.value);
const canRemoteManage = computed(() => isSelectedGitRepo.value);
const canEditGitignore = computed(() => isSelectedGitRepo.value);

const showGitignoreModal = ref(false);
const gitignoreModalContent = ref("");
const templates = ref<Array<{ name: string; content: string }>>([]);
const templateSyncing = ref(false);
const platformName = (typeof navigator !== "undefined" ? navigator.userAgent : "").toLowerCase();
const revealButtonLabelKey = computed(() => {
  if (platformName.includes("mac")) return "workspace.openInFinder";
  if (platformName.includes("win")) return "workspace.openInExplorer";
  return "workspace.openInFileManager";
});

const externalTools = ref<ExternalTool[]>([]);
const showExternalPicker = ref(false);

const loadExternalTools = async () => {
  try {
    const s = await invoke<{ externalTools?: ExternalTool[] }>("get_settings");
    externalTools.value = s.externalTools ?? [];
  } catch {
    externalTools.value = [];
  }
};

const copyPathInvoke = async () => {
  const p = activePath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  try {
    await navigator.clipboard.writeText(p);
    toast(t("workspace.copyDone"), "success");
  } catch {
    toast(t("workspace.clipboardCopyFailed"), "error");
  }
};

const copyRemote = async () => {
  const row = activeRow.value;
  const remote = row?.remote?.trim();
  if (!remote || row?.gitError) {
    toast(t("workspace.copyRemoteUnavailable"), "error");
    return;
  }
  try {
    await navigator.clipboard.writeText(remote);
    toast(t("workspace.copyRemoteSuccess"), "success");
  } catch {
    toast(t("workspace.clipboardCopyFailed"), "error");
  }
};

const revealPath = async () => {
  const p = activePath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  await invoke("reveal_path", { path: p });
};

const usableExternalTools = computed(() =>
  externalTools.value.filter((x) => x.command?.trim()),
);

const runExternalWithTool = async (tool: ExternalTool) => {
  const p = activePath.value;
  if (!p) return;
  const cmd = tool.command.trim();
  if (!cmd) {
    toast(t("workspace.externalOpenBadTool"), "error");
    return;
  }
  try {
    const tpl = tool.argsTemplate?.trim();
    await invoke("open_external", {
      path: p,
      command: cmd,
      argsTemplate: tpl ? tpl : null,
    });
    showExternalPicker.value = false;
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const openExternal = async () => {
  const p = activePath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  await loadExternalTools();
  const tools = usableExternalTools.value;
  if (tools.length === 0) {
    toast(t("workspace.externalOpenNoTools"), "error");
    return;
  }
  if (tools.length === 1) {
    await runExternalWithTool(tools[0]!);
    return;
  }
  showExternalPicker.value = true;
};

const openGitignoreModal = async () => {
  const p = activePath.value;
  if (!p) {
    toast(t("workspace.actionNeedsFolder"), "error");
    return;
  }
  try {
    const gi = await invoke<{ exists: boolean; content: string }>("read_gitignore", repoPathArgs(p));
    gitignoreModalContent.value = gi.content ?? "";
    templates.value = await invoke("template_list");
    showGitignoreModal.value = true;
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const saveGitignoreModal = async (content: string) => {
  const p = activePath.value;
  if (!p) return;
  try {
    await invoke("write_gitignore", { ...repoPathArgs(p), content });
    gitignoreModalContent.value = content;
    toast(t("workspace.gitignoreSaveSuccess"), "success");
    showGitignoreModal.value = false;
    await props.reloadFolderRows();
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const applyTemplateInModal = (content: string) => {
  gitignoreModalContent.value = content;
};

const syncTemplateInModal = async () => {
  templateSyncing.value = true;
  try {
    const r = await invoke<{ count: number }>("template_sync");
    templates.value = await invoke("template_list");
    toast(t("workspace.templateSyncSuccess", { count: r.count }), "success");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  } finally {
    templateSyncing.value = false;
  }
};

watchEffect((onCleanup) => {
  if (!showExternalPicker.value) return;
  const h = (e: KeyboardEvent) => {
    if (e.key !== "Escape") return;
    e.preventDefault();
    e.stopPropagation();
    showExternalPicker.value = false;
  };
  window.addEventListener("keydown", h, true);
  onCleanup(() => window.removeEventListener("keydown", h, true));
});

</script>

<template>
  <div class="detail-panel">
    <div v-if="!project" class="empty">
      {{ $t("workspace.selectProjectHintDetail") }}
    </div>
    <template v-else>
      <div class="actions-footer">
        <div class="action-row action-row--primary">
          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!hasSelectedFolder"
            @click="copyPathInvoke"
          >
            <span aria-hidden="true">📋</span>
            {{ $t("workspace.copyLocalPath") }}
          </button>

          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!canCopyRemoteUrl"
            @click="copyRemote"
          >
            <span aria-hidden="true">🔗</span>
            {{ $t("workspace.copyUrlMenu") }}
          </button>

          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!hasSelectedFolder"
            @click="revealPath"
          >
            <span aria-hidden="true">🧭</span>
            {{ $t(revealButtonLabelKey) }}
          </button>
          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!hasSelectedFolder"
            @click="openExternal"
          >
            <span aria-hidden="true">🚀</span>
            {{ $t("workspace.openExternalApp") }}
          </button>
        </div>

        <div class="action-row action-row--secondary">
          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!canRemoteManage"
            @click="showRemote = true"
          >
            <span aria-hidden="true">🌐</span>
            {{ $t("workspace.actionRemoteManager") }}
          </button>
          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!canArchive"
            @click="showArchive = true"
          >
            <span aria-hidden="true">🗜️</span>
            {{ $t("workspace.actionArchive") }}
          </button>
          <button
            type="button"
            class="btn btn-sm btn-secondary"
            :disabled="!canEditGitignore"
            @click="openGitignoreModal"
          >
            <span aria-hidden="true">✍️</span>
            {{ $t("workspace.editGitignore") }}
          </button>
        </div>
      </div>
    </template>

    <div
      v-if="showGitignoreModal"
      class="backdrop modal-backdrop"
      @click.self="showGitignoreModal = false"
    >
      <div class="gitignore-dialog unified" @click.stop>
        <h3 class="dialog-title">{{ $t("workspace.editGitignore") }}</h3>
        <TemplatePicker
          compact
          :items="templates"
          :syncing="templateSyncing"
          @apply="applyTemplateInModal"
          @sync="() => void syncTemplateInModal()"
        />
        <GitignoreEditor
          hide-heading
          :content="gitignoreModalContent"
          @save="saveGitignoreModal"
          @cancel="showGitignoreModal = false"
        />
        <div class="dialog-actions">
          <button type="button" class="btn btn-sm btn-secondary" @click="showGitignoreModal = false">{{ $t("workspace.close") }}</button>
        </div>
      </div>
    </div>

    <RemoteManagerDialog
      v-if="showRemote"
      :repo-path="activePath"
      @updated="() => void props.reloadFolderRows()"
      @close="showRemote = false"
    />
    <ArchiveDialog
      v-if="showArchive"
      :repo-path="activePath"
      @close="showArchive = false"
    />

    <div
      v-if="showExternalPicker"
      class="backdrop modal-backdrop"
      @click.self="showExternalPicker = false"
    >
      <div class="external-picker-dialog" @click.stop>
        <h3 class="dialog-title">{{ $t("workspace.externalOpenPickTitle") }}</h3>
        <ul class="external-tool-list">
          <li v-for="tool in usableExternalTools" :key="tool.id">
            <button type="button" class="btn-tool" @click="runExternalWithTool(tool)">
              {{ tool.label?.trim() || tool.command }}
            </button>
          </li>
        </ul>
        <div class="dialog-actions">
          <button type="button" class="btn btn-sm btn-secondary" @click="showExternalPicker = false">{{ $t("workspace.cancel") }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail-panel {
  flex: 0 0 auto;
  width: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
  box-sizing: border-box;
}

.empty {
  color: #9ca3af;
  font-size: 0.9rem;
  line-height: 1.45;
}

.actions-footer {
  flex: 0 0 auto;
  padding-top: 4px;
  border-top: 1px solid var(--color-border);
  display: grid;
  gap: 4px;
}

.action-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 6px;
  align-items: center;
}

.action-row--primary {
  padding-bottom: 0;
}

.action-row--secondary {
  padding-top: 2px;
  border-top: 1px solid rgb(255 255 255 / 5%);
}

.backdrop {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 45%);
  display: grid;
  place-items: center;
  z-index: 10050;
  padding: 16px;
}

.gitignore-dialog.unified {
  width: min(720px, 96vw);
  max-height: min(90vh, 720px);
  overflow: auto;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: #111522;
  padding: 14px;
  display: grid;
  gap: 12px;
}

.dialog-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
}

.external-picker-dialog {
  width: min(400px, 92vw);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: #111522;
  padding: 14px;
  display: grid;
  gap: 12px;
}

.external-tool-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 6px;
}

.btn-tool {
  width: 100%;
  text-align: left;
  padding: 8px 10px;
  font-size: 0.85rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: rgb(255 255 255 / 6%);
  color: inherit;
  cursor: pointer;
}

.btn-tool:hover {
  background: rgb(255 255 255 / 10%);
}
</style>
