<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { DialogRoot, DialogClose } from "radix-vue";
import { pickDirectory } from "@/composables/pickFolder";
import { useToastStore } from "@/stores/toast";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import { parentDirectory } from "@/utils/pathParent";

const props = defineProps<{
  projectId: string;
  /** 목록에서 폴더 미선택 시: 첫 루트 경로의 부모를 기본으로 사용 */
  firstRootPath?: string;
  /** 목록에서 폴더 선택 시: 이 경로의 부모를 기본으로 사용 (`firstRootPath`보다 우선) */
  selectedFolderPath?: string | null;
  /** 부모(툴바)에서 클립보드로 미리 읽은 Git 원격 URL */
  initialRemoteUrl?: string;
}>();
const emit = defineEmits<{ close: []; done: [] }>();

const { t } = useI18n();
const { push: toast } = useToastStore();

const remoteUrl = ref("");
const localPath = ref("");
const cloning = ref(false);
const urlInputRef = ref<{ focus?: () => void } | null>(null);

/** 선택/첫 루트 기준 부모 폴더만 (복제 시 그 안에 저장소 폴더가 생김) */
function defaultParentFolderPath(): string {
  const selected = props.selectedFolderPath?.trim();
  const anchor = selected || props.firstRootPath?.trim() || "";
  return anchor ? parentDirectory(anchor) : "";
}

onMounted(() => {
  const initial = props.initialRemoteUrl?.trim();
  if (initial) {
    remoteUrl.value = initial;
  }
  const next = defaultParentFolderPath();
  if (next) localPath.value = next;
  void nextTick(() => urlInputRef.value?.focus?.());
});

const clearRemoteUrl = () => {
  remoteUrl.value = "";
};

const clearLocalPath = () => {
  localPath.value = "";
};

const pickParentFolder = async () => {
  const picked = await pickDirectory();
  if (!picked?.trim()) return;
  localPath.value = picked.trim();
};

const canSubmit = computed(() => {
  return Boolean(props.projectId.trim() && remoteUrl.value.trim() && localPath.value.trim() && !cloning.value);
});

const submit = async () => {
  if (!canSubmit.value) return;
  cloning.value = true;
  try {
    await invoke("projects_git_clone", {
      projectId: props.projectId,
      remoteUrl: remoteUrl.value.trim(),
      localPath: localPath.value.trim(),
    });
    toast(t("workspace.cloneSuccess"), "success");
    emit("done");
    emit("close");
  } catch (e) {
    toast(e instanceof Error ? e.message : String(e), "error");
  } finally {
    cloning.value = false;
  }
};
</script>

<template>
  <DialogRoot :open="true" @update:open="(v: boolean) => { if (!v) emit('close') }">
    <DialogContent class="max-w-md">
      <DialogHeader>
        <DialogTitle>{{ $t("workspace.cloneTitle") }}</DialogTitle>
      </DialogHeader>
      <form class="grid gap-3 px-4 py-3" @submit.prevent="submit">
        <p class="text-sm text-[var(--muted-foreground)] leading-snug m-0">
          {{ $t("workspace.cloneLead") }}
        </p>
        <label class="grid gap-1">
          <span class="text-xs text-[var(--muted-foreground)]">{{ $t("workspace.cloneRemoteUrl") }}</span>
          <div class="flex items-center gap-1 min-w-0">
            <Input
              ref="urlInputRef"
              v-model="remoteUrl"
              class="min-w-0 flex-1"
              type="url"
              size="sm"
              autocomplete="off"
              :placeholder="$t('workspace.cloneRemotePlaceholder')"
            />
            <Button
              v-if="remoteUrl.trim()"
              type="button"
              size="sm"
              variant="ghost"
              class="shrink-0 h-8 min-h-8 w-8 px-0"
              :aria-label="$t('workspace.cloneClear')"
              @click="clearRemoteUrl"
            >
              <span aria-hidden="true" class="text-lg leading-none">×</span>
            </Button>
          </div>
        </label>
        <label class="grid gap-1">
          <span class="text-xs text-[var(--muted-foreground)]">{{ $t("workspace.cloneLocalPath") }}</span>
          <div class="flex items-center gap-1 min-w-0">
            <Input
              v-model="localPath"
              class="min-w-0 flex-1"
              type="text"
              size="sm"
              autocomplete="off"
              :placeholder="$t('workspace.cloneLocalPlaceholder')"
            />
            <Button
              v-if="localPath.trim()"
              type="button"
              size="sm"
              variant="ghost"
              class="shrink-0 h-8 min-h-8 w-8 px-0"
              :aria-label="$t('workspace.cloneClear')"
              @click="clearLocalPath"
            >
              <span aria-hidden="true" class="text-lg leading-none">×</span>
            </Button>
          </div>
        </label>
        <div class="flex flex-wrap gap-2">
          <Button type="button" size="sm" variant="secondary" @click="pickParentFolder">
            {{ $t("workspace.clonePickParent") }}
          </Button>
        </div>
        <DialogFooter class="border-0 p-0 pt-1">
          <DialogClose as-child>
            <Button type="button" size="sm" variant="secondary" @click="emit('close')">
              {{ $t("workspace.cancel") }}
            </Button>
          </DialogClose>
          <Button type="submit" size="sm" variant="default" :disabled="!canSubmit">
            {{ cloning ? $t("workspace.cloneRunning") : $t("workspace.cloneSubmit") }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </DialogRoot>
</template>
