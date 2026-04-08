<script setup lang="ts">
import { computed, ref, useAttrs } from "vue";

type Size = "sm" | "md";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    size?: Size;
    disabled?: boolean;
  }>(),
  { size: "md", disabled: false },
);

const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const attrs = useAttrs();
const el = ref<HTMLSelectElement | null>(null);

defineExpose({
  focus: () => el.value?.focus(),
});

const sizeClass = computed(() => (props.size === "sm" ? "ui-control-sm" : "ui-control-md"));
</script>

<template>
  <select
    ref="el"
    class="ui-control"
    :class="sizeClass"
    :value="modelValue"
    :disabled="disabled"
    v-bind="attrs"
    @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <slot />
  </select>
</template>

