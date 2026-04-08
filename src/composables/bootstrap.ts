import { invoke } from "@tauri-apps/api/core";

export type BootstrapPayload = {
  recommendedDataRoot: string;
  confirmedDataRoot: string | null;
};

export const getBootstrap = async (): Promise<BootstrapPayload | null> => {
  try {
    return await invoke<BootstrapPayload>("get_bootstrap");
  } catch {
    return null;
  }
};

export const confirmDataRoot = async (path: string): Promise<BootstrapPayload | null> => {
  try {
    return await invoke<BootstrapPayload>("confirm_data_root", { path });
  } catch {
    return null;
  }
};
