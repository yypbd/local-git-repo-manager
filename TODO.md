# TODO

> **문서 역할**: 구현·백로그·세부 체크리스트. Epic·큰 흐름은 [`agents/tasks/TASKS.md`](agents/tasks/TASKS.md).

**섹션 순서**: 대기/백로그 → 진행 중 → 완료.

---

## 대기 / 백로그

### 레이아웃 — 툴바 (`ProjectFolderPanel.vue`)

- [x] **A-1** 폴더 목록 패널 상단 툴바는 **버튼 3개·그룹 2개·구분선 1개**: [폴더 추가 · 폴더 정보 갱신] \| [선택 항목 삭제]. 프로젝트 간 **이동**, **Git 초기화·저장소 해제**는 목록 선택 시 **폴더 상세**( `FolderSelectionDetail` )에서 처리한다—구버전 메모의 «6버튼·3그룹» 설계와 혼동하지 않을 것.
- [x] **A-2** 표시·필터·정렬은 액션 버튼과 같은 상단 줄에서 **우측** 배치(`folder-top-bar`: 좌 버튼 · 우 필터), 접근성 `folder-filter-bar`에 `role="group"` + `workspace.folderFilterBarGroup`

### 폴더 목록 패널

- [ ] 저장소 clone 기능: URL·경로 입력 후 clone하고 프로젝트에 추가

### 공통 — 디자인

- [ ] 테마 설정: light, dark

---

## 진행 중

<!-- 현재 손대는 항목이 있으면 아래에 추가. 없으면 비움. -->

*(없음)*

---

## 완료

### shadcn-vue / Radix Vue / Tailwind 마이그레이션

- [x] Tailwind CSS + postcss (`tailwind.config.js`, `postcss.config.js`)
- [x] CSS 변수·`@tailwind` 레이어 (`base.css`)
- [x] 프리미티브: `Button`, `Input`, `Textarea`, `Select`, `AlertDialog` 세트, `Tabs/`
- [x] 다이얼로그 Radix Vue 마이그레이션
- [x] `tailwind.config.js` `darkMode: false` 경고 제거

### 미사용 컴포넌트 제거

- [x] `DeleteConfirmDialog.vue`, `ContextMenu.vue`, `GitLogPanel.vue` 삭제
- [x] `DialogDescription.vue`, `Dialog/index.ts` 삭제

### Ui* 래퍼 제거

- [x] `UiButton` → `Button` 등 교체, variant 정규화, `Ui*.vue` 삭제

### 레이아웃 B — 하단 상세 패널 통합

- [x] **B-1** Remote / 아카이브 / .gitignore → `FolderSelectionDetail` 기본 탭 하단
- [x] **B-2** `ProjectDetailPanel` 제거·`ProjectFolderPanel` 통합·`AppShell` 슬롯 단순화
- [x] **B-3** 기본 탭 Status 필드 제거·AppShell CSS 정리

### 레이아웃 C — `AppShell.vue`

- [x] **C-1** `col-folders-area`가 우측 전체
- [x] **C-2** 폴더 미선택 시 `.folder-detail-pane` 숨김 → 목록 전체 높이

### 폴더 목록 D·E

- [x] **D-1** 정렬: 폴더명 / 경로 / 상태
- [x] **D-2** 정렬 Select + 오름·내림차·localStorage
- [x] **E-1** 폴더명 말줄임
- [x] **E-2** 2행 레이아웃(이름+상태 / 경로+리모트)

### 성능·로딩 UI

- [x] 다수 루트: `folder_root_row` 캐시·강제 갱신 시 무효화 — `useFolderRootRows.ts`
- [x] 우선 로드: 앞쪽 24개 청크
- [x] 진행 표시: `loadedCount`/`totalCount` + progress bar — `ProjectsWorkspacePage.vue`, i18n

### 폴더 행 상태 (FOLDER-STATUS-01 ~ 05)

| 단계 | ID | 내용 |
|------|-----|------|
| ✓ | 01 | 비Git: `data-state="non-git"`, 좌측 바·배경 (`RootPathsList`) |
| ✓ | 02 | Git·remote 없음: `no-remote`, 경로 아래 문구 등 |
| ✓ | 03 | dirty: `WorkingTreeStatusLabel` 레이블·툴팁·배지 |
| ✓ | 04 | `--folder-status-*` 토큰·범례 정리 후 범례 칩 제거 |
| ✓ | 05 | `folderRowAriaLabel`, `aria-hidden` 중복 완화 |

**데이터**: `FolderRootRow` (`gitError`, `clean`, `remote`, `trackedChanges`, `untrackedFiles` — `useFolderRootRows.ts`).

### 프로젝트 패널 (요약)

- [x] 목록 행 UI(`folderCount`), 문구·i18n 정리
- [x] DnD 순서·`projects_reorder`, 삭제 시 폴더 이동·동명 방지
- [x] 프로젝트 패널 폴더 드롭·`projects_import_folder_drop` 등

### 폴더 목록 패널 (요약)

- [x] 타 프로젝트 경로 추가, Ctrl/Shift 멀티 선택·삭제
- [x] 이름순, 아이콘/목록 뷰, 필터

### 상세 패널·공통·워크스페이스 (요약)

- [x] gitignore, 외부 도구, 아카이브, remote 관리, 하단 버튼 가드
- [x] 앱 명칭·아이콘·최소 창·드래그·다이얼로그 백드롭·포커스·Esc/Enter
- [x] 2열 레이아웃, 멀티 폴더 추가

### 기타 (임시·정리 완료로 표기했던 항목)

- [x] 앱 아이콘(임시), 우클릭·텍스트 선택 관련 정리

---

## 참고

### 체크리스트 사용법

- 미완료 `- [ ]`, 완료 `- [x]`
- ID 예: `TODO-001`, `FOLDER-STATUS-01` (`agents/tasks/TASKS.md`의 `TASK-*`와 구분)

### 변경 이력 (요약)

| 날짜 | 내용 |
|------|------|
| 2026-04-21 | `FOLDER-STATUS-01`~`05`, 세부 TODO 루트 `TODO.md` 통합 |
| 2026-04-09 | 폴더 용어·i18n, 루트 행 캐시·로드 진행률 |
| 2026-04-08 | 다이얼로그 UX, 동명 프로젝트 방지 |
| 2026-04-02 ~ 03 | TASKS/TODO 분리, 멀티 폴더·DnD·필터 등 |

상세 로그: Git 히스토리.
