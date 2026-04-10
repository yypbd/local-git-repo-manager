<script setup lang="ts">
import { computed } from "vue";
import { pickDirectory } from "@/composables/pickFolder";
import Input from "@/components/ui/Input.vue";
import Button from "@/components/ui/Button.vue";

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
      <Input
        :model-value="dataRootPath"
        spellcheck="false"
        autocomplete="off"
        @update:model-value="emit('update', $event)"
      />
      <Button type="button" size="sm" variant="secondary" @click="pickFolder">
        {{ $t("settings.dataRootPickFolder") }}
      </Button>
      <Button
        type="button"
        size="sm"
        variant="secondary"
        :disabled="!hasRecommended"
        @click="useRecommended"
      >
        {{ $t("settings.dataRootUseRecommended") }}
      </Button>
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

.path-row :deep(.ui-control) {
  flex: 1;
  min-width: 12rem;
}
</style>
