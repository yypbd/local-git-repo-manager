<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToastStore } from "@/stores/toast";

const emit = defineEmits<{ dropped: [paths: string[]] }>();
const { push } = useToastStore();
const { t } = useI18n();

const dropEl = ref<HTMLElement | null>(null);
let unlistenDrag: (() => void) | undefined;

function pathFromFile(file: File): string | undefined {
  const extended = file as File & { path?: string };
  return typeof extended.path === "string" ? extended.path : undefined;
}

function uniquePaths(paths: string[]): string[] {
  return [...new Set(paths.map((p) => p.trim()).filter(Boolean))];
}

function pointInsideClientRect(
  clientX: number,
  clientY: number,
  rect: DOMRect,
): boolean {
  return clientX >= rect.left && clientX <= rect.right && clientY >= rect.top && clientY <= rect.bottom;
}

onMounted(async () => {
  try {
    unlistenDrag = await getCurrentWebview().onDragDropEvent(async (evt) => {
      const p = evt.payload;
      if (p.type !== "drop" || p.paths.length === 0) return;
      const el = dropEl.value;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const factor = await getCurrentWindow().scaleFactor();
      const pos = p.position;
      const physical =
        pos instanceof PhysicalPosition
          ? pos
          : new PhysicalPosition(
              (pos as { x: number; y: number }).x,
              (pos as { x: number; y: number }).y,
            );
      const logical = physical.toLogical(factor);
      if (pointInsideClientRect(logical.x, logical.y, rect)) {
        emit("dropped", uniquePaths(p.paths));
      }
    });
  } catch {
    /* Tauri 외 환경 */
  }
});

onUnmounted(() => {
  unlistenDrag?.();
});

const onDropHtml = (event: DragEvent) => {
  event.preventDefault();
  const files = event.dataTransfer?.files;
  const collected: string[] = [];
  if (files?.length) {
    for (let i = 0; i < files.length; i++) {
      const path = pathFromFile(files[i]!);
      if (path) collected.push(path);
    }
    if (collected.length) {
      emit("dropped", uniquePaths(collected));
      return;
    }
  }
  const raw = event.dataTransfer?.getData("text/uri-list")?.trim();
  if (raw) {
    const uriLines = raw.split("\n").map((l) => l.trim()).filter(Boolean);
    const fromUris: string[] = [];
    for (const line of uriLines) {
      if (!line.startsWith("file://")) continue;
      try {
        const url = new URL(line);
        fromUris.push(decodeURIComponent(url.pathname));
      } catch {
        /* ignore */
      }
    }
    if (fromUris.length) {
      emit("dropped", uniquePaths(fromUris));
      return;
    }
  }
  push(t("workspace.dropPathError"), "error");
};
</script>

<template>
  <div
    ref="dropEl"
    class="drop"
    @dragover.prevent
    @drop="onDropHtml"
  >
    <slot />
  </div>
</template>

<style scoped>
.drop {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
</style>
