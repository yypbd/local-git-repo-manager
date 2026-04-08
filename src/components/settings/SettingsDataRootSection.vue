<script setup lang="ts">
import { computed } from "vue";
import { pickDirectory } from "@/composables/pickFolder";

const props = defineProps<{
  dataRootPath: string;
  /** `get_bootstrap().recommendedDataRoot` — 비어 있으면 권장 버튼 비활성 */
  recommendedPath: string;
}>();

const emit = defineEmits<{ update: [path: string] }>();

const hasRecommended = computed(() => Boolean(props.recommendedPath?.trim()));

const pickFolder = async () => {
  const picked = await pickDirectory();
  if (picked) emit("update", picked);
};

const useRecommended = () => {
  const r = props.recommendedPath?.trim();
  if (r) emit("update", r);
};
</script>

<template>
  <section class="settings-block">
    <h4>{{ $t("settings.dataRootHeading") }}</h4>
    <div class="path-row">
      <input
        :value="dataRootPath"
        type="text"
        spellcheck="false"
        autocomplete="off"
        @input="emit('update', ($event.target as HTMLInputElement).value)"
      />
      <button type="button" class="btn btn-sm btn-secondary" @click="pickFolder">
        {{ $t("settings.dataRootPickFolder") }}
      </button>
      <button
        type="button"
        class="btn btn-sm btn-secondary"
        :disabled="!hasRecommended"
        @click="useRecommended"
      >
        {{ $t("settings.dataRootUseRecommended") }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.settings-block h4 {
  margin: 0 0 8px;
  font-size: 0.95rem;
  font-weight: 600;
}

.path-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: stretch;
}

.path-row input {
  flex: 1;
  min-width: 12rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #161b29;
  color: var(--color-text);
  padding: 8px 10px;
  font-size: 0.9rem;
  box-sizing: border-box;
}
</style>
