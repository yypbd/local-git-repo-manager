<script setup lang="ts">
import { useAttrs } from 'vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <div
    :class="cn(
      'flex flex-col gap-1 px-4 pt-4 pb-3 border-b border-[var(--border)]',
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
