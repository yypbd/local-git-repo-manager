<script setup lang="ts">
import type { FolderRootRow } from "@/composables/useFolderRootRows";
import { folderNameFromPath } from "@/composables/useFolderRootRows";
import WorkingTreeStatusLabel from "@/components/workspace/WorkingTreeStatusLabel.vue";
import GitStatusPathList from "@/components/workspace/GitStatusPathList.vue";
import { invoke } from "@tauri-apps/api/core";
import { repoPathArgs } from "@/utils/tauriRepoPath";
import { computed, ref, watch, toRefs } from "vue";
import UiButton from "@/components/ui/UiButton.vue";

const props = defineProps<{
  path: string | null;
  row: FolderRootRow | undefined;
  loading: boolean;
}>();
const { path, row, loading } = toRefs(props);

type GitStatusFilesPayload = {
  branch: string;
  upstream: string | null;
  clean: boolean;
  changedOrAdded: string[];
  untracked: string[];
};

const filesLoading = ref(false);
const changedOrAddedFiles = ref<string[]>([]);
const untrackedFiles = ref<string[]>([]);
/** `git_status_files`와 동일 출처의 현재 브랜치명 */
const branchFromStatusFiles = ref<string | null>(null);
/** `git rev-parse @{u}` — 예: origin/main */
const upstreamFromStatusFiles = ref<string | null>(null);

const reloadFiles = async () => {
  if (!props.path) {
    changedOrAddedFiles.value = [];
    untrackedFiles.value = [];
    branchFromStatusFiles.value = null;
    upstreamFromStatusFiles.value = null;
    return;
  }
  filesLoading.value = true;
  branchFromStatusFiles.value = null;
  upstreamFromStatusFiles.value = null;
  try {
    const payload = await invoke<GitStatusFilesPayload>("git_status_files", repoPathArgs(props.path));
    changedOrAddedFiles.value = payload.changedOrAdded ?? [];
    untrackedFiles.value = payload.untracked ?? [];
    branchFromStatusFiles.value = payload.branch?.trim() || null;
    const u = payload.upstream?.trim();
    upstreamFromStatusFiles.value = u || null;
  } catch {
    changedOrAddedFiles.value = [];
    untrackedFiles.value = [];
    branchFromStatusFiles.value = null;
    upstreamFromStatusFiles.value = null;
  } finally {
    filesLoading.value = false;
  }
};

const gitignoreLoading = ref(false);
const gitignoreExists = ref(false);
const gitignoreText = ref("");

const reloadGitignore = async () => {
  if (!props.path) {
    gitignoreExists.value = false;
    gitignoreText.value = "";
    return;
  }
  gitignoreLoading.value = true;
  try {
    const res = await invoke<{ exists: boolean; content: string }>("read_gitignore", repoPathArgs(props.path));
    gitignoreExists.value = res.exists;
    gitignoreText.value = res.content ?? "";
  } catch {
    gitignoreExists.value = false;
    gitignoreText.value = "";
  } finally {
    gitignoreLoading.value = false;
  }
};

watch(
  () => props.path,
  () => {
    void reloadFiles();
    void reloadGitignore();
  },
  { immediate: true },
);

type DetailTabId = "default" | "branch" | "changed" | "untracked" | "gitignore";
const activeTab = ref<DetailTabId>("default");

/** 브랜치 탭 표시용: 상태 파일 조회 결과 우선, 없으면 폴더 목록 행 */
const displayBranchName = computed(() => {
  if (!path.value || !row.value || row.value.gitError) return "";
  if (filesLoading.value) return row.value.branch || "—";
  return branchFromStatusFiles.value ?? row.value.branch ?? "—";
});
</script>

