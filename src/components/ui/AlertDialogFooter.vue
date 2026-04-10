<script setup lang="ts">
import { useAttrs } from 'vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <div
    :class="cn(
      'flex flex-wrap justify-end gap-2 px-4 py-3 border-t border-[var(--border)]',
      attrs.class as string
    )"
    v-bind="{ ...attrs, class: undefined }"
  >
    <slot />
  </div>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
