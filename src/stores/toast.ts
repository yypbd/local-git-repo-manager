import { ref } from "vue";

type ToastType = "info" | "success" | "error" | "warning";

export type ToastItem = {
  id: number;
  message: string;
  type: ToastType;
};

const items = ref<ToastItem[]>([]);
let nextId = 1;

export const useToastStore = () => {
  const push = (message: string, type: ToastType = "info") => {
    const id = nextId++;
    items.value.push({ id, message, type });
    window.setTimeout(() => {
      items.value = items.value.filter((item) => item.id !== id);
    }, 2600);
  };

  const remove = (id: number) => {
    items.value = items.value.filter((item) => item.id !== id);
  };

  return { items, push, remove };
};
