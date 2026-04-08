<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { pickExecutableFile } from "@/composables/pickFolder";
import { executableDisplayName } from "@/utils/pathBasename";

type Tool = { id: string; label: string; command: string; argsTemplate: string };

defineProps<{ tools: Tool[] }>();

const emit = defineEmits<{
  add: [];
  remove: [id: string];
  patch: [id: string, partial: Partial<Pick<Tool, "label" | "command" | "argsTemplate">>];
}>();

const { t } = useI18n();

const onPatch = (
  id: string,
  field: keyof Pick<Tool, "label" | "command" | "argsTemplate">,
  value: string,
) => {
  emit("patch", id, { [field]: value });
};

const browseExecutable = async (tool: Tool) => {
  const picked = await pickExecutableFile();
  if (!picked) return;
  const partial: Partial<Pick<Tool, "label" | "command">> = { command: picked };
  if (!tool.label?.trim()) {
    partial.label = executableDisplayName(picked);
  }
  emit("patch", tool.id, partial);
};
</script>

<template>
  <section class="external-tools">
    <div class="section-head">
      <h4>{{ $t("settings.externalToolsHeading") }}</h4>
      <p class="hint">{{ $t("settings.externalToolsHint") }}</p>
    </div>

    <div v-if="!tools.length" class="empty-tools">
      <p class="empty-text">{{ $t("settings.externalToolsEmpty") }}</p>
      <button type="button" class="btn-primary" @click="emit('add')">
        {{ $t("settings.externalToolsAdd") }}
      </button>
    </div>

    <template v-else>
      <ul class="tool-list">
        <li v-for="(tool, index) in tools" :key="tool.id" class="tool-row">
          <div class="tool-index">{{ $t("settings.externalToolsSlot", { n: index + 1 }) }}</div>
          <div class="fields">
            <label class="field">
              <span>{{ $t("settings.externalToolsLabel") }}</span>
              <input
                type="text"
                :value="tool.label"
                :placeholder="t('settings.externalToolsLabelPh')"
                @input="onPatch(tool.id, 'label', ($event.target as HTMLInputElement).value)"
              />
            </label>
            <label class="field">
              <span>{{ $t("settings.externalToolsCommand") }}</span>
              <div class="command-row">
                <input
                  type="text"
                  class="command-input"
                  :value="tool.command"
                  :placeholder="t('settings.externalToolsCommandPh')"
                  @input="onPatch(tool.id, 'command', ($event.target as HTMLInputElement).value)"
                />
                <button type="button" class="btn-browse" @click="browseExecutable(tool)">
                  {{ $t("settings.externalToolsBrowse") }}
                </button>
              </div>
            </label>
            <label class="field field-wide">
              <span>{{ $t("settings.externalToolsArgs") }}</span>
              <input
                type="text"
                :value="tool.argsTemplate"
                placeholder="$PATH"
                @input="onPatch(tool.id, 'argsTemplate', ($event.target as HTMLInputElement).value)"
              />
            </label>
          </div>
          <button type="button" class="btn-remove" @click="emit('remove', tool.id)">
            {{ $t("settings.externalToolsRemove") }}
          </button>
        </li>
      </ul>
      <button type="button" class="btn-add-more" @click="emit('add')">
        {{ $t("settings.externalToolsAddAnother") }}
      </button>
    </template>
  </section>
</template>

<style scoped>
.external-tools {
  display: grid;
  gap: 14px;
  max-width: 720px;
}

.section-head {
  display: grid;
  gap: 8px;
}

.hint {
  margin: 0;
  font-size: 0.85rem;
  color: #9ca3af;
  line-height: 1.45;
}

.empty-tools {
  padding: 28px 20px;
  border: 1px dashed var(--color-border, #3d465c);
  border-radius: 10px;
  display: grid;
  gap: 14px;
  justify-items: center;
  text-align: center;
  background: rgb(255 255 255 / 3%);
}

.empty-text {
  margin: 0;
  font-size: 0.9rem;
  color: #9ca3af;
  max-width: 28rem;
  line-height: 1.5;
}

.btn-primary {
  padding: 8px 16px;
  font-size: 0.9rem;
  border-radius: 8px;
  border: 1px solid #5b7cff;
  background: rgb(91 124 255 / 18%);
  color: #e8ecff;
  cursor: pointer;
}

.btn-primary:hover {
  background: rgb(91 124 255 / 28%);
}

.tool-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 14px;
}

.tool-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px 12px;
  align-items: start;
  padding: 12px 14px;
  border: 1px solid var(--color-border, #2a3142);
  border-radius: 10px;
  background: rgb(255 255 255 / 4%);
}

.tool-index {
  grid-column: 1 / -1;
  grid-row: 1;
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #6b7280;
}

.fields {
  grid-column: 1;
  grid-row: 2;
  display: grid;
  gap: 10px;
  min-width: 0;
}

.field {
  display: grid;
  gap: 5px;
  font-size: 0.8rem;
}

.field span {
  color: #9ca3af;
}

.field input {
  font-size: 0.9rem;
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid var(--color-border, #2a3142);
  background: rgb(0 0 0 / 25%);
  color: inherit;
  min-width: 0;
}

.command-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.command-input {
  flex: 1;
  min-width: 0;
}

.btn-browse {
  flex: 0 0 auto;
  padding: 6px 12px;
  font-size: 0.85rem;
  white-space: nowrap;
  border-radius: 6px;
  border: 1px solid var(--color-border, #3d465c);
  background: rgb(255 255 255 / 8%);
  color: inherit;
  cursor: pointer;
}

.btn-browse:hover {
  background: rgb(255 255 255 / 12%);
}

.field-wide {
  grid-column: 1 / -1;
}

.btn-remove {
  grid-column: 2;
  grid-row: 2;
  justify-self: end;
  align-self: start;
  font-size: 0.8rem;
  padding: 6px 10px;
}

.btn-add-more {
  width: fit-content;
  padding: 6px 12px;
  font-size: 0.85rem;
  border-radius: 6px;
  border: 1px dashed var(--color-border, #3d465c);
  background: transparent;
  color: #9ca3af;
  cursor: pointer;
}

.btn-add-more:hover {
  color: #e5e7eb;
  border-color: #6b7280;
}
</style>
