<script setup lang="ts">
import { useAttrs } from 'vue'
import {
  DialogPortal,
  DialogContent as RadixDialogContent,
} from 'radix-vue'
import { cn } from '@/lib/utils'
import DialogOverlay from './DialogOverlay.vue'

const attrs = useAttrs()
</script>

<template>
  <DialogPortal>
    <DialogOverlay />
    <RadixDialogContent
      :class="cn(
        'fixed left-1/2 top-1/2 z-[var(--z-modal)] -translate-x-1/2 -translate-y-1/2',
        'w-full max-w-md',
        'bg-[#161b29] border border-[var(--border)] rounded-lg shadow-2xl',
        'p-0 outline-none',
        'data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95 data-[state=open]:slide-in-from-top-4',
        'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=closed]:slide-out-to-top-4',
        attrs.class as string
      )"
      v-bind="{ ...attrs, class: undefined }"
      @escape-key-down.prevent
    >
      <slot />
    </RadixDialogContent>
  </DialogPortal>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
