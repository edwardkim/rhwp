# 단계별 완료보고서 S1 — 최근 문서 저장소 (M100 #2285)

- **이슈**: edwardkim/rhwp#2285
- **브랜치**: `local/task2285`
- **단계**: S1 / 4
- **작성일**: 2026-07-15

## 목표
`FileSystemFileHandle` + 메타를 IndexedDB에 저장/조회하는 최근 문서 저장소 모듈 신설.

## 구현 내용
- 신규 `src/recent/recent-store.ts`
  - DB `rhwpStudioRecent` (ver 1), objectStore `recent` (keyPath `id`).
  - `RecentDoc { id, fileName, sourceFormat, openedAt, handle }`.
  - API: `addRecentDoc`, `listRecentDocs`, `removeRecentDoc`, `clearRecentDocs`.
  - 중복 제거: `handle.isSameEntry()` 우선, 미지원/실패 시 `fileName` 비교.
  - 상한 `MAX_RECENT = 8`, `openedAt` 내림차순 정렬.
  - IndexedDB 미지원 환경 → 메모리 Map fallback (autosave-store.ts 패턴 준용).

## 검증
- `npx tsc --noEmit` 통과 (exit 0).

## 다음 단계
- S2: `loadBytes` 기록 훅 + `index.html` 서브메뉴 마크업 + `menu-bar.ts` 동적 렌더 훅.
