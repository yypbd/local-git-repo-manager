<script setup lang="ts">
import { ref } from "vue";
import { DialogRoot, DialogClose } from "radix-vue";
import { useDialogInputFocus } from "@/composables/useDialogShortcuts";
import DialogContent from "@/components/ui/DialogContent.vue";
import DialogHeader from "@/components/ui/DialogHeader.vue";
import DialogTitle from "@/components/ui/DialogTitle.vue";
import DialogFooter from "@/components/ui/DialogFooter.vue";
import Input from "@/components/ui/Input.vue";
import Button from "@/components/ui/Button.vue";

const emit = defineEmits<{
  close: [];
  submit: [name: string];
}>();

const name = ref("");
const inputRef = ref<{ focus?: () => void } | null>(null);
useDialogInputFocus(inputRef);

const submit = () => {
  if (!name.value.trim()) return;
  const value = name.value;
  name.value = "";
  emit("submit", value);
};
</script>

<template>
  <DialogRoot :open="true" @update:open="(v: boolean) => { if (!v) emit('close') }">
    <DialogContent class="max-w-xs">
      <DialogHeader>
        <DialogTitle>프로젝트 생성</DialogTitle>
      </DialogHeader>
      <form class="grid gap-3 px-4 py-3" @submit.prevent="submit">
        <Input ref="inputRef" v-model="name" placeholder="프로젝트 이름" autocomplete="off" />
        <DialogFooter class="border-0 p-0 pt-1">
          <DialogClose as-child>
            <Button type="button" size="sm" variant="secondary" @click="emit('close')">취소</Button>
          </DialogClose>
          <Button type="submit" size="sm" variant="default">생성</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </DialogRoot>
</template>
