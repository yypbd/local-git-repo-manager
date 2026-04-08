<script setup lang="ts">
defineProps<{ locale: string }>();
const emit = defineEmits<{ update: [locale: string] }>();
</script>

<template>
  <section class="settings-block">
    <h4>{{ $t("settings.localeHeading") }}</h4>
    <select
      class="locale-select"
      :value="locale"
      @change="emit('update', ($event.target as HTMLSelectElement).value)"
    >
      <option value="ko">{{ $t("settings.localeNameKo") }}</option>
      <option value="en">{{ $t("settings.localeNameEn") }}</option>
    </select>
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
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #161b29;
  color: var(--color-text);
  padding: 8px 10px;
  font-size: 0.9rem;
}
</style>
