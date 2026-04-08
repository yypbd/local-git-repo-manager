import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

/** 폴더→프로젝트 이동 포인터 세션 중 (Tauri WebView에서 HTML5 DnD 대신 사용) */
export const folderMovePointerActive = ref(false);
/** `elementFromPoint`으로 찾은 `[data-project-id]` */
export const folderMoveDropTargetId = ref<string | null>(null);
/** 드래그로 이동 직후 의도치 않은 선택 방지 */
export const suppressFolderRowClick = ref(false);
/** 임계값 넘긴 뒤 옮기는 중인 루트 경로들(행 하이라이트) */
export const folderMoveDraggingPaths = ref<string[] | null>(null);

function setMoveCursor(active: boolean) {
  document.body.style.cursor = active ? "grabbing" : "";
}

const MOVE_THRESHOLD_PX = 8;

type Session = {
  fromProjectId: string;
  paths: string[];
  startX: number;
  startY: number;
  pointerId: number;
  moved: boolean;
};

let session: Session | null = null;

/**
 * 폴더 목록 행에서 포인터 다운 — 임계값 이상 움직이면 프로젝트 행 위에 놓을 때 이동.
 */
export function beginFolderMovePointer(
  e: PointerEvent,
  ctx: { fromProjectId: string; paths: string[] },
  deps: {
    afterMove: (toProjectId: string) => Promise<void>;
    onError: (err: unknown) => void;
  },
): void {
  if (e.pointerType === "mouse" && e.button !== 0) return;
  const el = e.target as HTMLElement | null;
  if (el?.closest("button, a[href], input, select, textarea")) return;
  if (session) return;

  session = {
    fromProjectId: ctx.fromProjectId,
    paths: ctx.paths,
    startX: e.clientX,
    startY: e.clientY,
    pointerId: e.pointerId,
    moved: false,
  };

  const onMove = (ev: PointerEvent) => {
    if (!session || ev.pointerId !== session.pointerId) return;
    const dx = ev.clientX - session.startX;
    const dy = ev.clientY - session.startY;
    if (!session.moved && dx * dx + dy * dy >= MOVE_THRESHOLD_PX * MOVE_THRESHOLD_PX) {
      session.moved = true;
      folderMovePointerActive.value = true;
      folderMoveDraggingPaths.value = [...session.paths];
      setMoveCursor(true);
    }
    if (!session.moved) return;
    const hit = document.elementFromPoint(ev.clientX, ev.clientY);
    const row = hit?.closest("[data-project-id]");
    folderMoveDropTargetId.value = row?.getAttribute("data-project-id") ?? null;
  };

  const finish = (ev: PointerEvent) => {
    if (!session || ev.pointerId !== session.pointerId) return;
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", finish);
    window.removeEventListener("pointercancel", finish);

    const s = session;
    session = null;
    const targetId = folderMoveDropTargetId.value;
    folderMoveDropTargetId.value = null;
    folderMovePointerActive.value = false;
    folderMoveDraggingPaths.value = null;
    setMoveCursor(false);

    if (!s.moved) return;

    suppressFolderRowClick.value = true;

    if (!targetId || targetId === s.fromProjectId || !s.paths.length) return;

    void (async () => {
      try {
        for (const path of s.paths) {
          await invoke("move_root_to_project", {
            fromProjectId: s.fromProjectId,
            toProjectId: targetId,
            path,
          });
        }
        await deps.afterMove(targetId);
      } catch (err) {
        deps.onError(err);
      }
    })();
  };

  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", finish);
  window.addEventListener("pointercancel", finish);
}
