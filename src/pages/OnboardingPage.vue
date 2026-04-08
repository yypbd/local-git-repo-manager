<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { confirmDataRoot, getBootstrap } from "@/composables/bootstrap";
import { pickDirectory } from "@/composables/pickFolder";
import { useToastStore } from "@/stores/toast";
import UiInput from "@/components/ui/UiInput.vue";
import UiSelect from "@/components/ui/UiSelect.vue";
import UiButton from "@/components/ui/UiButton.vue";

type AppSettingsPayload = {
  dataRootPath?: string;
  locale: string;
  gitExecutablePath?: string;
  logMaskSensitive: boolean;
  externalTools: Array<{ id: string; label: string; command: string; argsTemplate: string }>;
};

const router = useRouter();
const { t, locale: appLocale } = useI18n();
const { push: toast } = useToastStore();

const recommendedPath = ref("");
const selectedPath = ref("");
const selectedLocale = ref("ko");
const loading = ref(true);

onMounted(async () => {
  loading.value = true;
  try {
    const bootstrap = await getBootstrap();
    recommendedPath.value = bootstrap?.recommendedDataRoot ?? "";
    selectedPath.value = bootstrap?.confirmedDataRoot ?? bootstrap?.recommendedDataRoot ?? "";

    const s = await invoke<AppSettingsPayload>("get_settings");
    if (s.locale === "ko" || s.locale === "en") {
      selectedLocale.value = s.locale;
    }
  } catch {
    /* ignore */
  } finally {
    loading.value = false;
    if (selectedLocale.value === "ko" || selectedLocale.value === "en") {
      appLocale.value = selectedLocale.value;
    }
  }
});

const syncLocalePreview = () => {
  if (selectedLocale.value === "ko" || selectedLocale.value === "en") {
    appLocale.value = selectedLocale.value;
  }
};

watch(selectedLocale, () => syncLocalePreview());

const completeOnboarding = async () => {
  const path = selectedPath.value.trim();
  if (!path) {
    toast(t("onboarding.confirmFailed"), "error");
    return;
  }

  const confirmed = await confirmDataRoot(path);
  if (!confirmed?.confirmedDataRoot) {
    toast(t("onboarding.confirmFailed"), "error");
    return;
  }

  try {
    const s = await invoke<AppSettingsPayload>("get_settings");
    await invoke("update_settings", {
      next: {
        ...s,
        locale: selectedLocale.value,
        dataRootPath: confirmed.confirmedDataRoot ?? s.dataRootPath,
      },
    });
  } catch (e) {
    toast(e instanceof Error ? e.message : t("onboarding.saveSettingsFailed"), "error");
    return;
  }

  if (selectedLocale.value === "ko" || selectedLocale.value === "en") {
    appLocale.value = selectedLocale.value;
  }

  toast(t("onboarding.success"), "success");
  router.push("/projects");
};

const pickDataRootPath = async () => {
  const picked = await pickDirectory();
  if (!picked) return;
  selectedPath.value = picked;
};
</script>

<template>
  <section class="page">
    <h1>{{ $t("onboarding.title") }}</h1>
    <p class="lead">{{ $t("onboarding.lead") }}</p>

    <div class="field-block">
      <p class="label">{{ $t("onboarding.recommendedCaption") }}</p>
      <code class="path-box">{{ loading ? $t("onboarding.loading") : recommendedPath || "—" }}</code>
    </div>

    <label class="field-block">
      <span class="label">{{ $t("onboarding.selectedCaption") }}</span>
      <div class="path-row">
        <UiInput
          v-model="selectedPath"
          :placeholder="$t('onboarding.pathPlaceholder')"
          spellcheck="false"
          autocomplete="off"
        />
        <UiButton type="button" size="sm" variant="secondary" @click="pickDataRootPath">
          {{ $t("onboarding.pickFolder") }}
        </UiButton>
      </div>
    </label>

    <label class="field-block">
      <span class="label">{{ $t("settings.localeHeading") }}</span>
      <UiSelect v-model="selectedLocale" class="locale-select">
        <option value="ko">{{ $t("settings.localeNameKo") }}</option>
        <option value="en">{{ $t("settings.localeNameEn") }}</option>
      </UiSelect>
    </label>

    <div class="actions">
      <UiButton type="button" size="sm" variant="secondary" @click="selectedPath = recommendedPath">
        {{ $t("onboarding.useRecommended") }}
      </UiButton>
      <UiButton type="button" size="sm" variant="primary" @click="completeOnboarding">
        {{ $t("onboarding.confirm") }}
      </UiButton>
    </div>
  </section>
</template>

<style scoped>
.page {
  max-width: 36rem;
  margin: 0 auto;
  padding: 24px 16px;
  display: grid;
  gap: 18px;
}

.lead {
  margin: 0;
  font-size: 0.95rem;
  line-height: 1.5;
  color: #9ca3af;
}

.field-block {
  display: grid;
  gap: 8px;
}

.label {
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--color-text, #e5e7eb);
}

.path-box {
  display: block;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border, #2f3542);
  background: #151925;
  font-size: 0.8rem;
  word-break: break-all;
  line-height: 1.45;
}

.path-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.path-row :deep(.ui-control) {
  flex: 1;
  min-width: 0;
}

.locale-select {
  max-width: 16rem;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 8px;
}
</style>
