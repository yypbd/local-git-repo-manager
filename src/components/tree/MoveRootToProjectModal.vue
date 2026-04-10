<script setup lang="ts">
import { ref, watch } from "vue";
import { DialogRoot, DialogClose } from "radix-vue";
import type { Project } from "@/stores/projects";
import { useDialogInputFocus } from "@/composables/useDialogShortcuts";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import Select from "@/components/ui/Select.vue";
import Button from "@/components/ui/Button.vue";

const props = defineProps<{ projects: Project[] }>();
const emit = defineEmits<{ close: []; submit: [projectId: string] }>();
const targetId = ref(props.projects[0]?.id ?? "");
const selectRef = ref<{ focus?: () => void } | null>(null);

useDialogInputFocus(selectRef);

watch(
  () => props.projects,
  (list) => {
    if (!list.length) return;
    if (!list.some((p) => p.id === targetId.value)) {
      targetId.value = list[0]!.id;
    }
  },
  { deep: true },
);

const submit = () => {
  if (!targetId.value) return;
  emit("submit", targetId.value);
};
</script>

<template>
  <DialogRoot :open="true" @update:open="(v: boolean) => { if (!v) emit('close') }">
    <DialogContent class="max-w-xs">
      <DialogHeader>
        <DialogTitle>다른 프로젝트로 이동</DialogTitle>
      </DialogHeader>
      <form class="grid gap-3 px-4 py-3" @submit.prevent="submit">
        <Select ref="selectRef" v-model="targetId">
          <option v-for="item in projects" :key="item.id" :value="item.id">{{ item.name }}</option>
        </Select>
        <DialogFooter class="border-0 p-0 pt-1">
          <DialogClose as-child>
            <Button type="button" size="sm" variant="secondary" @click="emit('close')">취소</Button>
          </DialogClose>
          <Button type="submit" size="sm" variant="default">이동</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </DialogRoot>
</template>
