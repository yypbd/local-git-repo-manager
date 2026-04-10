<script setup lang="ts">
import { computed, ref, useAttrs } from 'vue'
import { cn } from '@/lib/utils'

type Size = 'sm' | 'md'

// ── Props 정의 ─────────────────────────────────────────────────────────────
const props = withDefaults(
  defineProps<{
    modelValue?: string
    type?: string
    size?: Size
    disabled?: boolean
    placeholder?: string
  }>(),
  { type: 'text', size: 'md', disabled: false },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

// inheritAttrs: false — 부모 attrs를 수동 병합
const attrs = useAttrs()
const el = ref<HTMLInputElement | null>(null)

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
    'transition-colors',
    'placeholder:text-[var(--muted-foreground)]',
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
    v-bind="attrs" 로 부모 class·style·이벤트를 전달.
    class는 computedClass에서 이미 병합되었으므로 attrs에서 제외.
  -->
  <input
    ref="el"
    :type="type"
    :value="modelValue"
    :disabled="disabled"
    :placeholder="placeholder"
    :class="computedClass"
    v-bind="{ ...attrs, class: undefined }"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>

<script lang="ts">
export default { inheritAttrs: false }
</script>
