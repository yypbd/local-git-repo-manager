<script setup lang="ts">
import { computed, ref, useAttrs } from 'vue'
import { cn } from '@/lib/utils'

// TechLead 지시사항: 정적 2-옵션 케이스에는 네이티브 <select> 유지.
// 동적 목록이 필요해질 때만 Radix Select로 전환.

type Size = 'sm' | 'md'

// ── Props 정의 ─────────────────────────────────────────────────────────────
const props = withDefaults(
  defineProps<{
    modelValue: string
    size?: Size
    disabled?: boolean
  }>(),
  { size: 'md', disabled: false },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

// inheritAttrs: false — 부모 attrs를 수동 병합
const attrs = useAttrs()
const el = ref<HTMLSelectElement | null>(null)

// 부모에게 focus() 노출
defineExpose({ focus: () => el.value?.focus() })

// 크기별 클래스
const sizeClass = computed(() =>
  props.size === 'sm' ? 'ui-control-sm' : 'ui-control-md',
)

const computedClass = computed(() =>
  cn(
    // ── 브리지 클래스: `:deep(.ui-control)` 콜사이트 호환성 유지
    'ui-control',
    // ── Tailwind 스타일
    'w-full',
    'border border-solid border-[var(--border)]',
    'rounded-sm',
    'bg-[var(--input)]',
    'text-[var(--foreground)]',
    'outline-none',
    'box-border',
    'cursor-pointer',
    'transition-colors',
    'focus:border-[#3b82f6]',
    'focus:shadow-[0_0_0_2px_var(--ui-control-focus-ring)]',
    'disabled:opacity-[0.45]',
    'disabled:cursor-not-allowed',
    sizeClass.value,
    attrs.class as string,
  ),
)
</script>

<template>
  <!--
    네이티브 <select> 유지 — 정적 2-옵션 케이스 호환성.
    v-bind="attrs" 로 부모 class·style·이벤트를 전달.
  -->
  <select
    ref="el"
    :value="modelValue"
    :disabled="disabled"
    :class="computedClass"
    v-bind="{ ...attrs, class: undefined }"
    @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <slot />
  </select>
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
