<script setup lang="ts">
import { pickExecutableFile } from "@/composables/pickFolder";
import Input from "@/components/ui/Input.vue";
import Button from "@/components/ui/Button.vue";

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
      <Input
        :model-value="gitExecutablePath"
        spellcheck="false"
        autocomplete="off"
        :placeholder="$t('settings.gitExecutablePlaceholder')"
        @update:model-value="emit('updatePath', $event)"
      />
      <Button type="button" size="sm" variant="secondary" @click="pickGitExecutable">
        {{ $t("settings.gitPickExecutable") }}
      </Button>
    </div>
    <div class="git-row">
      <Button type="button" size="sm" variant="secondary" @click="emit('probe')">
        {{ $t("settings.gitProbe") }}
      </Button>
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

.path-row :deep(.ui-control) {
  flex: 1;
  min-width: 12rem;
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
  color: var(--muted-foreground);
}
</style>
