# Task M100 #1613 4단계(v2) 완료보고서 — 저장 picker 형식 포맷 정합

- 이슈: #1613
- 브랜치: `local/task1613`
- 작성일: 2026-06-28
- 단계: 4 (저장 대화창 형식 정정)

## 배경 (작업지시자 제보)

HWP 파일을 열고 "HWPX 형식으로 저장" 시, 저장 대화창의 파일 형식이 "HWP 문서"로 표시됨.
(저장 내용/확장자는 정상이나, picker 형식 필터가 HWP 로 고정.)

## 원인

`saveDocumentToFileSystem` 이 `showSaveFilePicker` 에 항상 `HWP_SAVE_PICKER_TYPES`
(`description: 'HWP 문서', accept: {'application/x-hwp': ['.hwp']}`)를 넘겼다. 출력 포맷과
무관하게 HWP 형식만 picker 에 노출되어, HWPX 저장 시에도 "HWP 문서(.hwp)" 로 표시됐다.

## 변경

### `rhwp-studio/src/command/file-system-access.ts`
- `HWPX_SAVE_PICKER_TYPES` 추가(`description: 'HWPX 문서', accept: {'application/hwp+zip': ['.hwpx']}`).
- `SaveDocumentOptions.saveAsHwpx?: boolean` 추가.
- `saveDocumentToFileSystem` 이 `saveAsHwpx` 면 HWPX picker types, 아니면 HWP picker types 사용.

### `rhwp-studio/src/command/commands/file.ts`
- `saveAsFormat`(save-as / save-as-hwp / save-as-hwpx) 이 `saveAsHwpx: isHwpx` 전달.
- `saveCurrentDocument`(file:save) 도 `saveAsHwpx: isHwpx` 전달(첫 저장 picker 일관).

## 검증

| 항목 | 결과 |
|---|---|
| studio `tsc` | 에러 0 |
| `npm test` | 147/147 |
| HWPX picker types 반영 | 확인 |

→ HWPX 저장 시 저장 대화창에 "HWPX 문서 (.hwpx)" 로 표시된다. WASM/Rust 무변경.

## 비고

- 실제 picker 표시는 브라우저 FS Access UI 라 작업지시자 환경 수동 확인 권장.
- file:save 의 currentFileHandle 직접 쓰기 경로는 picker 미표시(영향 없음). 첫 저장/HWPX 출처
  picker 경로만 형식이 정합화된다.
