<script setup lang="ts">
import { onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import SettingsDataRootSection from "@/components/settings/SettingsDataRootSection.vue";
import SettingsExternalToolsTable from "@/components/settings/SettingsExternalToolsTable.vue";
import SettingsGitSection from "@/components/settings/SettingsGitSection.vue";
import SettingsLocale from "@/components/settings/SettingsLocale.vue";
import { getBootstrap } from "@/composables/bootstrap";
import { useToastStore } from "@/stores/toast";
import UiButton from "@/components/ui/UiButton.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [open: boolean] }>();

const { t, locale: i18nLocale } = useI18n();
const { push: toast } = useToastStore();

const recommendedDataRoot = ref("");

type SettingsTab = "general" | "external";

const tabList: { id: SettingsTab; labelKey: string }[] = [
  { id: "general", labelKey: "settings.tabGeneral" },
  { id: "external", labelKey: "settings.tabExternalTools" },
];

const activeTab = ref<SettingsTab>("general");
const settings = ref({
  dataRootPath: "",
  locale: "ko",
  gitExecutablePath: "",
  logMaskSensitive: true,
  externalTools: [] as Array<{ id: string; label: string; command: string; argsTemplate: string }>,
});
const gitVersion = ref("");

const load = async () => {
  try {
    const got = await invoke<typeof settings.value>("get_settings");
    settings.value = {
      ...got,
      dataRootPath: got.dataRootPath ?? "",
      gitExecutablePath: got.gitExecutablePath ?? "",
    };
  } catch {}
  try {
    const b = await getBootstrap();
    recommendedDataRoot.value = b?.recommendedDataRoot ?? "";
  } catch {
    recommendedDataRoot.value = "";
  }
};

function applyUiLocale(loc: string) {
  if (loc === "ko" || loc === "en") {
    i18nLocale.value = loc;
  }
}

const onLocaleChange = (loc: string) => {
  settings.value.locale = loc;
};

const save = async () => {
  try {
    const saved = await invoke<typeof settings.value>("update_settings", { next: settings.value });
    settings.value = {
      ...saved,
      dataRootPath: saved.dataRootPath ?? "",
      gitExecutablePath: saved.gitExecutablePath ?? "",
    };
    applyUiLocale(saved.locale);
    toast(t("settings.saveSuccess"), "success");
  } catch (e) {
    toast(e instanceof Error ? e.message : t("settings.saveFailed"), "error");
  }
};

const probeGit = async () => {
  try {
    gitVersion.value = await invoke<string>("git_probe_version");
  } catch {
    gitVersion.value = t("settings.gitVersionNotFound");
  }
};

const addTool = () =>
  settings.value.externalTools.push({
    id: Math.random().toString(36).slice(2, 10),
    label: "",
    command: "",
    argsTemplate: "$PATH",
  });

const removeTool = (id: string) => {
  settings.value.externalTools = settings.value.externalTools.filter((tool) => tool.id !== id);
};

const patchTool = (
  id: string,
  partial: Partial<{ label: string; command: string; argsTemplate: string }>,
) => {
  const i = settings.value.externalTools.findIndex((tool) => tool.id === id);
  if (i < 0) return;
  const cur = settings.value.externalTools[i]!;
  settings.value.externalTools[i] = { ...cur, ...partial };
};

const close = () => {
  emit("update:modelValue", false);
};

let escHandler: ((e: KeyboardEvent) => void) | null = null;

watch(
  () => props.modelValue,
  (open) => {
    if (escHandler) {
      window.removeEventListener("keydown", escHandler, true);
      escHandler = null;
    }
    if (open) {
      activeTab.value = "general";
      void load();
      escHandler = (e: KeyboardEvent) => {
        if (e.key !== "Escape") return;
        e.preventDefault();
        e.stopPropagation();
        close();
      };
      window.addEventListener("keydown", escHandler, true);
    }
  },
);

onUnmounted(() => {
  if (escHandler) window.removeEventListener("keydown", escHandler, true);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="backdrop modal-backdrop settings-dlg-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="$t('settings.title')"
      @click.self="close"
    >
      <div class="settings-dialog" @click.stop>
        <header class="dialog-head">
          <h2 class="dialog-title">{{ $t("settings.title") }}</h2>
        </header>

        <div class="dialog-scroll">
          <div class="tab-buttons">
            <UiButton
              v-for="tab in tabList"
              :key="tab.id"
              type="button"
              size="sm"
              variant="secondary"
              :class="{ active: activeTab === tab.id }"
              @click="activeTab = tab.id"
            >
              {{ $t(tab.labelKey) }}
            </UiButton>
          </div>

          <div v-if="activeTab === 'general'" class="general-tab">
            <SettingsDataRootSection
              :data-root-path="settings.dataRootPath"
              :recommended-path="recommendedDataRoot"
              @update="(path) => (settings.dataRootPath = path)"
            />
            <SettingsLocale :locale="settings.locale" @update="onLocaleChange" />
            <SettingsGitSection
              :git-executable-path="settings.gitExecutablePath"
              :git-version="gitVersion"
              @update-path="(path) => (settings.gitExecutablePath = path)"
              @probe="probeGit"
            />
          </div>

          <SettingsExternalToolsTable
            v-if="activeTab === 'external'"
            :tools="settings.externalTools"
            @add="addTool"
            @remove="removeTool"
            @patch="patchTool"
          />
        </div>

        <footer class="dialog-foot">
          <UiButton type="button" variant="secondary" @click="close">
            {{ $t("workspace.close") }}
          </UiButton>
          <UiButton type="button" variant="primary" @click="save">{{ $t("settings.save") }}</UiButton>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
/**
 * 전역 `.modal-backdrop`에는 z-index만 있고 position이 없음.
 * Teleport된 루트에 fixed + inset을 반드시 줘야 뷰포트를 덮고 본문 스크롤을 유발하지 않음.
 */
.settings-dlg-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10050;
  isolation: isolate;
  background: rgb(0 0 0 / 45%);
  padding: 24px 16px;
  display: grid;
  place-items: center;
  box-sizing: border-box;
  overflow-y: auto;
}

.settings-dialog {
  width: min(44rem, 100%);
  max-height: min(88vh, 900px);
  display: grid;
  grid-template-rows: auto 1fr auto;
  min-height: 0;
  background: var(--color-surface-muted, #151925);
  border: 1px solid var(--color-border, #2f3542);
  border-radius: 10px;
  box-shadow: 0 16px 48px rgb(0 0 0 / 45%);
}

.dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border, #2f3542);
  flex-shrink: 0;
}

.dialog-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
}

.dialog-scroll {
  min-height: 0;
  overflow: auto;
  padding: 16px;
}

.dialog-foot {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border, #2f3542);
  flex-shrink: 0;
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.tab-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 16px;
}

.tab-buttons .active {
  border-color: #7aa2ff;
}

.general-tab {
  display: grid;
  gap: 24px;
  max-width: 42rem;
}

.general-tab :deep(.ui-control) {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
}
</style>
