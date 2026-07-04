# Task M100 #1448 구현계획서

- 이슈: #1448
- 수행계획서: `mydocs/plans/task_m100_1448.md`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1448`
- 작성일: 2026-06-21

## 1. Stage 1 목표

미저장 문서 자동 복구의 기반을 만든다. 이번 단계는 사용자에게 복구 모달을 보여주기 전에, dirty 문서를 안전하게
로컬 스냅샷으로 남기고 정상 저장/폐기 시 정리할 수 있는 저장소와 매니저를 구축하는 데 집중한다.

## 2. 관련 코드

| 파일 | 역할 |
|---|---|
| `rhwp-studio/src/core/document-dirty-state.ts` | dirty/clean 상태와 `beforeunload` 경고 |
| `rhwp-studio/src/command/commands/file.ts` | 수동 저장, 문서 교체 전 저장 확인 |
| `rhwp-studio/src/ui/unsaved-changes-dialog.ts` | 저장하지 않은 변경사항 확인 모달 |
| `rhwp-studio/src/history/idb-store.ts` | 문서 비교 이력 IndexedDB 예시 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `exportHwp`, `loadDocument`, 파일명/파일 핸들 상태 |
| `rhwp-studio/src/main.ts` | 문서 초기화, dirty 이벤트 연결, 파일 로드 진입점 |

## 3. 신규 구조

### 3.1 `recovery/autosave-store.ts`

자동 백업 전용 IndexedDB 저장소를 추가한다.

- DB 이름 후보: `rhwpStudioAutosave`
- store 이름 후보: `drafts`
- row key: 현재 세션/문서 단위 `draftId`
- 저장 필드:
  - `id`
  - `fileName`
  - `sourceFormat`
  - `savedAt`
  - `byteLength`
  - `data`
  - `dirtyReason`

문서 비교 이력 DB(`rhwpStudioDocHistory`)와 이름과 API를 분리한다.

### 3.2 `recovery/autosave-manager.ts`

dirty 이벤트와 저장소를 연결하는 매니저를 추가한다.

- dirty 이벤트 수신 시 debounce 타이머를 건다.
- 이미 저장 중이면 다음 저장 요청을 pending 상태로 둔다.
- 저장 시점에는 `wasm.exportHwp()`로 복구 스냅샷을 만든다.
- dirty 상태가 clean으로 전환되면 정상 저장으로 판단해 현재 draft를 정리한다.
- 문서가 새로 로드되면 draft id를 새로 만든다.

Stage 1에서는 복구 UI 없이 저장/정리 API를 먼저 안정화한다.

## 4. main.ts 연결 지점

- `DocumentDirtyState` 생성 직후 autosave manager를 생성한다.
- `document-mutated`, `document-changed` → dirty 전환 뒤 autosave 예약.
- `initializeDocument` 또는 `loadBytes` 완료 후 현재 문서 메타데이터를 autosave manager에 전달한다.
- `saveCurrentDocument` 성공 후 clean 전환이 발생하면 autosave manager가 draft를 삭제한다.
- `confirmSaveBeforeReplacingDocument`에서 사용자가 `저장 안 함`을 선택한 경우에도 draft 정리 경로가 필요하다.

## 5. Stage 1 테스트 계획

### 단위 테스트

- IndexedDB 사용 가능 시 draft 저장/조회/삭제
- IndexedDB 사용 불가 시 메모리 폴백 저장/조회/삭제
- debounce 후 한 번만 저장되는지
- clean 전환 시 draft 삭제

### 기존 회귀 테스트

- `cd rhwp-studio && npm test`
- `cd rhwp-studio && npx tsc --noEmit`

## 6. Stage 1 구현 제외

- 앱 시작 시 복구 모달
- 복구 후보 목록 UI
- 복구본 로드 UX
- HWPX 출처 전용 안내 문구
- 대용량 문서 저장 주기 튜닝

## 7. 승인 필요 사항

Stage 1 구현 전에 다음 정책을 확정한다.

1. 자동 백업 저장 주기 기본값: debounce 2초 + 최소 저장 간격 10초.
2. 복구 스냅샷 형식: 우선 `exportHwp()` 결과 HWP 바이트.
3. 정상 저장 또는 `저장 안 함` 선택 시 해당 문서 draft 삭제.
4. 앱 시작 복구 UI는 Stage 2에서 별도 구현.
