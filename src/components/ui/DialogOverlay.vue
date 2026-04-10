<script setup lang="ts">
import { useAttrs } from 'vue'
import { DialogOverlay as RadixDialogOverlay } from 'radix-vue'
import { cn } from '@/lib/utils'

const attrs = useAttrs()
</script>

<template>
  <RadixDialogOverlay
    :class="cn(
      'fixed inset-0 z-[var(--z-modal)] bg-black/60 backdrop-blur-[1px]',
      'data-[state=open]:animate-in data-[state=open]:fade-in-0',
      'data-[state=closed]:animate-out data-[state=closed]:fade-out-0',
      attrs.class as string
    )"
    v-bind="{ ...attrs, class: undefined }"
  />
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
