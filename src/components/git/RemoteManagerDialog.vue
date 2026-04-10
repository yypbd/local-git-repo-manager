<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { DialogRoot, DialogClose } from "radix-vue";
import { useToastStore } from "@/stores/toast";
import { repoPathArgs } from "@/utils/tauriRepoPath";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import Input from "@/components/ui/Input.vue";
import Button from "@/components/ui/Button.vue";

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
</script>

<template>
  <DialogRoot :open="true" @update:open="(v: boolean) => { if (!v) emit('close') }">
    <DialogContent class="w-[min(760px,96vw)] max-w-none max-h-[90vh] overflow-auto">
      <DialogHeader>
        <DialogTitle>{{ $t("remote.dialogTitle") }}</DialogTitle>
      </DialogHeader>
      <div class="grid gap-3 px-4 py-3">
        <div v-if="!canUse" class="muted">{{ $t("workspace.actionNeedsFolder") }}</div>
        <div v-else-if="checkingRepo" class="muted">{{ $t("remote.checkingRepo") }}</div>
        <div v-else-if="!isGitRepo" class="muted">{{ $t("remote.notGitRepoRoot") }}</div>
        <div v-else class="grid gap-3">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="muted">{{ $t("remote.quickPreset") }}</span>
            <Button type="button" size="sm" variant="secondary" @click="addName = 'origin'">origin</Button>
            <Button type="button" size="sm" variant="secondary" @click="addName = 'upstream'">upstream</Button>
          </div>
          <div class="grid grid-cols-[1fr_2fr_auto] gap-2 items-center">
            <Input v-model="addName" :placeholder="$t('remote.namePlaceholder')" />
            <Input v-model="addUrl" :placeholder="$t('remote.urlPlaceholder')" />
            <Button type="button" size="sm" variant="default" :disabled="loading || !canManage" @click="addRemote">
              {{ $t("remote.add") }}
            </Button>
          </div>

          <div v-if="loading" class="muted">{{ $t("remote.loading") }}</div>
          <div v-else-if="items.length === 0" class="muted">{{ $t("remote.empty") }}</div>
          <ul v-else class="list-none m-0 p-0 grid gap-3">
            <li v-for="r in items" :key="r.name" class="border border-[var(--color-border)] rounded-lg p-3 grid gap-2">
              <div class="flex justify-between items-center">
                <strong>{{ r.name }}</strong>
                <Button type="button" size="sm" variant="destructive" @click="removeRemote(r.name)">
                  {{ $t("remote.remove") }}
                </Button>
              </div>
              <div class="grid grid-cols-[1fr_auto] gap-2 items-center">
                <Input v-model="renameTo[r.name]" />
                <Button type="button" size="sm" variant="secondary" @click="renameRemote(r.name)">
                  {{ $t("remote.rename") }}
                </Button>
              </div>
              <div class="grid grid-cols-[1fr_auto] gap-2 items-center">
                <Input v-model="setFetchUrl[r.name]" :placeholder="$t('remote.fetchUrl')" />
                <div class="flex gap-2">
                  <Button type="button" size="sm" variant="secondary" @click="setUrl(r.name, false)">
                    {{ $t("remote.updateFetch") }}
                  </Button>
                  <Button type="button" size="sm" variant="secondary" @click="copyText(r.fetchUrl)">
                    {{ $t("remote.copy") }}
                  </Button>
                </div>
              </div>
              <div class="grid grid-cols-[1fr_auto] gap-2 items-center">
                <Input v-model="setPushUrl[r.name]" :placeholder="$t('remote.pushUrl')" />
                <div class="flex gap-2">
                  <Button type="button" size="sm" variant="secondary" @click="setUrl(r.name, true)">
                    {{ $t("remote.updatePush") }}
                  </Button>
                  <Button type="button" size="sm" variant="secondary" @click="copyText(r.pushUrl)">
                    {{ $t("remote.copy") }}
                  </Button>
                </div>
              </div>
            </li>
          </ul>
        </div>
      </div>
      <DialogFooter>
        <DialogClose as-child>
          <Button type="button" size="sm" variant="secondary" @click="emit('close')">
            {{ $t("workspace.close") }}
          </Button>
        </DialogClose>
      </DialogFooter>
    </DialogContent>
  </DialogRoot>
</template>

<style scoped>
.muted {
  opacity: 0.7;
  font-size: 0.85rem;
}
</style>
