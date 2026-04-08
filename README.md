# Local Git Repo Manager

Tauri + Vue 기반 데스크톱 앱입니다. (npm 패키지명: `local-git-repo-manager`)

## 요구 사항

- Node.js 20+
- Rust toolchain (`rustup`, `cargo`)
- OS별 Tauri 선행 의존성
  - 공식 가이드: <https://tauri.app/start/prerequisites/>

## 설치

```bash
npm install
```

## 개발 실행 (Tauri 앱)

아래 명령으로 Vite dev server + Tauri 앱을 함께 실행합니다.

```bash
npm run tauri dev
```

## 프론트엔드만 실행

```bash
npm run dev
```

## 프로덕션 빌드

프론트 산출물(`dist/`) 생성:

```bash
npm run build
```

Tauri 번들 빌드(플랫폼별 앱 패키지):

```bash
npm run tauri build
```

## 프로젝트 구조

- `src/`: Vue UI
- `src-tauri/`: Rust/Tauri 코어
- `agents/docs/`: 요구사항/설계/UX 문서
- `agents/tasks/`: 태스크 관리 문서
