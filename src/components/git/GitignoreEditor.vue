<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { AlertDialogRoot } from "radix-vue";
import { useToastStore } from "@/stores/toast";
import Textarea from "@/components/ui/Textarea.vue";
import AlertDialogContent from "@/components/ui/AlertDialogContent.vue";
import AlertDialogHeader from "@/components/ui/AlertDialogHeader.vue";
import AlertDialogTitle from "@/components/ui/AlertDialogTitle.vue";
import AlertDialogDescription from "@/components/ui/AlertDialogDescription.vue";
import AlertDialogFooter from "@/components/ui/AlertDialogFooter.vue";
import AlertDialogCancel from "@/components/ui/AlertDialogCancel.vue";
import AlertDialogAction from "@/components/ui/AlertDialogAction.vue";
import Button from "@/components/ui/Button.vue";

const props = withDefaults(
  defineProps<{ content: string; hideHeading?: boolean }>(),
  { hideHeading: false },
);
const emit = defineEmits<{ save: [content: string]; cancel: [] }>();
const { t } = useI18n();
const { push: toast } = useToastStore();

const local = ref(props.content);
const showSaveConfirm = ref(false);

watch(
  () => props.content,
  (next) => {
    local.value = next;
  },
);

const requestSave = () => {
  showSaveConfirm.value = true;
};

const confirmSave = () => {
  showSaveConfirm.value = false;
  emit("save", local.value);
};

const copyToClipboard = async () => {
  const text = local.value ?? "";
  try {
    await navigator.clipboard.writeText(text);
    toast(t("workspace.gitignoreCopyDone"), "success");
  } catch {
    toast(t("workspace.clipboardCopyFailed"), "error");
  }
};

</script>

<template>
  <section class="gitignore-editor">
    <h4 v-if="!hideHeading" class="title">{{ $t("workspace.gitignoreEditorTitle") }}</h4>
    <Textarea
      v-model="local"
      class="editor"
      rows="14"
      spellcheck="false"
      :placeholder="t('workspace.gitignoreEditorPlaceholder')"
    />
    <div class="actions">
      <Button type="button" size="sm" variant="secondary" @click="copyToClipboard">
        {{ $t("workspace.gitignoreCopyButton") }}
      </Button>
      <Button type="button" size="sm" variant="default" @click="requestSave">
        {{ $t("workspace.gitignoreSaveButton") }}
      </Button>
    </div>

    <AlertDialogRoot :open="showSaveConfirm" @update:open="(v) => { if (!v) showSaveConfirm = false }">
      <AlertDialogContent class="max-w-xs">
        <AlertDialogHeader>
          <AlertDialogTitle>{{ $t("workspace.gitignoreSaveConfirmTitle") }}</AlertDialogTitle>
          <AlertDialogDescription>{{ $t("workspace.gitignoreSaveConfirmMessage") }}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="showSaveConfirm = false">{{ $t("workspace.cancel") }}</AlertDialogCancel>
          <AlertDialogAction @click="confirmSave">{{ $t("workspace.gitignoreSaveButton") }}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialogRoot>
  </section>
</template>

<style scoped>
.gitignore-editor {
  display: grid;
  gap: 10px;
  min-width: 0;
  position: relative;
}

.title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.editor {
  width: 100%;
  min-height: 220px;
  box-sizing: border-box;
  padding: 10px 12px;
  font-family: ui-monospace, "Cascadia Code", "SF Mono", Menlo, monospace;
  font-size: 0.82rem;
  line-height: 1.45;
  tab-size: 2;
  background: #0d1018;
  color: var(--color-text, #e5e7eb);
  border: 1px solid var(--color-border, #2a3142);
  border-radius: 8px;
  resize: vertical;
}

.editor::placeholder {
  color: #6b7280;
}

.actions {
  display: flex;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

</style>
