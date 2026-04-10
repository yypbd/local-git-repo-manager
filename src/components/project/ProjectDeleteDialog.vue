<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { AlertDialogRoot } from "radix-vue";
import type { Project } from "@/stores/projects";
import AlertDialogContent from "@/components/ui/AlertDialogContent.vue";
import AlertDialogHeader from "@/components/ui/AlertDialogHeader.vue";
import AlertDialogTitle from "@/components/ui/AlertDialogTitle.vue";
import AlertDialogDescription from "@/components/ui/AlertDialogDescription.vue";
import AlertDialogFooter from "@/components/ui/AlertDialogFooter.vue";
import AlertDialogCancel from "@/components/ui/AlertDialogCancel.vue";
import AlertDialogAction from "@/components/ui/AlertDialogAction.vue";
import Button from "@/components/ui/Button.vue";

const props = defineProps<{ project: Project; hasOtherProjects: boolean }>();
const emit = defineEmits<{
  close: [];
  deleteOnly: [];
  moveThenDelete: [];
}>();

const { t } = useI18n();
const hasLinkedFolders = computed(() => props.project.rootPaths.length > 0);
const linkedFolderCount = computed(() => props.project.rootPaths.length);
</script>

<template>
  <AlertDialogRoot :open="true" @update:open="(v: boolean) => { if (!v) emit('close') }">
    <AlertDialogContent class="max-w-[400px]">
      <AlertDialogHeader>
        <AlertDialogTitle>{{ $t("workspace.projectDeleteTitle") }}</AlertDialogTitle>
        <AlertDialogDescription>
          {{ $t("workspace.projectDeleteMessage", { name: project.name }) }}
        </AlertDialogDescription>
        <p v-if="hasLinkedFolders" class="text-sm text-[var(--muted-foreground)] leading-snug mt-1">
          {{ $t("workspace.projectDeleteFoldersHint", { n: linkedFolderCount }) }}
        </p>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel @click="emit('close')">
          {{ $t("workspace.cancel") }}
        </AlertDialogCancel>
        <template v-if="hasLinkedFolders">
          <Button
            type="button"
            size="sm"
            variant="secondary"
            :disabled="!props.hasOtherProjects"
            @click="emit('moveThenDelete')"
          >
            {{ $t("workspace.projectDeleteMoveFoldersFirst") }}
          </Button>
          <AlertDialogAction @click="emit('deleteOnly')">
            {{ $t("workspace.projectDeleteUnlinkOnly") }}
          </AlertDialogAction>
        </template>
        <AlertDialogAction v-else @click="emit('deleteOnly')">
          {{ $t("workspace.projectDeleteConfirm") }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialogRoot>
</template>
