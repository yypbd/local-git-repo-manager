<script setup lang="ts">
import Select from "@/components/ui/Select.vue";
defineProps<{ locale: string }>();
const emit = defineEmits<{ update: [locale: string] }>();
</script>

<template>
  <section class="settings-block">
    <h4>{{ $t("settings.localeHeading") }}</h4>
    <Select
      class="locale-select"
      :model-value="locale"
      @update:modelValue="emit('update', $event)"
    >
      <option value="ko">{{ $t("settings.localeNameKo") }}</option>
      <option value="en">{{ $t("settings.localeNameEn") }}</option>
    </Select>
  </section>
</template>

<style scoped>
.settings-block h4 {
  margin: 0 0 8px;
  font-size: 0.95rem;
  font-weight: 600;
}

.locale-select {
  max-width: 18rem;
  width: 100%;
}
</style>
