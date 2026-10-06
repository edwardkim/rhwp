# Task M100 #2214 Stage 4 완료보고서 — pre-cursor 경계 flush와 Studio 정합

## 0. 판정 요약

- **Stage 판정**: 완료
- **production 변경 범위**: Studio WASM bridge, command/history effect, 일반·IME·iOS 입력 router
- **Rust 변경**: 없음. Stage 3의 `cellFlowChanged`와 scoped cache coherence를 그대로 사용
- **정확성 결과**: HWP/HWPX 각 3회에서 44번째 경계 flush 1회, cursor lookup 전 완료, 115쪽·exact tree/cursor·bounds 971.5 유지
- **stable 성능 결과**: operation p95 27.9~29.0ms, keyboard p95 47.0~50.6ms
- **boundary 성능 결과**: handler 926~969ms, 그중 full flush 881~924ms. 1.5초 관찰 기준 이내
- **raw 입력 결과**: HWP/HWPX의 IME/iOS stable 입력은 flush 0회, flow 경계는 1회
- **독립 리뷰**: production, E2E, 계획 대조 리뷰에서 남은 P0/P1 없음
- **다음 단계**: Stage 5 광역 게이트·최종 보고 승인 대기

## 1. 작업 기준 동기화

Stage 4 시작 전 `upstream/devel@4f9aaaff`를 병합했다.

```text
e8ffd898 Merge remote-tracking branch 'upstream/devel' into issue-2214-page-local-repaint
```

`#2183`/`#2233` 오더 문서 행과 `#2214` 진행 기록을 모두 보존했고, 병합 후 Stage 3 Rust 게이트를 다시 통과한 뒤 Studio 구현을 시작했다.

## 2. 구현 내용

### 2.1 typed deferred mutation result

`WasmBridge.insertTextInCellDeferredPagination()`은 이제 문자열 JSON이 아니라 다음 결과를 반환한다.

```text
ok, charOffset, paginationDeferred, cellFlowChanged
```

- 실제 deferred API: `paginationDeferred=true`
- API가 없는 immediate fallback: `paginationDeferred=false`
- Stage 3 이전 WASM처럼 mutation 결과에 `cellFlowChanged`가 없음: mutation 후 예외를 던지지 않고 보수적 `true`로 취급해 1회 flush로 복구
- `ok`/`charOffset`이 잘못된 결과: 기존처럼 예외

구형 WASM이 섞인 캐시 상태에서 텍스트만 변경되고 history/cursor가 누락되는 mutation-after-throw 상태를 피했다.

### 2.2 실행 단위 mutation effect

`TextMutationEffects`는 세 신호를 전달한다.

| 신호 | 의미 |
|------|------|
| `deferredPagination` | 현 mutation이 pagination을 지연함 |
| `cellFlowChanged` | 후속 flow advance가 바뀌어 pre-cursor flush가 필요함 |
| `paginationCompleted` | immediate mutation이 이전 pending까지 포함해 pagination을 완료함 |

`InsertTextCommand`와 `DeleteTextCommand`는 물리 실행마다 effect를 새로 계산하고 한 번만 소비한다. `CommandHistory`는 merge 전에 현재 실행 effect를 회수한다.

- 이전 `true`가 다음 stable merge에 전파되지 않음
- redo는 과거 effect를 재사용하지 않고 실제 mutation 결과를 다시 계산
- undo·IME `recordWithoutExecute`·history clear는 남은 effect를 제거
- 삭제 undo와 구조 복원 삽입은 명시적 immediate API를 사용

### 2.3 pre-cursor 단일 소비

일반 입력과 redo의 순서를 다음으로 고정했다.

```text
WASM mutation
→ history effect capture
→ immediate pagination이면 기존 pending 제거
→ deferred mutation이면 pending 등록
→ cellFlowChanged=true이면 full flush 1회
→ cursor.moveTo / exact cursor lookup
→ page-local 또는 full 표시 무효화
```

boundary flush 후에는 `afterEdit(false)`로 full 표시만 갱신해 동일 mutation을 다시 flush하지 않는다. 후속 stable raw 입력이 새 pending을 만들면 30쪽 이하 문서에서만 기존 10초 idle 정책을 복원한다.

page-local redraw는 `deferredPaginationPending=true`인 stable deferred mutation에서만 허용한다. 삭제, depth 2 중첩 셀, 구형 API fallback처럼 immediate pagination을 끝낸 mutation은 flow/cut 신호가 없으므로 후속 페이지 Canvas가 stale하지 않도록 추가 pagination 없이 full 표시 무효화로 보낸다.

### 2.4 IME/iOS raw 입력과 lifecycle

depth 1 raw 셀 입력도 command가 쓰는 `insertTextWithMutationEffects()`를 공유한다. depth 2 이상은 기존 path immediate API를 유지한다.

