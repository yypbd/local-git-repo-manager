import { onMounted, onUnmounted, nextTick, type Ref } from "vue";

/** 다이얼로그가 열릴 때 첫 입력/선택 요소에 포커스 */
export function useDialogInputFocus(elRef: Ref<HTMLElement | null | undefined>) {
  onMounted(() => {
    nextTick(() => elRef.value?.focus());
  });
}

/** Esc: 취소(닫기) */
export function useDialogEscape(onCancel: () => void) {
  const handler = (e: KeyboardEvent) => {
    if (e.key !== "Escape") return;
    e.preventDefault();
    e.stopPropagation();
    onCancel();
  };
  onMounted(() => window.addEventListener("keydown", handler, true));
  onUnmounted(() => window.removeEventListener("keydown", handler, true));
}

type EnterConfirmOptions = {
  /** false면 Enter로 확인하지 않음 */
  enabled?: () => boolean;
};

/**
 * Enter: 확인 동작 (textarea·시스템 다이얼로그 제외)
 * input/select/button에서는 동작 — 폼 제출과 중복되지 않게 컴포넌트에서 하나만 쓰는 것을 권장
 */
export function useDialogEnterConfirm(onConfirm: () => void, options?: EnterConfirmOptions) {
  const handler = (e: KeyboardEvent) => {
    if (e.key !== "Enter") return;
    if (options?.enabled && !options.enabled()) return;
    const t = e.target as HTMLElement | null;
    if (!t) return;
    if (t.tagName === "TEXTAREA") return;
    if (t.closest("textarea")) return;
    /** 포커스가 버튼이면 브라우저 기본(해당 버튼 클릭)에 맡김 */
    if (t.tagName === "BUTTON") return;
    e.preventDefault();
    e.stopPropagation();
    onConfirm();
  };
  onMounted(() => window.addEventListener("keydown", handler, true));
  onUnmounted(() => window.removeEventListener("keydown", handler, true));
}
