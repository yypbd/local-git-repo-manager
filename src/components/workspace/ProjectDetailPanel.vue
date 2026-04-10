<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, ref, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { DialogRoot } from "radix-vue";
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
import Button from "@/components/ui/Button.vue";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";

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

/** 폴더 경로만 있으면 시도 가능. `folderRows`에 행이 아직 없을 수 있어 `!row`로 막지 않음. */
const canArchive = computed(() => isSelectedGitRepo.value);
const canRemoteManage = computed(() => isSelectedGitRepo.value);
const canEditGitignore = computed(() => isSelectedGitRepo.value);

const showGitignoreModal = ref(false);
const gitignoreModalContent = ref("");
const templates = ref<Array<{ name: string; content: string }>>([]);
const templateSyncing = ref(false);

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

</script>

<template>
  <div class="detail-panel">
    <div v-if="!project" class="empty">
      {{ $t("workspace.selectProjectHintDetail") }}
    </div>
    <template v-else>
      <div class="actions-footer">
        <div class="action-row action-row--secondary">
          <Button
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!canRemoteManage"
            @click="showRemote = true"
          >
            <span aria-hidden="true">🌐</span>
            {{ $t("workspace.actionRemoteManager") }}
          </Button>
          <Button
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!canArchive"
            @click="showArchive = true"
          >
            <span aria-hidden="true">🗜️</span>
            {{ $t("workspace.actionArchive") }}
          </Button>
          <Button
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!canEditGitignore"
            @click="openGitignoreModal"
          >
            <span aria-hidden="true">✍️</span>
            {{ $t("workspace.editGitignore") }}
          </Button>
        </div>
      </div>
    </template>

    <DialogRoot :open="showGitignoreModal" @update:open="(v) => { if (!v) showGitignoreModal = false }">
      <DialogContent class="w-[min(720px,96vw)] max-h-[min(90vh,720px)] max-w-none flex flex-col">
        <DialogHeader>
          <DialogTitle>{{ $t("workspace.editGitignore") }}</DialogTitle>
        </DialogHeader>
        <div class="min-h-0 flex-1 overflow-auto px-4 py-3 grid gap-3">
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
        </div>
      </DialogContent>
    </DialogRoot>

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
  color: var(--muted-foreground);
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

</style>
