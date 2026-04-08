<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToastStore } from "@/stores/toast";
import { useDialogEscape } from "@/composables/useDialogShortcuts";
import { repoPathArgs } from "@/utils/tauriRepoPath";

type RemoteItem = {
  name: string;
  fetchUrl: string | null;
  pushUrl: string | null;
};

const props = defineProps<{ repoPath: string | null }>();
const emit = defineEmits<{ close: []; updated: [] }>();
const { push: toast } = useToastStore();
const { t } = useI18n();

const loading = ref(false);
const checkingRepo = ref(false);
const isGitRepo = ref(false);
const items = ref<RemoteItem[]>([]);
const addName = ref("origin");
const addUrl = ref("");
const renameTo = ref<Record<string, string>>({});
const setFetchUrl = ref<Record<string, string>>({});
const setPushUrl = ref<Record<string, string>>({});

const canUse = computed(() => Boolean(props.repoPath?.trim()));
const canManage = computed(() => canUse.value && isGitRepo.value && !checkingRepo.value);

const verifyRepo = async () => {
  const repo = props.repoPath?.trim();
  if (!repo) {
    isGitRepo.value = false;
    return;
  }
  checkingRepo.value = true;
  try {
    isGitRepo.value = await invoke<boolean>("path_is_git_repo_root", repoPathArgs(repo));
  } catch {
    isGitRepo.value = false;
  } finally {
    checkingRepo.value = false;
  }
};

const load = async () => {
  const repo = props.repoPath?.trim();
  if (!repo) return;
  await verifyRepo();
  if (!isGitRepo.value) {
    items.value = [];
    return;
  }
  loading.value = true;
  try {
    const list = await invoke<RemoteItem[]>("git_remote_list", repoPathArgs(repo));
    items.value = list;
    const nextRename: Record<string, string> = {};
    const nextFetch: Record<string, string> = {};
    const nextPush: Record<string, string> = {};
    for (const r of list) {
      nextRename[r.name] = r.name;
      nextFetch[r.name] = r.fetchUrl ?? "";
      nextPush[r.name] = r.pushUrl ?? "";
    }
    renameTo.value = nextRename;
    setFetchUrl.value = nextFetch;
    setPushUrl.value = nextPush;
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  } finally {
    loading.value = false;
  }
};

const addRemote = async () => {
  const repo = props.repoPath?.trim();
  if (!repo || !canManage.value) return;
  try {
    await invoke("git_remote_add", { ...repoPathArgs(repo), name: addName.value, url: addUrl.value });
    addName.value = "origin";
    addUrl.value = "";
    await load();
    emit("updated");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const removeRemote = async (name: string) => {
  const repo = props.repoPath?.trim();
  if (!repo || !canManage.value) return;
  try {
    await invoke("git_remote_remove", { ...repoPathArgs(repo), name });
    await load();
    emit("updated");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const renameRemote = async (oldName: string) => {
  const repo = props.repoPath?.trim();
  if (!repo || !canManage.value) return;
  try {
    await invoke("git_remote_rename", {
      ...repoPathArgs(repo),
      oldName,
      newName: renameTo.value[oldName] ?? "",
    });
    await load();
    emit("updated");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const setUrl = async (name: string, pushOnly: boolean) => {
  const repo = props.repoPath?.trim();
  if (!repo || !canManage.value) return;
  const url = pushOnly ? setPushUrl.value[name] : setFetchUrl.value[name];
  try {
    await invoke("git_remote_set_url", { ...repoPathArgs(repo), name, url, pushOnly });
    await load();
    emit("updated");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  }
};

const copyText = async (v: string | null | undefined) => {
  if (!v?.trim()) return;
  try {
    await navigator.clipboard.writeText(v);
    toast(t("workspace.copyDone"), "success");
  } catch {
    toast(t("workspace.clipboardCopyFailed"), "error");
  }
};

onMounted(() => {
  void load();
});

useDialogEscape(() => emit("close"));
</script>

<template>
  <Teleport to="body">
    <div class="backdrop modal-backdrop" @click.self="emit('close')">
      <div class="dialog">
        <h3>{{ $t("remote.dialogTitle") }}</h3>
        <div v-if="!canUse" class="muted">{{ $t("workspace.actionNeedsFolder") }}</div>
        <div v-else-if="checkingRepo" class="muted">{{ $t("remote.checkingRepo") }}</div>
        <div v-else-if="!isGitRepo" class="muted">{{ $t("remote.notGitRepoRoot") }}</div>
        <div v-else class="content">
          <div class="preset-row">
            <span class="muted">{{ $t("remote.quickPreset") }}</span>
            <button type="button" @click="addName = 'origin'">origin</button>
            <button type="button" @click="addName = 'upstream'">upstream</button>
          </div>
          <div class="add-row">
            <input v-model="addName" :placeholder="$t('remote.namePlaceholder')" />
            <input v-model="addUrl" :placeholder="$t('remote.urlPlaceholder')" />
            <button :disabled="loading || !canManage" @click="addRemote">{{ $t("remote.add") }}</button>
          </div>

          <div v-if="loading" class="muted">{{ $t("remote.loading") }}</div>
          <div v-else-if="items.length === 0" class="muted">{{ $t("remote.empty") }}</div>
          <ul v-else class="list">
            <li v-for="r in items" :key="r.name" class="item">
              <div class="head">
                <strong>{{ r.name }}</strong>
                <button @click="removeRemote(r.name)">{{ $t("remote.remove") }}</button>
              </div>
              <div class="row">
                <input v-model="renameTo[r.name]" />
                <button @click="renameRemote(r.name)">{{ $t("remote.rename") }}</button>
              </div>
              <div class="row">
                <input v-model="setFetchUrl[r.name]" :placeholder="$t('remote.fetchUrl')" />
                <div class="btns">
                  <button @click="setUrl(r.name, false)">{{ $t("remote.updateFetch") }}</button>
                  <button type="button" @click="copyText(r.fetchUrl)">{{ $t("remote.copy") }}</button>
                </div>
              </div>
              <div class="row">
                <input v-model="setPushUrl[r.name]" :placeholder="$t('remote.pushUrl')" />
                <div class="btns">
                  <button @click="setUrl(r.name, true)">{{ $t("remote.updatePush") }}</button>
                  <button type="button" @click="copyText(r.pushUrl)">{{ $t("remote.copy") }}</button>
                </div>
              </div>
            </li>
          </ul>
        </div>
        <div class="actions">
          <button @click="emit('close')">{{ $t("workspace.close") }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop { position: fixed; inset: 0; z-index: 12000; display: grid; place-items: center; background: rgb(0 0 0 / 45%); }
.dialog { width: min(760px, 96vw); max-height: 90vh; overflow: auto; display: grid; gap: 10px; background: #161b29; border: 1px solid var(--color-border); border-radius: 8px; padding: 12px; }
.content { display: grid; gap: 10px; }
.preset-row { display: flex; align-items: center; gap: 6px; }
.add-row, .row { display: grid; grid-template-columns: 1fr 2fr auto; gap: 6px; }
.row { grid-template-columns: 1fr auto; }
.btns { display: flex; gap: 6px; }
.list { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; }
.item { border: 1px solid var(--color-border); border-radius: 8px; padding: 8px; display: grid; gap: 6px; }
.head { display: flex; justify-content: space-between; align-items: center; }
.muted { opacity: 0.7; font-size: 0.85rem; }
.actions { display: flex; justify-content: flex-end; }
</style>
