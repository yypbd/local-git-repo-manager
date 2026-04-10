<script setup lang="ts">
import { useAttrs } from 'vue'
import { AlertDialogTitle as RadixAlertDialogTitle } from 'radix-vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <RadixAlertDialogTitle
    :class="cn(
      'text-base font-semibold text-[var(--foreground)] leading-none',
      attrs.class as string
    )"
    v-bind="{ ...attrs, class: undefined }"
  >
    <slot />
  </RadixAlertDialogTitle>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
