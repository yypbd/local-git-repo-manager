/** @type {import('tailwindcss').Config} */
export default {
  // Scan only frontend source files; deliberately exclude src-tauri to avoid
  // scanning hundreds of thousands of Rust files which would slow builds.
  content: [
    "./index.html",
    "./src/**/*.{vue,ts}",
  ],

  theme: {
    extend: {
      colors: {
        // ── 기존 디자인 토큰 매핑 ──────────────────────────────────
        bg:              "var(--color-bg)",
        // NOTE: 'text' key 사용 금지 — Tailwind의 text-* 유틸리티를 가림.
        //       대신 shadcn 'foreground' 사용.
        "surface-muted": "var(--color-surface-muted)",
        "ui-control":    "var(--ui-control-bg)",
        accent:          "var(--color-accent)",

        // ── shadcn 시맨틱 색상 (CSS 변수에서 읽어옴) ──────────────
        background:            "var(--background)",
        foreground:            "var(--foreground)",
        primary: {
          DEFAULT:             "var(--primary)",
          foreground:          "var(--primary-foreground)",
        },
        secondary: {
          DEFAULT:             "var(--secondary)",
          foreground:          "var(--secondary-foreground)",
        },
        muted: {
          DEFAULT:             "var(--muted)",
          foreground:          "var(--muted-foreground)",
        },
        destructive: {
          DEFAULT:             "var(--destructive)",
          foreground:          "var(--destructive-foreground)",
        },
        border:                "var(--border)",
        input:                 "var(--input)",
        ring:                  "var(--ring)",
      },

      borderRadius: {
        // 기존 토큰
        sm:     "var(--radius-sm)",
        md:     "var(--radius-md)",
        // shadcn --radius 매핑
        DEFAULT: "var(--radius)",
      },

      spacing: {
        1: "var(--space-1)",
        2: "var(--space-2)",
        3: "var(--space-3)",
        4: "var(--space-4)",
      },

      minHeight: {
        "btn-sm": "var(--btn-height-sm)",
        "btn-md": "var(--btn-height-md)",
      },

      fontSize: {
        "btn-sm": "var(--btn-font-sm)",
        "btn-md": "var(--btn-font-md)",
      },

      gap: {
        btn: "var(--btn-gap)",
      },

      ringColor: {
        DEFAULT: "var(--ring)",
        ring:    "var(--ring)",
      },

      // z-index 스케일 (CSS 변수와 병행; Tailwind z-[var(--z-modal)] 문법도 가능)
      zIndex: {
        toast:        "var(--z-toast)",
        modal:        "var(--z-modal)",
        "modal-nested": "var(--z-modal-nested)",
        "dialog-top": "var(--z-dialog-top)",
      },
    },
  },

  corePlugins: {
    // CRITICAL: Tailwind의 base/preflight 리셋이 base.css의 버튼/폼 스타일을
    // 덮어쓰지 않도록 false 유지.
    preflight: false,
  },
};
