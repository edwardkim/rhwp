# Task M100 #1613 최종 보고서 — rhwp-studio 저장 출력 포맷(HWP/HWPX) 선택 메뉴

- 이슈: #1613 "rhwp-studio: 저장 시 출력 포맷(HWP/HWPX) 사용자 선택 메뉴"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1613`
- 작성일: 2026-06-28

## 1. 개요

#1532(PR #1533)로 HWPX 직접 저장이 활성화됐으나, 저장은 `getSourceFormat()` 기준 출처 포맷으로만
가능했다. 사용자가 출력 포맷(HWP/HWPX)을 직접 선택할 수 있게 한다. HWPX 문서를 HWP 로, HWP 문서를
HWPX 로 저장하는 경로를 메뉴로 노출한다.

## 2. 변경

### `rhwp-studio/src/command/commands/file.ts`

- `saveAsFormat(services, isHwpx)` 공유 헬퍼 — 출력 포맷을 명시 받아 export(`exportHwpx`/
  `exportHwp`)·파일명 확장자·MIME 결정. FS Access picker → 폴백 download 흐름은 file:save-as 동일.
- `file:save-as` 를 `saveAsFormat(services, getSourceFormat()==='hwpx')` 호출로 정리(출처 포맷 유지).
- 신규 명령 `file:save-as-hwp`("HWP 형식으로 저장") / `file:save-as-hwpx`("HWPX 형식으로 저장"),
  `canExecute: ctx.hasDocument`.

### `rhwp-studio/index.html`

- 파일 메뉴에 `file:save-as-hwp` / `file:save-as-hwpx` 항목 2개 추가. 활성/비활성은
  `menu-bar.ts updateMenuStates` 의 `canExecute` 기반 자동 토글.

### `rhwp-studio/e2e/save-as-format.test.mjs` (신규)

- HWP 문서 → HWPX 저장(PK 매직), HWPX 문서 → HWP 저장(CFB 매직) 양방향 e2e.

### `rhwp-studio/src/command/file-system-access.ts` (4단계 — 저장 picker 형식)

- 저장 대화창(`showSaveFilePicker`)이 항상 "HWP 문서(.hwp)" 형식만 노출하던 문제 정정.
- `HWPX_SAVE_PICKER_TYPES`("HWPX 문서", `.hwpx`) 추가, `SaveDocumentOptions.saveAsHwpx` 옵션으로
  포맷별 picker types 선택. `saveAsFormat`/`saveCurrentDocument` 가 `isHwpx` 전달.
- → HWPX 저장 시 대화창에 "HWPX 문서 (.hwpx)" 로 표시. (작업지시자 환경 시각 확인 통과.)

WASM/Rust 변경 없음(기존 `exportHwp`/`exportHwpx` 재사용).

## 3. 검증 결과

| 항목 | 결과 |
|---|---|
| studio `tsc` | 에러 0 |
| `npm test` | 147/147 |
| `npm run build` | 통과 (dist 메뉴 항목 2건 반영) |
| **신규 e2e** `save-as-format` (headless) | HWP→HWPX(MIME application/hwp+zip, PK 매직 50 4B 03 04, 재오픈 6p), HWPX→HWP(MIME application/x-hwp, CFB 매직 D0 CF 11 E0, 재오픈 6p) **전부 PASS** |
| **회귀** `hwpx-direct-save` e2e | 기본 저장(file:save) 무영향, 전부 PASS |
| 저장 picker 형식(4단계) | HWPX 저장 시 "HWPX 문서(.hwpx)" 표시 — 작업지시자 시각 확인 통과 |

→ 매직 바이트로 산출 포맷이 사용자 선택값과 일치함을 직접 확인.

> 참고: studio 가 참조하는 WASM(`@wasm` → `../pkg`)이 stale 이면 exportHwp 가 구버전 동작을
> 할 수 있다. 본 작업 검증 시 devel 기준 WASM 재빌드 후 정상 확인(작업지시자 환경 stale 캐시
> 이슈 해소). studio TS 변경은 pkg 재빌드와 독립이나, exportHwp/exportHwpx 동작은 pkg 의존.

## 4. 영향

- 사용자가 출처와 무관하게 HWP/HWPX 출력 포맷을 메뉴에서 선택 저장 가능.
- 기본 저장(`file:save`/`file:save-as`)은 출처 포맷 유지(회귀 없음).
- rhwp-studio TS/HTML 한정. WASM/Rust 무변경.

## 5. 비고

- HWP↔HWPX 변환 충실도(한컴 호환)는 본 작업 범위 밖이며, 기존 export API 의 직렬화 충실도에
  의존한다. 시각 판정 권위는 작업지시자 환경(`feedback_self_verification_not_hancom`).
- autosave 복구본은 종전대로 HWP(별도 트랙).

## 6. 산출물

- 수행/구현 계획서: `mydocs/plans/task_m100_1613{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1613_stage{1,2}.md`
- 최종 보고서: 본 문서
- 소스: `rhwp-studio/src/command/commands/file.ts`, `rhwp-studio/index.html`,
  `rhwp-studio/e2e/save-as-format.test.mjs`
