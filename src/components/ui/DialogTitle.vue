<script setup lang="ts">
import { useAttrs } from 'vue'
import { DialogTitle as RadixDialogTitle } from 'radix-vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <RadixDialogTitle
    :class="cn(
      'text-base font-semibold text-[var(--foreground)] leading-none',
      attrs.class as string
    )"
    v-bind="{ ...attrs, class: undefined }"
  >
    <slot />
  </RadixDialogTitle>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