<template>
  <div class="detail">
    <div class="tabs-wrap">
      <div class="tabs" role="tablist">
        <UiButton
          type="button"
          class="tab"
          size="sm"
          variant="secondary"
          role="tab"
          :aria-selected="activeTab === 'default'"
          @click="activeTab = 'default'"
        >
          <span class="tab-ico" aria-hidden="true">ℹ️</span>
          {{ $t("workspace.folderDetailTabDefault") }}
        </UiButton>
        <UiButton
          type="button"
          class="tab"
          size="sm"
          variant="secondary"
          role="tab"
          :aria-selected="activeTab === 'branch'"
          @click="activeTab = 'branch'"
        >
          <span class="tab-ico" aria-hidden="true">🌿</span>
          {{ $t("workspace.folderDetailTabBranch") }}
        </UiButton>
        <UiButton
          type="button"
          class="tab"
          size="sm"
          variant="secondary"
          role="tab"
          :aria-selected="activeTab === 'changed'"
          @click="activeTab = 'changed'"
        >
          <span class="tab-ico" aria-hidden="true">📝</span>
          {{ $t("workspace.folderDetailTabChangedAdded") }}
        </UiButton>
        <UiButton
          type="button"
          class="tab"
          size="sm"
          variant="secondary"
          role="tab"
          :aria-selected="activeTab === 'untracked'"
          @click="activeTab = 'untracked'"
        >
          <span class="tab-ico" aria-hidden="true">📌</span>
          {{ $t("workspace.folderDetailTabUntracked") }}
        </UiButton>
        <UiButton
          type="button"
          class="tab"
          size="sm"
          variant="secondary"
          role="tab"
          :aria-selected="activeTab === 'gitignore'"
          @click="activeTab = 'gitignore'"
        >
          <span class="tab-ico" aria-hidden="true">🚫</span>
          {{ $t("workspace.folderDetailTabGitignore") }}
        </UiButton>
      </div>
    </div>

    <div
      v-show="activeTab === 'default'"
      class="tab-panel"
      role="tabpanel"
    >
      <div v-if="!path" class="empty">{{ $t("workspace.folderSelectHint") }}</div>
      <template v-else>
        <dl class="fields">
          <div class="field">
            <dt>{{ $t("workspace.folderName") }}</dt>
            <dd>{{ folderNameFromPath(path) }}</dd>
          </div>
          <div class="field">
            <dt>{{ $t("workspace.localPath") }}</dt>
            <dd><code class="mono">{{ path }}</code></dd>
          </div>
          <div class="field">
            <dt>{{ $t("workspace.remotePath") }}</dt>
            <dd>
              <template v-if="loading">…</template>
              <span v-else-if="row?.gitError" class="muted">{{ $t("workspace.remoteNotGit") }}</span>
              <span v-else-if="row?.remote">
                <code class="mono">{{ row.remote }}</code>
                <small
                  v-if="(row.remoteCount ?? 0) > 1"
                  class="remote-more muted"
                >
                  {{ $t("workspace.remoteMoreCount", { count: row.remoteCount - 1 }) }}
                </small>
              </span>
              <span v-else class="muted">{{ $t("workspace.remoteNoOrigin") }}</span>
            </dd>
          </div>
          <div class="field">
            <dt>{{ $t("workspace.statusLabel") }}</dt>
            <dd>
              <template v-if="loading">…</template>
              <template v-else-if="!row">—</template>
              <template v-else-if="row.gitError">{{ $t("workspace.notGitRepo") }}</template>
              <template v-else-if="row.clean">{{ $t("workspace.statusClean") }}</template>
              <template v-else>
                <WorkingTreeStatusLabel
                  :tracked-changes="row.trackedChanges"
                  :untracked-files="row.untrackedFiles"
                />
              </template>
            </dd>
          </div>
        </dl>
      </template>
    </div>

    <div
      v-show="activeTab === 'branch'"
      class="tab-panel"
      role="tabpanel"
    >
      <div v-if="!path" class="empty">{{ $t("workspace.folderSelectHint") }}</div>
      <template v-else>
        <div v-if="loading" class="files-empty">…</div>
        <p v-else-if="row?.gitError" class="empty muted">{{ $t("workspace.notGitRepo") }}</p>
        <dl v-else-if="row" class="fields branch-fields">
          <div class="field">
            <dt>{{ $t("workspace.branchTabCurrent") }}</dt>
            <dd>
              <code class="branch-highlight">{{ displayBranchName }}</code>
            </dd>
          </div>
          <div class="field">
            <dt>{{ $t("workspace.branchTabUpstream") }}</dt>
            <dd>
              <template v-if="filesLoading">…</template>
              <code v-else-if="upstreamFromStatusFiles" class="branch-highlight">{{ upstreamFromStatusFiles }}</code>
              <span v-else class="muted">{{ $t("workspace.branchTabNoUpstream") }}</span>
            </dd>
          </div>
        </dl>
        <div v-else class="empty">—</div>
      </template>
    </div>

    <div
      v-show="activeTab === 'changed'"
      class="tab-panel"
      role="tabpanel"
    >
      <div v-if="!path" class="empty">{{ $t("workspace.folderSelectHint") }}</div>
      <template v-else>
        <GitStatusPathList
          :paths="changedOrAddedFiles"
          :pending="loading || filesLoading"
          :empty-text="$t('workspace.changedAddedListEmpty')"
        />
      </template>
    </div>

    <div
      v-show="activeTab === 'untracked'"
      class="tab-panel"
      role="tabpanel"
    >
      <div v-if="!path" class="empty">{{ $t("workspace.folderSelectHint") }}</div>
      <template v-else>
        <GitStatusPathList
          :paths="untrackedFiles"
          :pending="loading || filesLoading"
          :empty-text="$t('workspace.untrackedListEmpty')"
        />
      </template>
    </div>

    <div
      v-show="activeTab === 'gitignore'"
      class="tab-panel tab-panel--gitignore"
      role="tabpanel"
    >
      <div v-if="!path" class="empty">{{ $t("workspace.folderSelectHint") }}</div>
      <template v-else>
        <div v-if="gitignoreLoading" class="files-empty">…</div>
        <p v-else-if="!gitignoreExists" class="empty muted">{{ $t("workspace.gitignoreTabNoFile") }}</p>
        <pre v-else class="gitignore-pre">{{ gitignoreText }}</pre>
      </template>
    </div>
  </div>
