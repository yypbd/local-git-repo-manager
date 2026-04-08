import { createApp } from "vue";
import { invoke } from "@tauri-apps/api/core";
import App from "./App.vue";
import router from "./router";
import i18n from "./i18n";
import "./assets/base.css";

const app = createApp(App);
app.use(router);
app.use(i18n);

void invoke<{ locale: string }>("get_settings")
  .then((s) => {
    if (s.locale === "ko" || s.locale === "en") {
      i18n.global.locale.value = s.locale;
    }
  })
  .catch(() => {});

app.mount("#app");

// WebView 기본 우클릭 메뉴(검사·새로고침 등) — 빌드 여부와 관계없이 뜨므로, 입력·편집 영역은 제외하고 막습니다.
document.addEventListener(
  "contextmenu",
  (e) => {
    const t = e.target;
    if (
      t instanceof HTMLInputElement ||
      t instanceof HTMLTextAreaElement ||
      (t instanceof HTMLElement && t.isContentEditable)
    ) {
      return;
    }
    e.preventDefault();
  },
  { capture: true },
);
