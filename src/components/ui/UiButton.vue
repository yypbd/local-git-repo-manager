<script setup lang="ts">
import { computed, useAttrs } from "vue";

type Variant = "primary" | "secondary" | "danger" | "success";
type Size = "sm" | "md";
type NativeType = "button" | "submit" | "reset";

const props = withDefaults(
  defineProps<{
    variant?: Variant;
    size?: Size;
    type?: NativeType;
    disabled?: boolean;
  }>(),
  { variant: "secondary", size: "md", type: "button", disabled: false },
);

const attrs = useAttrs();

const className = computed(() => [
  "btn",
  props.size === "sm" ? "btn-sm" : "",
  props.variant === "primary" ? "btn-primary" : "",
  props.variant === "danger" ? "btn-danger" : "",
  props.variant === "success" ? "btn-success" : "",
  props.variant === "secondary" ? "btn-secondary" : "",
]);
</script>

<template>
  <button :type="type" :disabled="disabled" :class="className" v-bind="attrs">
    <slot />
  </button>
</template>

