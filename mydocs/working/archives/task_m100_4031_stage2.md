# Task M100 #4031 Stage 2 완료보고서 — 셀 Enter command-aware cancellation

## 1. 구현

Stage 1(§5)에서 확정한 경로에 최소 변경 3점을 넣었다.

### 1.1 admission — `isCommittedCellEnterSplit` (input-handler-keyboard.ts)

이 keydown이 같은 함수 하단 `case 'Enter'`에서 `SplitParagraphInCellCommand`로 확정
실행되는 좁은 조건만 통과시킨다. flush 지점과 `case 'Enter'` 사이의 조기 분기 전부를
배제한다: modifier(Shift/Ctrl/Meta/Alt), IME 조합, form mode, 머리말/꼬리말, 각주,
그림/표 객체 선택, 블록/셀 선택 모드, 선택 영역(deleteSelection 선행 방지), 그리고
`isInCell()`. 하나라도 확신할 수 없으면 false → 기존 full flush로 fail-closed.

`Enter`는 `PAGINATION_BOUNDARY_KEYS`에서 제거하지 않았다(구현 원칙 1·2). 판정 결과는
같은 keydown 함수 스코프의 지역 상수 `committedCellEnterSplit`로 전달해 인스턴스 상태
누수가 없다.

### 1.2 계산 없는 취소 (input-handler.ts)

`cancelDeferredPaginationForOwnedMutation()`: idle flush timer 취소 + runner 취소
(`runner.cancel()`이 전진 중이면 `wasm.cancelDeferredPagination()`까지 수행).
`wasm.flushDeferredPagination()`을 호출하지 않는 것이 flush 경로와의 유일한 차이.
`deferredPaginationPending`은 유지 — split 실패 시 다음 boundary flush가 기존 barrier
의미론으로 복구한다(fail-closed).

### 1.3 완료 소유 — command effects 선언 (command.ts)

`SplitParagraphInCellCommand.execute`가 native split 성공 뒤
`IMMEDIATE_TEXT_MUTATION_EFFECTS`(paginationCompleted)를 선언한다. 기존
`executeOperation → prepareTextMutationBeforeCursor` effects 경로가 pending 해소·runner
취소·focused cursor geometry invalidation을 수행하므로, 직후 `afterEdit`의
`before-full-edit` flush가 no-op 판정(shouldFlush=false)으로 wasm 호출 없이 끝난다.
예외 시 effects는 `NO_TEXT_MUTATION_EFFECTS`로 남아 아무것도 해소하지 않는다.

처음에는 keydown에서 명시적 완료 메서드를 호출했으나, e2e가 split 직후 `afterEdit`의
`before-full-edit` flush 1회(native no-op이지만 wasm 호출)를 검출했다 — 완료 선언이
executeOperation의 refresh보다 늦었기 때문이다. effects 선언은 이 순서 문제를 기존
계약 안에서 해소하며 undo/redo·비admitted 경로에도 동일하게 정합적이다.

### 1.4 배선 (input-handler-keyboard.ts)

- boundary key 지점: admitted면 취소, 아니면 기존
  `flushDeferredPaginationIfNeeded('before-navigation')`.
- `case 'Enter'`의 inCell 분기: 예외 시
  `flushDeferredPaginationIfNeeded('cell-enter-split-fallback')` 후 rethrow.
  fallback은 `committedCellEnterSplit`가 참일 때만 수행한다.

### 1.5 IME 경로는 의도적으로 비대상

`processPendingNav`의 `code === 'Enter'`는 조합 확정만 하고 structural command가 없다
(stage1 §5). 그 flush는 최신 모델의 유일한 pagination barrier이므로 유지한다. 취소하면
115쪽 문서에서 idle auto-flush 상한 밖이라 pagination이 runner 완주까지 표류한다.
이슈의 "direct와 IME를 같은 계약으로"는 "admission을 증명한 경우에만 flush를 생략"이라는
같은 규칙의 적용이며, IME 경로는 admission(구조 명령 확정)이 성립하지 않는다.

## 2. 검증

| 게이트 | 결과 |
|---|---|
| `tests/cell-enter-owned-pagination.test.ts` (신규 6 계약) | 6 passed |
| Studio `npm test` 전체 | 769 passed, 0 failed |
| `npx tsc --noEmit` (wasm pkg 빌드 후) | 통과 |
| `npm run build` (tsc + vite build) | 통과 |
| `cargo clippy --profile release-test --test issue_4031_enter_latency_probe -- -D warnings` | 통과 |
| production wasm + headless Chrome e2e | §4 |

계약 테스트 6종: ① boundary flush의 admitted/비admitted 분기, ② admission guard 전수,
③ split 성공→effects 완료 선언·실패→fallback, ④ effects paginationCompleted의
pending 해소·runner 취소·geometry invalidation, ⑤ 소유 취소의 pending 유지(fail-closed),
⑥ IME 경로 flush 보존.

## 3. 기대 효과 (Stage 1 native 실측 기준)

pending cell Enter = full pagination 2회 → 1회. 시나리오 B 실측으로 최종 page count가
flush-then-split과 완전 일치하고 사후 flush가 0.1ms no-op임을 확인했다. 브라우저
전 구간(direct/IME × HWP/HWPX × 3회)과 acceptance criteria(중앙값 cold 110% 이내 등)는
Stage 3에서 production wasm으로 측정한다.
