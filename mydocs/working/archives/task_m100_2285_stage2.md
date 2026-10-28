# 단계별 완료보고서 S2 — 기록 훅 + 메뉴 동적 렌더 (M100 #2285)

- **이슈**: edwardkim/rhwp#2285
- **브랜치**: `local/task2285`
- **단계**: S2 / 4
- **작성일**: 2026-07-15

## 구현 내용

### (a) `src/main.ts` — `loadBytes()` 기록 훅
- 문서 초기화 후 `fileHandle`이 있을 때만 `addRecentDoc({ fileName, sourceFormat, handle })` 호출.
- 실패는 `console.warn`으로만 처리(문서 로드에 영향 없음).

### (b) `index.html` — 파일 메뉴 서브메뉴
- "열기" 아래 `.md-sub[data-recent]` "최근 문서" 추가. 패널 `#recent-docs-panel`은 동적 렌더 대상.

### (c) `src/ui/menu-bar.ts` — 메뉴 open 훅
- `MenuBarOptions.onMenuOpen?(menuName, menuEl)` 추가.
- 타이틀 클릭/hover로 메뉴가 열릴 때 `updateMenuStates` 직후 콜백 호출.

### (d) `src/main.ts` — `renderRecentSubmenu()`
- 메뉴 open(`menuName==='file'`) 시 `listRecentDocs()`로 `#recent-docs-panel` 재렌더.
- 항목: `data-cmd="file:open-recent" data-id` + 파일명(라벨) + 형식(우측). `title`로 파일명 표시.
- 하단 구분선 + "최근 문서 목록 지우기"(`file:clear-recent`).
- 목록이 비면 "(최근 문서 없음)" + 서브메뉴 자체 `disabled` 처리(스테일 판정 보정).
- 파일명은 `textContent`로 삽입(HTML 인젝션 방지).

## 검증
- `npx tsc --noEmit` 통과 (exit 0).

## 비고
- 커맨드(`file:open-recent`/`file:clear-recent`) 실체는 S3에서 추가. 현 단계에서 메뉴 렌더까지 동작.

## 다음 단계
- S3: 재열기/지우기 커맨드 + 핸들 권한 재확인·실패 처리.
