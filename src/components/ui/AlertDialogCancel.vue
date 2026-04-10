<script setup lang="ts">
import { useAttrs } from 'vue'
import { AlertDialogCancel as RadixAlertDialogCancel } from 'radix-vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <RadixAlertDialogCancel
    :class="cn(
      'btn btn-sm btn-secondary',
      'inline-flex items-center justify-center',
      'cursor-pointer rounded-sm border border-solid',
      'border-[#4a5568] bg-[#1a2029] text-[var(--foreground)]',
      'hover:bg-[#242d3a] transition-colors',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ring)] focus-visible:ring-offset-1',
      'disabled:opacity-[0.45] disabled:cursor-not-allowed disabled:pointer-events-none',
      'min-h-[var(--btn-height-sm)] px-2 text-[var(--btn-font-sm)]',
      attrs.class as string
    )"
    v-bind="{ ...attrs, class: undefined }"
  >
    <slot />
  </RadixAlertDialogCancel>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
