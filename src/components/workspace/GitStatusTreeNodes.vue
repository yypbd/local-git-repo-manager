<script setup lang="ts">
import type { PathTreeNode } from "@/utils/gitPathTree";
import GitStatusTreeNodes from "./GitStatusTreeNodes.vue";

withDefaults(
  defineProps<{
    nodes: PathTreeNode[];
    /** 최상위 `GitStatusPathList`에서만 true */
    root?: boolean;
  }>(),
  { root: false },
);
</script>

<template>
  <ul class="tree-nodes" :class="{ 'tree-nodes--root': root }" role="list">
    <li v-for="node in nodes" :key="node.fullPath" class="tree-item" role="listitem">
      <details v-if="node.children.length > 0" class="tree-dir" open>
        <summary class="tree-summary">{{ node.name }}</summary>
        <GitStatusTreeNodes :nodes="node.children" />
      </details>
      <span v-else class="tree-file">{{ node.name }}</span>
    </li>
  </ul>
</template>

<style scoped>
.tree-nodes {
  list-style: none;
  margin: 0;
  padding: 0 0 0 10px;
  display: grid;
  gap: 2px;
}

.tree-nodes--root {
  padding-left: 0;
}

.tree-item {
  margin: 0;
  min-width: 0;
}

.tree-dir {
  margin: 0;
}

.tree-summary {
  cursor: pointer;
  font-size: 0.78rem;
  line-height: 1.35;
  list-style: none;
  user-select: none;
}

.tree-summary::-webkit-details-marker {
  display: none;
}

.tree-summary::before {
  content: "▸ ";
  display: inline-block;
  opacity: 0.55;
  font-size: 0.65rem;
  margin-right: 2px;
}

.tree-dir[open] > .tree-summary::before {
  content: "▾ ";
}

.tree-file {
  display: block;
  font-size: 0.78rem;
  line-height: 1.35;
  padding-left: 0.85rem;
  word-break: break-all;
  user-select: text;
  -webkit-user-select: text;
}

.tree-dir .tree-nodes {
  padding-left: 12px;
  margin-top: 2px;
  border-left: 1px solid var(--color-border, #2d3748);
}
</style>
