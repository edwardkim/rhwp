# 구현계획서 — 파일 메뉴 '최근 문서' 서브메뉴 (M100 #2285)

- **이슈**: [edwardkim/rhwp#2285](https://github.com/edwardkim/rhwp/issues/2285)
- **브랜치**: `local/task2285`
- **수행계획서**: [`task_m100_2285.md`](task_m100_2285.md)
- **작성일**: 2026-07-15

## 개요

파일 메뉴에 "최근 문서" 서브메뉴를 추가한다. `FileSystemFileHandle`을 IndexedDB에 저장하고, 메뉴 open 시 동적 렌더, 항목 클릭 시 권한 재확인 후 재열기한다. 총 4단계.

---

## S1 — 최근 문서 저장소 (`src/recent/recent-store.ts` 신규)

`autosave-store.ts` 패턴을 준용한 IndexedDB CRUD 모듈.

- DB `rhwpStudioRecent` (ver 1), objectStore `recent` (keyPath `id`).
- 레코드 인터페이스:
  ```ts
  interface RecentDoc {
    id: string;            // crypto.randomUUID
    fileName: string;
    sourceFormat: string;  // 'hwp' | 'hwpx' | 'hml'
    openedAt: number;      // Date.now()
    handle: FileSystemFileHandleLike;  // structured-clone 저장
  }
  ```
- API:
  - `addRecentDoc({ fileName, sourceFormat, handle })` — 중복 제거 후 맨 앞 삽입, 상한 `MAX_RECENT = 8` 유지.
    - 중복 판정: 기존 목록을 순회하며 `handle.isSameEntry?.(existing.handle)` true 이거나(비동기), 미지원 시 `fileName` 동일 → 기존 레코드 삭제 후 신규 삽입.
  - `listRecentDocs(): RecentDoc[]` — `openedAt` 내림차순.
  - `removeRecentDoc(id)`.
  - `clearRecentDocs()`.
- IndexedDB 미지원(테스트/제한 환경) → 메모리 Map fallback (autosave-store와 동일 구조). 단, 핸들은 structured-clone 불가 환경에서만 메모리 유지.
- `id` 생성 헬퍼는 `crypto.randomUUID?.() ?? fallback` (autosave의 `createAutosaveDraftId` 패턴).

**검증**: 타입 컴파일. (단독 단위테스트는 IndexedDB 필요로 S4 통합 수동검증에서 확인.)

---

## S2 — 기록 훅 + 메뉴 마크업 + 동적 렌더

### (a) `src/main.ts` — `loadBytes()` 기록 훅
- `loadBytes` 성공 경로 끝(문서 초기화 후)에서, `fileHandle`이 truthy 이면
  `void addRecentDoc({ fileName, sourceFormat: wasm.getSourceFormat(), handle: fileHandle })`.
- 핸들 없으면(드롭/input/복구) 기록 안 함.

### (b) `index.html` — 파일 메뉴에 서브메뉴 추가
"열기"(line 25) 바로 아래 삽입:
```html
<div class="md-sub" data-recent>
  <span class="md-icon"></span><span class="md-label">최근 문서</span><span class="md-arrow">▶</span>
  <div class="md-sub-panel" id="recent-docs-panel">
    <div class="md-item disabled"><span class="md-icon"></span><span class="md-label">(최근 문서 없음)</span></div>
  </div>
</div>
```

### (c) `src/ui/menu-bar.ts` — 메뉴 open 훅
- 생성자에 선택적 콜백 `onMenuOpen?: (menuName: string, menuEl: HTMLElement) => void` 추가.
- `setupTitleClicks`/`setupTitleHover`에서 메뉴가 열릴 때 `this.onMenuOpen?.(item.dataset.menu ?? '', item)` 호출 (기존 `updateMenuStates` 직후).
- `main.ts` MenuBar 생성부(362)에서 콜백 전달: `menuName === 'file'` 이면 `renderRecentSubmenu()` 호출.
- `renderRecentSubmenu()`(main.ts 신설): `listRecentDocs()`로 `#recent-docs-panel` 재렌더.
  - 목록 있으면 각 항목 `<div class="md-item" data-cmd="file:open-recent" data-id="...">` + 파일명(+형식) 라벨. 하단 `md-sep` + "목록 지우기"(`file:clear-recent`).
  - 비면 "(최근 문서 없음)" 비활성 항목만.
  - 비동기 렌더이므로 open 시 즉시 이전 내용 유지 → 완료 시 교체(깜빡임 최소화).

**검증**: 타입 컴파일 + 개발서버에서 메뉴 열림 시 패널 렌더 확인.

---

## S3 — 재열기/지우기 커맨드 (`src/command/commands/file.ts`)

### (a) `FileSystemFileHandleLike` 권한 메서드 확장 (`file-system-access.ts`)
- 인터페이스에 선택적 메서드 추가:
  ```ts
  queryPermission?(desc?: { mode?: 'read' | 'readwrite' }): Promise<PermissionState>;
  requestPermission?(desc?: { mode?: 'read' | 'readwrite' }): Promise<PermissionState>;
  ```

### (b) `file:open-recent` 커맨드
- params `id`로 `listRecentDocs()`에서 레코드 조회(없으면 무시).
- 권한: `handle.queryPermission?.({mode:'read'})` → `'granted'` 아니면 `handle.requestPermission?.({mode:'read'})`. 최종 `'granted'` 아니면 → 토스트 "권한이 거부되어 열 수 없습니다" + 종료(목록 유지).
- `readFileFromHandle(handle)` → `open-document-bytes` emit(`bytes`, `fileName`, `fileHandle: handle`, `skipUnsavedGuard: false`) — 기존 `file:open`과 동일 규약(미저장 확인은 open-document-bytes 핸들러가 처리).
- 실패(파일 이동/삭제 → `getFile()` throw): 토스트 "파일을 찾을 수 없어 목록에서 제거했습니다" + `removeRecentDoc(id)`.

### (c) `file:clear-recent` 커맨드
- 확인(간단 `confirm` 또는 토스트 확인) 후 `clearRecentDocs()` + 토스트.

**검증**: 타입 컴파일 + 개발서버 재열기/권한/실패/지우기 흐름.

---

## S4 — 빌드 · 수동검증 · 회귀

- `npm run build` (tsc 타입체크 + vite build) 통과.
- 개발서버(`vite`)에서 시나리오 검증:
  1. File System Access로 문서 열기 → "최근 문서"에 등재
  2. 새로고침 후 목록 유지
  3. 항목 클릭 → 권한 재확인 → 재열기
  4. 파일 이동/삭제 후 클릭 → 실패 안내 + 목록 제거
  5. 드롭/파일선택으로 연 파일 → 목록에 미등재
  6. "목록 지우기" 동작
- 기존 e2e 무영향 확인(관련 e2e 스모크).

---

## 커밋 계획

- S1: `Task #2285: 최근 문서 IndexedDB 저장소 추가` (recent-store.ts + stage1 보고서)
- S2: `Task #2285: 최근 문서 기록 훅 + 메뉴 동적 렌더` (main.ts, index.html, menu-bar.ts + stage2)
- S3: `Task #2285: 최근 문서 재열기/지우기 커맨드` (file.ts, file-system-access.ts + stage3)
- S4: `Task #2285: 빌드·검증 + 최종 보고서` (report + orders 갱신은 작업지시자 관할이라 제외)

## 영향 파일

| 파일 | 단계 | 변경 |
|------|------|------|
| `src/recent/recent-store.ts` | S1 | 신규 |
| `src/main.ts` | S2 | `loadBytes` 훅 + `renderRecentSubmenu` |
| `index.html` | S2 | 서브메뉴 마크업 |
| `src/ui/menu-bar.ts` | S2 | `onMenuOpen` 콜백 |
| `src/command/commands/file.ts` | S3 | `file:open-recent`, `file:clear-recent` |
| `src/command/file-system-access.ts` | S3 | 핸들 권한 메서드 타입 |
