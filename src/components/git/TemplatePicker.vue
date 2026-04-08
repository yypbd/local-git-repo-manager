<script setup lang="ts">
import { computed, ref } from "vue";

type TemplateItem = { name: string; content: string };
const props = withDefaults(
  defineProps<{
    items: TemplateItem[];
    /** true면 검색·동기화·이름 칩만 (다이얼로그 안에서 사용) */
    compact?: boolean;
    /** GitHub 동기화 중이면 버튼 비활성 */
    syncing?: boolean;
  }>(),
  { compact: false, syncing: false },
);
const emit = defineEmits<{ apply: [content: string]; sync: [] }>();
const query = ref("");

const filtered = computed(() =>
  props.items.filter((item) => item.name.toLowerCase().includes(query.value.toLowerCase())),
);
</script>

<template>
  <section class="picker" :class="{ compact }">
    <template v-if="compact">
      <div class="toolbar">
        <input
          v-model="query"
          class="search"
          type="search"
          :placeholder="$t('workspace.templateSearchPlaceholder')"
        />
        <button
          type="button"
          class="sync"
          :disabled="syncing"
          @click="emit('sync')"
        >
          {{ syncing ? $t("workspace.templateSyncing") : $t("workspace.templateSync") }}
        </button>
      </div>
      <div class="chips-scroll">
        <div class="chips">
          <button
            v-for="item in filtered"
            :key="item.name"
            type="button"
            class="chip"
            :title="item.name"
            @click="emit('apply', item.content)"
          >
            {{ item.name }}
          </button>
        </div>
      </div>
    </template>
    <template v-else>
      <h4 class="picker-title">{{ $t("workspace.templatesHeading") }}</h4>
      <input
        v-model="query"
        class="search-wide"
        type="search"
        :placeholder="$t('workspace.templateSearchPlaceholder')"
      />
      <button type="button" class="sync-wide" :disabled="syncing" @click="emit('sync')">
        {{ syncing ? $t("workspace.templateSyncing") : $t("workspace.templateSync") }}
      </button>
      <ul>
        <li v-for="item in filtered" :key="item.name">
          <strong>{{ item.name }}</strong>
          <pre>{{ item.content }}</pre>
          <button type="button" @click="emit('apply', item.content)">적용</button>
        </li>
      </ul>
    </template>
  </section>
</template>

<style scoped>
.picker {
  display: grid;
  gap: 8px;
}

.compact .toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.compact .search {
  flex: 1 1 140px;
  min-width: 0;
  padding: 6px 8px;
  font-size: 0.8rem;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: #161b29;
  color: var(--color-text);
}

.compact .sync {
  flex: 0 0 auto;
  padding: 6px 10px;
  font-size: 0.78rem;
}

.compact .chips-scroll {
  min-height: 0;
  max-height: 112px;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 2px 4px 2px 0;
  border-radius: 6px;
  border: 1px solid rgb(255 255 255 / 8%);
  background: rgb(0 0 0 / 15%);
}

.compact .chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-content: flex-start;
  padding: 4px 2px 4px 4px;
}

.sync:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.picker-title {
  margin: 0 0 4px;
  font-size: 0.9rem;
  font-weight: 600;
}

.search-wide {
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid var(--color-border);
  background: #161b29;
  color: var(--color-text);
}

.sync-wide {
  padding: 8px 12px;
  width: fit-content;
}

.compact .chip {
  padding: 4px 10px;
  font-size: 0.75rem;
  border-radius: 999px;
  border: 1px solid var(--color-border);
  background: #1a2030;
  cursor: pointer;
}

.compact .chip:hover {
  border-color: #7aa2ff;
}
</style>
