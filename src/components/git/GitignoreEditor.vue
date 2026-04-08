<script setup lang="ts">
import { ref, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useToastStore } from "@/stores/toast";
import UiTextarea from "@/components/ui/UiTextarea.vue";
import UiButton from "@/components/ui/UiButton.vue";

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

/** Esc: 저장 확인이 열려 있으면 확인만 닫음. 에디터 textarea 포커스일 때는 상위 모달을 닫지 않음 */
watchEffect((onCleanup) => {
  const h = (e: KeyboardEvent) => {
    if (e.key !== "Escape") return;
    if (showSaveConfirm.value) {
      e.preventDefault();
      e.stopPropagation();
      showSaveConfirm.value = false;
      return;
    }
    const t = e.target as HTMLElement;
    if (t.tagName === "TEXTAREA" || t.closest("textarea")) return;
    e.preventDefault();
    e.stopPropagation();
    emit("cancel");
  };
  window.addEventListener("keydown", h, true);
  onCleanup(() => window.removeEventListener("keydown", h, true));
});

/** 저장 확인: Enter로 확정 */
watchEffect((onCleanup) => {
  if (!showSaveConfirm.value) return;
  const h = (e: KeyboardEvent) => {
    if (e.key !== "Enter") return;
    e.preventDefault();
    e.stopPropagation();
    confirmSave();
  };
  window.addEventListener("keydown", h, true);
  onCleanup(() => window.removeEventListener("keydown", h, true));
});
</script>

<template>
  <section class="gitignore-editor">
    <h4 v-if="!hideHeading" class="title">{{ $t("workspace.gitignoreEditorTitle") }}</h4>
    <UiTextarea
      v-model="local"
      class="editor"
      rows="14"
      spellcheck="false"
      :placeholder="t('workspace.gitignoreEditorPlaceholder')"
    />
    <div class="actions">
      <UiButton type="button" size="sm" variant="secondary" class="btn-secondary" @click="copyToClipboard">
        {{ $t("workspace.gitignoreCopyButton") }}
      </UiButton>
      <UiButton type="button" size="sm" variant="primary" class="btn-save" @click="requestSave">
        {{ $t("workspace.gitignoreSaveButton") }}
      </UiButton>
    </div>

    <div
      v-if="showSaveConfirm"
      class="confirm-backdrop modal-backdrop"
      @click.self="showSaveConfirm = false"
    >
      <div class="confirm-dialog" @click.stop>
        <h4 class="confirm-title">{{ $t("workspace.gitignoreSaveConfirmTitle") }}</h4>
        <p class="confirm-msg">{{ $t("workspace.gitignoreSaveConfirmMessage") }}</p>
        <div class="confirm-actions">
          <UiButton type="button" size="sm" variant="secondary" class="btn-cancel" @click="showSaveConfirm = false">
            {{ $t("workspace.cancel") }}
          </UiButton>
          <UiButton type="button" size="sm" variant="primary" class="btn-confirm" @click="confirmSave">
            {{ $t("workspace.gitignoreSaveButton") }}
          </UiButton>
        </div>
      </div>
    </div>
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

.btn-secondary {
  padding: 6px 14px;
  font-size: 0.85rem;
  border-radius: 6px;
  border: 1px solid var(--color-border, #3d465c);
  background: rgb(255 255 255 / 6%);
  color: inherit;
  cursor: pointer;
}

.btn-secondary:hover {
  background: rgb(255 255 255 / 10%);
}

.btn-save {
  padding: 6px 14px;
  font-size: 0.85rem;
  border-radius: 6px;
  border: 1px solid #5b7cff;
  background: rgb(91 124 255 / 18%);
  color: #e8ecff;
  cursor: pointer;
}

.btn-save:hover {
  background: rgb(91 124 255 / 28%);
}

.confirm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10060;
  background: rgb(0 0 0 / 50%);
  display: grid;
  place-items: center;
  padding: 16px;
}

.confirm-dialog {
  width: min(400px, 100%);
  padding: 16px;
  border-radius: 8px;
  border: 1px solid var(--color-border, #2a3142);
  background: #111522;
  display: grid;
  gap: 12px;
}

.confirm-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.confirm-msg {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.45;
  color: #9ca3af;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
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

.btn-confirm {
  padding: 6px 12px;
  font-size: 0.85rem;
  border-radius: 6px;
  border: 1px solid #5b7cff;
  background: rgb(91 124 255 / 22%);
  color: #e8ecff;
  cursor: pointer;
}
</style>