- raw delete immediate effect와 insert deferred effect를 OR 누적
- 누적 effect를 `cursor.moveTo()` 전에 한 번만 소비
- iOS 100ms debounce 묶음은 full-refresh 필요를 OR 누적하고 callback에서 재-flush하지 않음
- `compositionend`에서 조합 오버레이를 숨긴 뒤 일반 DOM caret를 exact rect에서 다시 표시
- 문서 전환 시 pending/timer, raw accumulator, composition anchor/ghost suppression, iOS anchor/value를 초기화

반복 E2E 중 직전 문서의 `_lastComposedText`와 hidden input 값이 다음 문서 첫 입력을 무시하거나 두 글자로 합치는 lifecycle 누수를 발견했고 같은 초기화 계약에 포함했다.

## 3. 영구 GREEN 검증 계약

### 3.1 focused 기본 runner

`issue-2214-page-local-repaint.test.mjs`의 기본 실행은 `--diagnose` 없이 HWP/HWPX 각 3회를 실행한다. 각 실행의 필수 단언은 다음과 같다.

| 시점 | 단언 |
|------|------|
| 1~43번째 | `cellFlowChanged=false`, WASM flush 0회, 4줄, bounds 945.9, exact text/tree/cursor |
| 44번째 | `cellFlowChanged=true`, mutation → flush → cursor 순서, 누적 flush 1회 |
| 44번째 geometry | 5줄, bounds 971.5, `cellOverflowed=false`, page 0 exact cursor, 115쪽 |
| 45~50번째 | 추가 flush 0회, 최종 180자, 115쪽 |
| 지연 안정성 | 2 rAF·100ms·850ms·1.6초의 text/tree/layout/cursor/DOM caret 불변 |
| 합성 화면 | 43→44 crop은 10,074 pixel 변화, 44 후 네 시점 crop은 exact SHA 동일 |

trace wrapper는 deferred result, pending preparation, 실제 `wasm.flushDeferredPagination`, path-near cursor query를 기록한다. CI hard gate는 시간이 아니라 횟수·순서·exact state·pixel 안정성이다.

### 3.2 raw stable/boundary smoke

각 형식의 IME/iOS 경로를 별도 reload로 실행했다.

| 형식 | 경로 | stable 1자 | 44번째 경계 |
|------|------|-------------|----------------|
| HWP | IME | flush 0 | flush 1 |
| HWP | iOS debounce | flush 0 | flush 1 |
| HWPX | IME | flush 0 | flush 1 |
| HWPX | iOS debounce | flush 0 | flush 1 |

stable은 4줄·bounds 945.9·pending true, boundary는 5줄·bounds 971.5·pending false였다. 모두 150ms callback 후에도 추가 flush가 없었고 DOM caret를 다시 표시했다.

desktop Chrome에서 iOS 분기를 강제한 로직 검증이므로 실제 iOS contentEditable·가상 키보드·포커스는 기기 검증 범위로 남는다.

### 3.3 optional diagnostic 보존

기존 timeline·PNG·full-layer·explicit-flush 대조는 `--diagnose`로 보존했다. 새 계약은 boundary 자동 flush 1회와 후속 explicit flush no-op이다.

HWP/HWPX 각 1회 최종 진단에서 둘 다 `GREEN / existing-green`이었다. automatic timeline, full-layer control, explicit control의 crop 비교는 변경 pixel 0, max channel delta 0이었다.

## 4. 정확성 결과

### 4.1 geometry와 cut

focused browser는 bounds 945.9→971.5와 115쪽을 검증했다. crate-internal structured GREEN은 같은 최종 production에서 page 0 `PartialTable` cut 37→38을 HWP/HWPX 모두 재확인했다.

```text
transient: end_cut=[37], bounds=945.9
full flush: end_cut=[38], bounds=971.5
PartialTable fragments=115, changed_after_flush_count=115
```

### 4.2 trace 순서

양 형식 6회 모두 다음 순서였다.

```text
deferred insert 완료
< wasm.flushDeferredPagination 완료
< 첫 getCursorRectByPathNear 완료
```

50자 trace의 deferred result는 `[false × 43, true, false × 6]`, 최종 flush 횟수는 1이었다. 115쪽 exact-miss full fallback scan은 발생하지 않았다.

## 5. 성능 결과

Chrome 1280×900, DPR 1, zoom 100%, 로컬 Vite 7714 환경의 관찰값이다.

| 형식 | run | keyboard stable p95 | operation stable p95 | cursor query p95 | boundary handler | boundary flush |
|------|----:|--------------------:|---------------------:|-----------------:|-----------------:|---------------:|
| HWP | 1 | 47.6ms | 27.9ms | 27.4ms | 968.7ms | 923.5ms |
| HWP | 2 | 47.9ms | 28.5ms | 28.0ms | 927.4ms | 881.8ms |
| HWP | 3 | 47.0ms | 28.2ms | 27.7ms | 926.1ms | 881.3ms |
| HWPX | 1 | 50.6ms | 28.6ms | 28.1ms | 941.6ms | 896.5ms |
| HWPX | 2 | 47.8ms | 28.4ms | 27.5ms | 952.6ms | 906.9ms |
| HWPX | 3 | 47.9ms | 29.0ms | 28.4ms | 936.3ms | 890.5ms |

