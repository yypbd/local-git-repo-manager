<script setup lang="ts">
import { pickExecutableFile } from "@/composables/pickFolder";

defineProps<{ gitExecutablePath: string; gitVersion: string }>();
const emit = defineEmits<{ updatePath: [path: string]; probe: [] }>();

const pickGitExecutable = async () => {
  const picked = await pickExecutableFile();
  if (picked) emit("updatePath", picked);
};
</script>

<template>
  <section class="settings-block">
    <h4>{{ $t("settings.gitHeading") }}</h4>
    <div class="path-row">
      <input
        :value="gitExecutablePath"
        type="text"
        spellcheck="false"
        autocomplete="off"
        :placeholder="$t('settings.gitExecutablePlaceholder')"
        @input="emit('updatePath', ($event.target as HTMLInputElement).value)"
      />
      <button type="button" class="btn btn-sm btn-secondary" @click="pickGitExecutable">
        {{ $t("settings.gitPickExecutable") }}
      </button>
    </div>
    <div class="git-row">
      <button type="button" class="btn btn-sm btn-secondary" @click="emit('probe')">
        {{ $t("settings.gitProbe") }}
      </button>
      <span class="git-ver">{{ gitVersion || $t("settings.gitVersionUnknown") }}</span>
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

.git-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-top: 10px;
}

.git-ver {
  font-size: 0.85rem;
  color: #9ca3af;
}
</style>
