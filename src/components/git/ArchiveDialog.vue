<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { DialogRoot, DialogClose } from "radix-vue";
import { pickDirectory } from "@/composables/pickFolder";
import { useToastStore } from "@/stores/toast";
import { outputParentDirArgs, repoPathArgs } from "@/utils/tauriRepoPath";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import Button from "@/components/ui/Button.vue";

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
</script>

<template>
  <DialogRoot :open="true" @update:open="(v: boolean) => { if (!v) emit('close') }">
    <DialogContent class="max-w-sm">
      <DialogHeader>
        <DialogTitle>{{ $t("workspace.archiveTitle") }}</DialogTitle>
      </DialogHeader>
      <div class="grid gap-2 px-4 py-3">
        <p class="text-sm text-[var(--muted-foreground)] leading-snug m-0">
          {{ $t("workspace.archiveBody") }}
        </p>
        <p class="font-mono text-xs text-[#6b7280] leading-snug m-0">
          {{ $t("workspace.archiveFilenameHint") }}
        </p>
      </div>
      <DialogFooter>
        <DialogClose as-child>
          <Button type="button" size="sm" variant="secondary" @click="emit('close')">
            {{ $t("workspace.cancel") }}
          </Button>
        </DialogClose>
        <Button
          type="button"
          size="sm"
          variant="default"
          :disabled="!canExport || exporting"
          @click="pickAndExport"
        >
          {{ exporting ? $t("workspace.archiveExporting") : $t("workspace.archivePickExport") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </DialogRoot>
</template>
