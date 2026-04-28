<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { DialogRoot } from "radix-vue";
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "@/components/ui/Tabs/index";
import SettingsExternalToolsTable from "@/components/settings/SettingsExternalToolsTable.vue";
import SettingsGitSection from "@/components/settings/SettingsGitSection.vue";
import SettingsLocale from "@/components/settings/SettingsLocale.vue";
import { useToastStore } from "@/stores/toast";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import Button from "@/components/ui/Button.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [open: boolean] }>();

const { t, locale: i18nLocale } = useI18n();
const { push: toast } = useToastStore();

type SettingsTab = "general" | "external";

const tabList: { id: SettingsTab; labelKey: string }[] = [
  { id: "general", labelKey: "settings.tabGeneral" },
  { id: "external", labelKey: "settings.tabExternalTools" },
];

const activeTab = ref<SettingsTab>("general");
const settings = ref({
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
      gitExecutablePath: got.gitExecutablePath ?? "",
    };
  } catch {}
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

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      activeTab.value = "general";
      void load();
    }
  },
);
</script>

<template>
  <DialogRoot :open="modelValue" @update:open="(v: boolean) => { if (!v) close() }">
    <DialogContent
      class="w-[min(44rem,100%)] max-h-[min(88vh,900px)] flex flex-col max-w-none"
      :aria-label="$t('settings.title')"
    >
      <!-- Header -->
      <DialogHeader>
        <DialogTitle>{{ $t("settings.title") }}</DialogTitle>
      </DialogHeader>

      <!-- Scrollable body with tabs -->
      <div class="min-h-0 flex-1 overflow-auto p-4">
        <TabsRoot :model-value="activeTab" @update:model-value="(v) => activeTab = v as SettingsTab">
          <TabsList class="flex flex-wrap gap-2 mb-4 border-b border-[var(--border)] pb-3">
            <TabsTrigger
              v-for="tab in tabList"
              :key="tab.id"
              :value="tab.id"
              class="btn btn-sm btn-secondary rounded-sm px-3 text-xs border border-[#4a5568] bg-[#1a2029] text-[var(--foreground)] cursor-pointer transition-colors data-[state=active]:border-[#7aa2ff] data-[state=active]:bg-[#1a2440] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ring)]"
            >
              {{ $t(tab.labelKey) }}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="general" class="grid gap-6 max-w-[42rem] focus-visible:outline-none">
            <SettingsLocale :locale="settings.locale" @update="onLocaleChange" />
            <SettingsGitSection
              :git-executable-path="settings.gitExecutablePath"
              :git-version="gitVersion"
              @update-path="(path) => (settings.gitExecutablePath = path)"
              @probe="probeGit"
            />
          </TabsContent>

          <TabsContent value="external" class="focus-visible:outline-none">
            <SettingsExternalToolsTable
              :tools="settings.externalTools"
              @add="addTool"
              @remove="removeTool"
              @patch="patchTool"
            />
          </TabsContent>
        </TabsRoot>
      </div>

      <!-- Footer -->
      <DialogFooter>
        <Button type="button" variant="secondary" @click="close">
          {{ $t("workspace.close") }}
        </Button>
        <Button type="button" variant="default" @click="save">
          {{ $t("settings.save") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </DialogRoot>
</template>

<style scoped>
/* Preserve deep selector for settings sub-components */
:deep(.ui-control) {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
}
</style>