</template>

<style scoped>
.detail {
  flex: 1;
  min-height: 0;
  border-top: 1px solid var(--color-border);
  padding: 8px 0 0;
  margin-top: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tabs-wrap {
  flex-shrink: 0;
  min-height: 0;
  max-width: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  border-bottom: 1px solid var(--color-border);
  -webkit-overflow-scrolling: touch;
  scrollbar-width: thin;
}

.tabs {
  display: flex;
  flex-wrap: nowrap;
  gap: 0;
  width: max-content;
  min-width: 100%;
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin: 0;
  padding: 6px 10px;
  font: inherit;
  font-size: 0.78rem;
  border: none;
  background: transparent;
  color: inherit;
  opacity: 0.65;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  white-space: nowrap;
  flex-shrink: 0;
}

.tab-ico {
  line-height: 1;
  font-size: 0.88em;
}

.tab:hover {
  opacity: 0.9;
}

.tab[aria-selected="true"] {
  opacity: 1;
  font-weight: 600;
  border-bottom-color: var(--color-accent, #60a5fa);
}

.tab-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: auto;
}

.empty {
  margin: 0;
  font-size: 0.85rem;
  color: #9ca3af;
  line-height: 1.4;
}

.fields {
  margin: 0;
  display: grid;
  gap: 8px;
}

.field {
  display: grid;
  gap: 2px;
}

dt {
  margin: 0;
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  opacity: 0.7;
}

dd {
  margin: 0;
  font-size: 0.82rem;
  line-height: 1.35;
}

.mono {
  word-break: break-all;
  font-size: 0.78rem;
}

.remote-more {
  margin-left: 6px;
  font-size: 0.7rem;
}

.muted {
  opacity: 0.65;
  font-style: italic;
}

.files-empty {
  font-size: 0.8rem;
  opacity: 0.75;
}

.branch-fields {
  margin: 0;
}

.branch-highlight {
  font-family: ui-monospace, monospace;
  font-size: 0.9rem;
  font-weight: 600;
  word-break: break-all;
}

.tab-panel--gitignore {
  min-height: 0;
}

.gitignore-pre {
  margin: 0;
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 8px 10px;
  font-family: ui-monospace, monospace;
  font-size: 0.72rem;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
  background: rgb(0 0 0 / 20%);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: inherit;
}
</style>
