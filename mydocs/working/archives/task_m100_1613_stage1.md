# Task M100 #1613 1단계 완료보고서 — 포맷 인자 저장 헬퍼 + 명시 저장 명령

- 이슈: #1613
- 브랜치: `local/task1613`
- 작성일: 2026-06-28
- 단계: 1/3

## 변경 내용

`rhwp-studio/src/command/commands/file.ts`:

1. `saveAsFormat(services, isHwpx)` 공유 헬퍼 추가
   - 출력 포맷(isHwpx)을 명시 받아 export(`exportHwpx`/`exportHwp`)·파일명 확장자·MIME 결정.
   - FS Access picker(forceSaveAs) → 폴백 download 흐름은 기존 file:save-as 와 동일.
   - 출처 무관 양방향 저장(HWP 문서→HWPX, HWPX 문서→HWP) 가능.

2. 기존 `file:save-as` execute 를 `saveAsFormat(services, getSourceFormat()==='hwpx')` 호출로 정리.
   - 출처 포맷 유지 동작 보존(회귀 없음).

3. 신규 명령 2개:
   - `file:save-as-hwp` (label "HWP 형식으로 저장") → `saveAsFormat(services, false)`.
   - `file:save-as-hwpx` (label "HWPX 형식으로 저장") → `saveAsFormat(services, true)`.
   - `canExecute: ctx.hasDocument`.

## 검증

- studio `tsc`: file.ts 에러 0 (canvaskit-wasm 미설치는 무관·기존).
- WASM/Rust 변경 없음(기존 export API 재사용).

## 다음 단계

2단계: index.html 파일 메뉴에 두 항목 추가 + 문서 유무 활성/비활성 토글 확인.
