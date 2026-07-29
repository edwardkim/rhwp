# Stage 1 완료보고서 — 코어 + window API + 경합 방어 (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660`
- **구현계획서**: [`../plans/task_m100_2660_impl.md`](../plans/task_m100_2660_impl.md)
- **작성일**: 2026-07-21

## 1. 구현 내용

TDD(red→green)로 진행했다. 테스트를 먼저 작성해 실패를 확인한 뒤 구현했다.

### `src/recovery/autosave-manager.ts` — flush↔discard 경합 방어

- `discardGeneration` 카운터 추가. `discardCurrentDraft()` 진입 시 증가.
- `flushNow()`는 `store.saveDraft()` 직전 세대를 캡처하고, 저장 완료 후 세대가
  변했으면 방금 저장으로 부활한 draft를 `deleteDraft(..., 'discarded-during-save')`로
  재삭제하고 `lastSavedAt`/`saved` 상태 콜백을 생략한다.

### `src/main.ts` — 코어 + window 공개 API

- `completeHostSave(fileName?)`: `wasDirty` 캡처 → (선택) `wasm.fileName` 갱신 →
  `markClean('host-save')` → **`await autosaveManager.discardCurrentDraft('host-save')`**
  → `{ ok: true, wasDirty }` 반환. 삭제 완료를 await하므로 resolve 이후 팝업
  `window.close()`에 IndexedDB 삭제가 잘리지 않는다.
- `window.rhwpStudio = { notifySaved }` — DEV 전용 `__*` 노출과 달리 프로덕션 포함
  항상 노출 (SDK 없이 스튜디오 페이지 안에서 통합하는 팝업/포크 호스트용).

## 2. 테스트 (RED → GREEN)

| 테스트 | RED (구현 전) | GREEN (구현 후) |
|---|---|---|
| `tests/autosave-manager.test.ts` "저장 진행 중 discard가 끼어들면 저장 완료로 부활한 draft를 재삭제한다" | 실패 — ops가 `['delete','save']`로 끝나 draft 부활 잔존 | 통과 — `['delete','save','delete']` |
| `tests/autosave-manager.test.ts` "discard 없는 정상 flush에서는 draft를 삭제하지 않는다" (회귀 가드) | 통과(기존 동작 확인용) | 통과 |
| `tests/embed-protocol.test.ts` "main.ts는 호스트 저장 완료 API completeHostSave를 window.rhwpStudio로 노출한다 (#2660)" | 실패 — API 부재 | 통과 |

## 3. 검증 결과

Node v24.16.0 (`node --test *.ts` 타입 스트리핑 요구사항), 로그
`$TMPDIR/task2660_stage1_test.log`.

- `npm test`: **455개 중 453 통과, 실패 2**
  - 실패 2건(`cell-flow-boundary.test.ts`, `canvaskit-resource-key.test.ts`)은
    **본 변경과 무관한 기존 로컬 환경 이슈** — `git stash`로 원복한 HEAD에서도 동일
    실패 재현 확인. 원인: Windows에서 `spawnSync('node_modules/.bin/tsc')` 실행 불가
    (`compilation.status === null`). CI(Linux)에서는 통과하는 테스트.
- `npx tsc --noEmit`: 오류 7줄 — **HEAD와 diff 완전 동일(IDENTICAL)**, 본 변경 파일
  관련 0건. 원인: 로컬 `pkg/` WASM 타입 구본 + `@noble/hashes` 로컬 미설치.
- 커밋 대상 파일: `src/recovery/autosave-manager.ts`, `src/main.ts`,
  `tests/autosave-manager.test.ts`, `tests/embed-protocol.test.ts`.

## 4. 관찰 사항 (범위 외 기록)

- `beginDocument({ discardPreviousDraft: true })` 경로에도 유사한 부활 경합이
  잠재한다(진행 중 flush가 이전 draftId를 재저장). 이 경로는 `discardCurrentDraft`를
  거치지 않아 이번 세대 카운터의 보호를 받지 않는다. 발생 조건이 문서 교체 순간으로
  좁고 본 타스크 범위(호스트 저장 통지) 밖이므로 기록만 남긴다. 필요 시 후속 이슈로
  분리 권고.

## 5. 다음 단계

Stage 2 — embed RPC (`protocol.ts` capability, `rpc-router.ts` 인터페이스·라우팅,
`main.ts` 핸들러, `embed-protocol.test.ts` 갱신). 승인 후 진행.
