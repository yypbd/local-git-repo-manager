<script setup lang="ts">
import { computed } from "vue";
import GitStatusTreeNodes from "@/components/workspace/GitStatusTreeNodes.vue";
import { buildPathTree } from "@/utils/gitPathTree";

const props = defineProps<{
  /** 저장소 기준 상대 경로 목록 */
  paths: string[];
  /** 폴더 행 로딩 또는 git_status_files 로딩 중 */
  pending: boolean;
  /** 목록이 비었을 때 표시할 문구(부모에서 `$t(...)` 로 넘김) */
  emptyText: string;
}>();

const treeRoots = computed(() => buildPathTree(props.paths));
</script>

<template>
  <div class="list-root">
    <div v-if="pending" class="files-empty">…</div>
    <div v-else-if="paths.length" class="files-scroll-wrap">
      <GitStatusTreeNodes :nodes="treeRoots" root />
    </div>
    <div v-else class="files-empty">{{ emptyText }}</div>
  </div>
</template>

<style scoped>
.list-root {
  flex: 1 1 auto;
  min-height: 0;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.files-empty {
  font-size: 0.8rem;
  opacity: 0.75;
}

.files-scroll-wrap {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
}
</style>
