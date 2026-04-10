<script setup lang="ts">
import { useAttrs } from 'vue'
import { AlertDialogDescription as RadixAlertDialogDescription } from 'radix-vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <RadixAlertDialogDescription
    :class="cn(
      'text-sm text-[var(--muted-foreground)] leading-snug',
      attrs.class as string
    )"
    v-bind="{ ...attrs, class: undefined }"
  >
    <slot />
  </RadixAlertDialogDescription>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
