<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { useToastStore } from "@/stores/toast";
import AppShell from "@/layouts/AppShell.vue";
import ProjectSidebar from "@/components/workspace/ProjectSidebar.vue";
import ProjectFolderPanel from "@/components/workspace/ProjectFolderPanel.vue";
import { useFolderRootRows } from "@/composables/useFolderRootRows";
import { useProjectsStore } from "@/stores/projects";

const route = useRoute();
const router = useRouter();
const { projects, syncFromBackend } = useProjectsStore();
const { t } = useI18n();
const { push: toast } = useToastStore();

const projectId = computed(() => (typeof route.params.id === "string" ? route.params.id : ""));
const project = computed(() => projects.value.find((p) => p.id === projectId.value));

const rootPathsRef = computed(() => project.value?.rootPaths ?? []);
const {
  rows: folderRows,
  loading: folderRowsLoading,
  loadedCount: folderRowsLoaded,
  totalCount: folderRowsTotal,
  reload: reloadFolderRows,
} = useFolderRootRows(rootPathsRef);

/** 폴더 목록: 포커스(상세·마지막 클릭) + Ctrl/Shift 멀티 선택 */
const selectedFolderPath = ref<string | null>(null);
const selectedFolderPaths = ref<string[]>([]);

const addFolders = async (paths: string[]) => {
  if (!project.value || paths.length === 0) return;
  const id = project.value.id;
  const unique = [...new Set(paths.map((p) => p.trim()).filter(Boolean))];
  let ok = 0;
  let fail = 0;
  for (const path of unique) {
    try {
      await invoke("projects_add_root", { projectId: id, path });
      ok += 1;
    } catch {
      fail += 1;
    }
  }
  await syncFromBackend();
  if (fail > 0 && ok === 0) {
    toast(t("workspace.addFoldersAllFailed"), "error");
  } else if (fail > 0) {
    toast(t("workspace.addFoldersSomeFailed", { fail, ok }), "warning");
  }
};

onMounted(() => {
  void syncFromBackend();
});

watch(projectId, () => {
  selectedFolderPath.value = null;
  selectedFolderPaths.value = [];
});

watch(
  () => [projectId.value, projects.value] as const,
  ([id, list]) => {
    if (list.length === 0) return;
    const first = list[0]!;
    /** `/projects`만 열리면 id가 없어 상세·하단 액션이 비는 문제 방지 */
    if (!id) {
      void router.replace(`/projects/${first.id}`);
      return;
    }
    if (!list.some((p) => p.id === id)) {
      void router.replace(`/projects/${first.id}`);
    }
  },
  { immediate: true },
);
</script>

<template>
  <AppShell>
    <template #projects>
      <ProjectSidebar />
    </template>
    <template #folders>
      <ProjectFolderPanel
        v-model:selected-folder-path="selectedFolderPath"
        v-model:selected-folder-paths="selectedFolderPaths"
        :project="project"
        :rows="folderRows"
        :loading="folderRowsLoading"
        :folder-rows-loaded="folderRowsLoaded"
        :folder-rows-total="folderRowsTotal"
        :reload="reloadFolderRows"
        @dropped="addFolders"
      />
    </template>
  </AppShell>
</template>
