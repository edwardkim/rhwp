# Stage 2 완료보고서 — embed RPC notifySaved (M100 #2660)

- **이슈**: [edwardkim/rhwp#2660](https://github.com/edwardkim/rhwp/issues/2660)
- **브랜치**: `local/task2660`
- **구현계획서**: [`../plans/task_m100_2660_impl.md`](../plans/task_m100_2660_impl.md)
- **작성일**: 2026-07-21

## 1. 구현 내용

TDD(red→green). RED에서 3개 테스트가 의도한 이유(capability 부재 / `Unknown method:
notifySaved` / main.ts 핸들러 부재)로 실패함을 확인한 뒤 구현했다.

### `src/embed/protocol.ts`

- `EMBED_CAPABILITIES`에 `'notify-saved-v1'` 추가 (4번째 항목).
- `EMBED_PROTOCOL_VERSION = 1` 유지 — 순수 additive 확장.

### `src/embed/rpc-router.ts`

- `EmbedNotifySavedResult { ok: true; wasDirty: boolean }` 타입 추가.
- `EmbedRpcHandlers.notifySaved(fileName?: string)` 추가.
- `case 'notifySaved'`: fileName은 **비어있지 않은 문자열만** 통과, 그 외
  (비문자열/빈 문자열/누락)는 `undefined`로 정규화.
- v1 MessagePort·legacy 경로 모두 `routeEmbedRequest` 공유 — `runtime.ts` 무수정.

### `src/main.ts`

- 임베드 핸들러 블록에 `notifySaved(fileName)` 배선 — `await initPromise` 후
  Stage 1의 `completeHostSave(fileName)` 호출 (코어 단일화).

## 2. 테스트 (RED → GREEN)

| 테스트 (`tests/embed-protocol.test.ts`) | RED | GREEN |
|---|---|---|
| capability deepEqual에 `'notify-saved-v1'` 추가 | 실패 (protocol 미반영) | 통과 |
| "embed router는 notifySaved fileName을 정규화해 핸들러로 전달한다 (#2660)" — 4케이스(생략/'a.hwp'/123/'') | 실패 (`Unknown method`) | 통과 |
| #2660 소스 계약 테스트에 embed 핸들러 정규식 추가 (`async notifySaved(fileName)` → `completeHostSave(fileName)`) | 실패 (핸들러 부재) | 통과 |

`EmbedRpcHandlers` 완전 타입 리터럴 목 3곳(라우터/런타임/suppressDialogs 테스트)에
`notifySaved` 목을 추가해 인터페이스 확장 누락이 타입으로 강제 검출되게 했다.

## 3. 검증 결과

- `npm test`: **456개 중 454 통과, 실패 2** — Stage 1 보고서와 동일한 기존 로컬 환경
  이슈 2건(`cell-flow-boundary`, `canvaskit-resource-key`; Windows spawnSync 문제,
  HEAD에서도 동일 재현 확인됨). 로그 `$TMPDIR/task2660_s2_green.log`.
- `npx tsc --noEmit`: 변경 전 베이스라인과 **diff 완전 동일(IDENTICAL)** — 신규 타입
  오류 0건.

## 4. 다음 단계

Stage 3 — `@rhwp/editor` SDK (`transport.js` CAPABILITIES, `index.js`
`notifySaved()` capability 게이팅, `index.d.ts`, 계약 테스트). 승인 후 진행.
