<script setup lang="ts">
import { computed, useAttrs } from 'vue'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

// ── CVA 변형 정의 ──────────────────────────────────────────────────────────
const buttonVariants = cva(
  // 기본 클래스 (모든 변형 공통)
  [
    'btn',                         // 브리지: base.css .btn @layer 스타일 적용
    'inline-flex', 'items-center', 'justify-center',
    'gap-[var(--btn-gap)]',
    'leading-none',
    'cursor-pointer',
    'rounded-sm',
    'border', 'border-solid',
    'transition-colors',
    'focus-visible:outline-none',
    'focus-visible:ring-2',
    'focus-visible:ring-[var(--ring)]',
    'focus-visible:ring-offset-1',
    'disabled:opacity-[0.45]',
    'disabled:cursor-not-allowed',
    'disabled:pointer-events-none',
  ],
  {
    variants: {
      variant: {
        // shadcn 'default' = 기존 'primary'
        default:     ['border-[#3b4f7a]', 'bg-[#1e2a42]', 'text-[var(--foreground)]', 'enabled:hover:bg-[#253350]'],
        secondary:   ['border-[#4a5568]', 'bg-[#1a2029]', 'text-[var(--foreground)]', 'enabled:hover:bg-[#242d3a]'],
        destructive: ['border-[#7f1d1d]', 'bg-[#3f1212]', 'text-[var(--foreground)]', 'enabled:hover:bg-[#5c1a1a]'],
        success:     ['border-[#3d5a45]', 'bg-[#15251a]', 'text-[var(--foreground)]', 'enabled:hover:bg-[#1a3024]'],
        ghost:       ['border-transparent', 'bg-transparent', 'text-[var(--foreground)]', 'enabled:hover:bg-[#1b2233]'],
        outline:     ['border-[var(--border)]', 'bg-transparent', 'text-[var(--foreground)]', 'enabled:hover:bg-[#1b2233]'],
      },
      size: {
        default: ['min-h-btn-md', 'px-[10px]', 'text-btn-md'],
        sm:      ['min-h-btn-sm', 'px-2',      'text-btn-sm'],
        lg:      ['min-h-[38px]', 'px-4',      'text-[14px]'],
        icon:    ['min-h-btn-md', 'w-[var(--btn-height-md)]', 'px-0'],
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  },
)

type ButtonVariants = VariantProps<typeof buttonVariants>

// ── Props 정의 ─────────────────────────────────────────────────────────────
const props = withDefaults(
  defineProps<{
    variant?: ButtonVariants['variant']
    size?: ButtonVariants['size']
    disabled?: boolean
    type?: 'button' | 'submit' | 'reset'
    asChild?: boolean
  }>(),
  {
    variant: 'default',
    size: 'default',
    disabled: false,
    type: 'button',
    asChild: false,
  },
)

// inheritAttrs: false — 부모가 전달한 class/style/이벤트를 수동으로 병합
const attrs = useAttrs()

const computedClass = computed(() =>
  cn(buttonVariants({ variant: props.variant, size: props.size }), attrs.class as string),
)
</script>

<template>
  <!--
    v-bind="attrs" 로 부모 class·style·이벤트를 전달.
    inheritAttrs: false 이므로 Vue가 자동으로 루트 엘리먼트에 attrs를 붙이지 않음.
  -->
  <button
    :type="type"
    :disabled="disabled"
    :class="computedClass"
    v-bind="{ ...attrs, class: undefined }"
  >
    <slot />
  </button>
</template>

<script lang="ts">
// inheritAttrs는 Options API 블록에서 선언해야 함 (<script setup>과 병행)
export default { inheritAttrs: false }
</script>
