# Local Git Repo Manager

Tauri 2 + Vue 3 기반 데스크톱 앱입니다. 로컬 Git 저장소·프로젝트 폴더를 묶어 관리합니다.

## 주요 기능

- 프로젝트·루트 폴더를 묶어 관리하고, 드래그로 순서·프로젝트 간 이동.
- Git 상태 요약, 원격 관리, init / `.git` 제거, zip 내보내기.
- 브랜치·변경·미추적·`.gitignore`를 탭으로 확인·편집.
- 언어·Git 경로·외부 도구 등 설정. 첫 실행 시 데이터 저장 위치를 확정.

| 구분 | 버전·이름 |
|------|-----------|
| npm 패키지명 | `local-git-repo-manager` |
| npm 패키지 버전 | **1.0.0** (`package.json`) |
| 앱/번들 표시 버전 | **0.1.0** (`src-tauri/tauri.conf.json`, `Cargo.toml`과 동일) |

프론트 번들 버전과 앱 번들 버전은 역할이 다르므로 숫자가 일치하지 않을 수 있습니다. 배포 태그를 맞출 때는 위 두 곳을 함께 조정하면 됩니다.

## 요구 사항

- **Node.js** 20 이상
- **Rust** (`rustup`, `cargo`)
- OS별 **Tauri 2** 선행 의존성 — [공식 Prerequisites](https://tauri.app/start/prerequisites/)

## 설치

```bash
npm install
```

## 개발

### Tauri 앱 (Vite + 네이티브 셸)

```bash
npm run tauri dev
```

### 웹 UI만 (브라우저에서 `http://localhost:1420` 등)

```bash
npm run dev
```

### 타입 검사 (선택)

```bash
npx vue-tsc --noEmit
```

### Rust 쪽 확인 (선택)

```bash
cd src-tauri && cargo check
```

## 빌드

### 프론트 정적 산출물 (`dist/`)

```bash
npm run build
```

### Tauri 앱 패키지 (`.app` / `.msi` / `.deb` 등 플랫폼별)

```bash
npm run tauri build
```

## 기술 스택 (현재 `package.json` 기준)

| 영역 | 주요 의존성 |
|------|-------------|
| UI | Vue **^3.5**, Vue Router **^4.5**, vue-i18n **^9.14** |
| 빌드 | Vite **^8**, `@vitejs/plugin-vue` **^6** (Vite 8과 peer 호환) |
| 타입 | TypeScript **^5.6**, vue-tsc **^2.1** |
| 데스크톱 | Tauri **2** (`@tauri-apps/cli` / `api` **^2**), `@tauri-apps/plugin-dialog` **^2.6** |

백엔드 크레이트는 `src-tauri/Cargo.toml`을 참고하세요 (Tauri 2, `tauri-plugin-dialog` 2 등).

## 프로젝트 구조

- `src/` — Vue 앱 (페이지, 컴포넌트, i18n, 라우터)
- `src-tauri/` — Rust/Tauri 명령·설정·번들 메타데이터
- `agents/docs/` — 요구사항·설계·UX 문서
- `agents/tasks/` — 태스크·승인 기록

## 설정/상태 파일 저장 경로

앱은 사용자 홈 디렉터리 아래 `~/.local-git-repo-manager/`에 기본 메타·설정·상태 파일을 저장합니다.

- `bootstrap.json` — 첫 실행 시 확정한 데이터 루트(`confirmed_data_root`)
- `app-settings.json` — 앱 설정(언어, Git 실행 경로, 외부 도구 등)
- `state.json` — 프로젝트 목록 및 루트 폴더 연결 상태(`schemaVersion` 포함)

동작 개요:

- 앱 시작 시 위 파일을 먼저 읽어 초기 상태를 복원합니다.
- `state.json`, `app-settings.json`은 임시 파일(`*.tmp`)에 먼저 쓰고 rename하는 원자적 저장 방식을 사용합니다.
- 저장된 루트 폴더가 실제 디스크에서 사라진 경우, 부팅 시 정리(sanitize)되어 목록에서 제거됩니다.

## npm 참고

의존성 peer 충돌이 나면 `vite`와 `@vitejs/plugin-vue` major를 함께 맞추는 것이 안전합니다. (예: Vite 8 ↔ `@vitejs/plugin-vue` 6.x)
