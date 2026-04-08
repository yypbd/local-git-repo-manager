import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type Project = {
  id: string;
  name: string;
  rootPaths: string[];
};

const projects = ref<Project[]>([]);

export const useProjectsStore = () => {
  const hasProjects = computed(() => projects.value.length > 0);

  const syncFromBackend = async () => {
    try {
      projects.value = await invoke<Project[]>("projects_list");
    } catch {
      // Browser-only dev fallback without Tauri runtime.
    }
  };

  const createProject = async (name: string) => {
    const trimmed = name.trim();
    if (!trimmed) return;
    const created = await invoke<Project>("projects_create", { name: trimmed });
    projects.value.push(created);
  };

  const renameProject = async (id: string, name: string) => {
    const target = projects.value.find((p) => p.id === id);
    if (!target) return;
    const nextName = name.trim() || target.name;
    const updated = await invoke<Project>("projects_update", { id, name: nextName });
    target.name = updated.name;
  };

  const deleteProject = async (id: string) => {
    try {
      await invoke("projects_delete", { id });
    } catch {
      // Browser-only dev fallback without Tauri runtime.
    }
    projects.value = projects.value.filter((p) => p.id !== id);
  };

  const importFolderDrop = async (path: string) => {
    return await invoke<Project>("projects_import_folder_drop", { path });
  };

  const reorderProjects = async (orderedIds: string[]) => {
    projects.value = await invoke<Project[]>("projects_reorder", { orderedIds });
  };

  return {
    projects,
    hasProjects,
    syncFromBackend,
    createProject,
    renameProject,
    deleteProject,
    importFolderDrop,
    reorderProjects,
  };
};
