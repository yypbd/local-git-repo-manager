<script setup lang="ts">
import { computed, ref, useAttrs } from "vue";

type Size = "sm" | "md";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    type?: string;
    size?: Size;
    disabled?: boolean;
  }>(),
  { type: "text", size: "md", disabled: false },
);

const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const attrs = useAttrs();
const el = ref<HTMLInputElement | null>(null);

defineExpose({
  focus: () => el.value?.focus(),
});

const sizeClass = computed(() => (props.size === "sm" ? "ui-control-sm" : "ui-control-md"));
</script>

<template>
  <input
    ref="el"
    class="ui-control"
    :class="sizeClass"
    :type="type"
    :value="modelValue"
    :disabled="disabled"
    v-bind="attrs"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>

