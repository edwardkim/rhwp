# 단계별 완료보고서 S3 — 재열기/지우기 커맨드 (M100 #2285)

- **이슈**: edwardkim/rhwp#2285
- **브랜치**: `local/task2285`
- **단계**: S3 / 4
- **작성일**: 2026-07-15

## 구현 내용

### (a) `src/command/file-system-access.ts`
- `FileSystemFileHandleLike`에 선택적 권한 메서드 추가: `queryPermission?`, `requestPermission?` (반환 `FileSystemPermissionState = 'granted'|'denied'|'prompt'`).

### (b) `src/command/commands/file.ts`
- `ensureReadPermission(handle)`: `queryPermission({mode:'read'})` → 'granted' 아니면 `requestPermission` → 최종 granted 여부 반환. 권한 API 미지원 브라우저는 통과(getFile 위임).
- `file:open-recent` 커맨드:
  - `params.id`로 `listRecentDocs()`에서 레코드 조회(없으면 토스트).
  - 권한 재확인 실패 → "접근 권한 없음" 토스트.
  - `readFileFromHandle` → `open-document-bytes` emit (skipUnsavedGuard 미지정 → 핸들러가 미저장 확인 수행).
  - `getFile()` 실패(파일 이동/삭제) → `removeRecentDoc(id)` + 안내 토스트.
- `file:clear-recent` 커맨드: `confirm` 후 `clearRecentDocs()` + 토스트.
- 두 커맨드는 `fileCommands` 배열에 포함되어 `registry.registerAll`로 자동 등록.

## 검증
- `npx tsc --noEmit` 통과 (exit 0).
- `registry.registerAll(fileCommands)` 경로로 신규 커맨드 등록 확인.

## 다음 단계
- S4: `npm run build` + 개발서버 수동 검증 + 회귀.