stable operation/cursor는 100ms 관찰 기준을 충족했고 boundary는 1.5초 기준을 충족했다. Stage 3의 약 27ms warm query fast path가 Studio 일반 입력에서도 유지됐다.

boundary 약 0.9초는 정확성 우선 full pagination의 의도된 잔여 비용이다. 매 입력이 아니라 실제 flow advance 경계에서만 발생한다.

## 6. 검증 결과

| 검증 | 결과 |
|------|------|
| `docker-compose --env-file .env.docker run --rm wasm` | 통과, 최신 Stage 3 Rust WASM 재빌드 |
| `cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture` | 1 passed |
| `cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture` | 1 passed, HWP/HWPX 115쪽·저장 재로드 유지 |
| `cargo test --profile release-test --test issue_2214_page_local_repaint -- --nocapture` | 2 passed |
| `cargo test --profile release-test --test issue_2222_layer_json_cache -- --nocapture` | 1 passed |
| `cargo test --profile release-test --lib issue2214 -- --nocapture` | 9 passed, cut 37→38·115 fragment 정합 |
| `npm test` | 214 passed |
| `npm run build` | TypeScript + Vite production build 통과 |
| `npm run e2e:renderer-contract` | 통과 |
| focused HWP/HWPX `--runs=3` | 6/6 GREEN, raw stable/boundary 8/8 GREEN |
| diagnostic HWP/HWPX `--runs=1` | 2/2 GREEN, PNG oracle diff 0 |
| `npx tsc --noEmit` | 통과 |
| `node --check e2e/issue-2214-page-local-repaint.test.mjs` | 통과 |
| `git diff --check` | 통과 |

중점 TS 동작 테스트는 effect one-shot, raw OR, immediate delete completion, history merge 비누수, redo 재계산, depth 1 deferred/depth 2 immediate를 검증한다. source contract와 실제 Chrome raw smoke를 같이 사용해 InputHandler 순서를 보완했다.

## 7. 변경 파일

| 파일 | 역할 |
|------|------|
| `rhwp-studio/src/core/wasm-bridge.ts` | typed result, immediate fallback, 구형 결과 보수 복구 |
| `rhwp-studio/src/engine/command.ts` | deferred/flow/immediate effect와 raw 공유 insert helper |
| `rhwp-studio/src/engine/history.ts` | 실행 단위 effect capture·one-shot 소비 |
| `rhwp-studio/src/engine/input-handler.ts` | pre-cursor pending/flush, full/page-local 라우팅, lifecycle 초기화 |
| `rhwp-studio/src/engine/input-handler-text.ts` | IME/iOS raw effect, debounce OR, composition caret 복귀 |
| `rhwp-studio/tests/cell-flow-boundary.test.ts` | command/history/effect 동작 GREEN |
| `rhwp-studio/tests/input-edit-invalidation.test.ts` | raw 순서·compatibility·lifecycle source contract |
| `rhwp-studio/e2e/issue-2214-page-local-repaint.test.mjs` | focused 3회·raw 0→1·Canvas crop 영구 GREEN, optional diagnostic |
| `mydocs/orders/20260713.md` | Stage 4 완료·승인 대기 상태 |

## 8. 남은 범위와 후속

### 8.1 성능·paginator

- boundary full flush의 약 0.9초 비용은 `#2193` 종합 성능 범위와 후속 bounded/partial paginator 설계에서 다룬다.
- immediate cell mutation은 flow 신호가 없어 correctness 우선 full 표시 무효화를 사용한다. 이 표시 비용을 다시 page-local로 줄이려면 delete/path API에도 flow/cut dirty scope 계약이 필요하다.

### 8.2 오류·플랫폼

- boundary flush 예외 시 pending은 보존되지만 현 cursor lookup은 새 geometry를 보장하지 못한다. 30쪽 초과 문서의 retry/error UX는 별도 오류 복구 계약 후보다.
- iOS E2E는 desktop Chrome의 로직 분기 검증이다. 실제 iOS 기기의 contentEditable·IME·포커스는 추가 기기 검증 범위다.
- 하나의 조합 중 여러 raw update가 같은 wrap 경계를 반복해 넘으면 exact cursor 정확성을 위해 update별 경계 flush가 정당할 수 있다. composition 단위 추가 축약은 paginator 설계와 함께 다룬다.

Stage 4에서 paginator, font metric, line-break semantic, parser/serializer, Canvas production renderer는 변경하지 않았다. `#2215` 드래그 selection과 기타 `#2193` 후속은 별도 범위다.

## 9. 독립 리뷰

최종 dirty diff를 production, E2E, 계획 대조로 분리해 독립 리뷰했다.

- production: P0/P1/P2 없음
- E2E: P0/P1 없음. 기본 3회, raw stable/boundary, crop exact hash로 보완
- 계획 대조: 코드 P0/P1 없음. Docker/Rust/Studio/renderer/cut/PNG 증거 충족

Stage 5 승인 전에 추가 production 수정, push, PR, 이슈 close를 수행하지 않는다.
