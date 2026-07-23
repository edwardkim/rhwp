# 구현계획서 — Task M100 #2424

## 1. 구현 원칙

거대 셀 fixture에서는 affected continuation chain 자체가 115 fragments로 길고 full flush 뒤 page 2~114의
113 cuts가 바뀐다. 앞부분 재사용만으로는 main-thread long task를 충분히 줄일 수 없으므로, 하나의
resume state를 Native exact-cut 검증과 browser chunk scheduler가 공유한다.

범용 `TypesetEngine` 전체를 coroutine으로 바꾸지 않는다. target table continuation을 shadow state에서
stepwise 계산할 수 있는 좁은 fast path를 먼저 구현하고, 지원하지 않는 구조는 기존 full pagination을
사용한다.

## 2. 상태 모델

### 2.1 `DeferredPaginationDescriptor`

- document edit revision
- section / parent paragraph / table control / cell / cell paragraph
- 최초 affected global page와 fragment
- table row/column/span/cell structure fingerprint
- section/page/column definition fingerprint
- old fragment chain fingerprint

descriptor는 최신 deferred target 하나만 소유한다. 새 mutation은 revision을 올리고 이전 pending job을
supersede한다.

### 2.2 `TableContinuationCursor`

- cursor row
- start cut과 block-cut 여부
- continuation 여부
- caption/header/host-spacing 적용 상태
- emitted fragment와 step count

각 step은 cut이 단조 전진하는지 확인한다. row/cut이 전진하지 않거나 terminal state가 모순되면 job을
폐기하고 full fallback한다.

### 2.3 `BlockTableContinuationContext`

- target table의 immutable measured source를 재조회할 descriptor 좌표
- cut row heights, rowspan touched state, repeat header와 caption 입력
- current page/column, used height, footnote/zone/flow state
- prefix 이후 새 pages/items를 축적하는 shadow `TypesetState`

context는 `Paragraph`, `Table`, `MeasuredTable`, styles를 장기 borrow하지 않는다. 각 step 시작 시 current
revision/structure를 검증하고 descriptor 좌표로 immutable source를 재조회한다.

### 2.4 `PendingPaginationJob`

- descriptor와 expected revision
- continuation context
- 기존 pagination의 validated prefix
- shadow result
- status: pending / complete / stale / unsupported / failed

완료 결과는 revision, fragment continuity, page count/global offset과 downstream flow fingerprint가 모두
유효할 때만 기존 `PaginationResult`를 교체한다.

## 3. 구현 순서와 테스트

### B1 — descriptor 수명

1. `DocumentCore`에 단조 revision과 descriptor를 추가한다.
2. deferred cell edit 성공 시 target과 구조 fingerprint를 기록한다.
3. 동기 full pagination 완료 시 descriptor를 소비한다.
4. latest/superseded/target-missing/structure-changed 상태 테스트를 추가한다.

검증:

- descriptor가 latest edit까지 유지되는지
- stable edit가 이미 감지한 flow boundary를 지우지 않는지
- target 변경과 full flush의 수명 계약
- 기존 #2214와 text-editing focused tests

완료 결과(2026-07-22):

- `DeferredPaginationDescriptor`와 단조 revision을 `DocumentCore`에 추가했다.
- row/column/span/cell paragraph/control 구조 fingerprint와 target 첫 global page를 기록한다.
- `Current`/`Superseded`/`TargetMissing`/`StructureChanged` stale guard를 추가했다.
- full pagination 성공 시 최신 descriptor를 소비한다.
- focused 상태 전이, #2214 library 9건, page-local repaint 3건, wasm32 library check를 통과했다.

### C1 — cursor/context 리팩터링

1. 최신 continuation loop의 지역 cursor를 값 타입으로 이동한다.
2. iteration 결과를 skipped/emitted/complete로 분리한다.
3. fragment budget 1/8/무제한 driver가 one-shot과 동일한 결과를 만드는지 확인한다.
4. 최신 #2439/#2279/#2699 table-flow 보정을 context 입력과 step 내부에 보존한다.

검증:

- HWP/HWPX 115 fragments와 113 changed cuts
- 모든 adjacent `end_cut == next.start_cut`
- `is_continuation`, `is_block_split`, row range exact 일치
- renderer typeset focused tests와 WASM library check

완료 결과(2026-07-22):

- row/start-cut/block-cut/continuation을 `TableContinuationCursor`로 묶었다.
- split 준비값과 shadow `TypesetState`를 `BlockTableContinuationContext`가 소유한다.
- iteration 결과를 `Skipped`/`Emitted`/`Complete`로 나누고 caller-controlled `step`을 추가했다.
- budget 1/8/무제한에서 각각 115/15/1 step으로 같은 115-fragment HWP/HWPX oracle을 재현했다.
- 최신 #2439/#2279 table-flow 보정과 footnote refit 경로를 그대로 보존했다.

### D1 — DocumentCore pending job

1. selective measurement 후 descriptor target의 measured table을 재조회한다.
2. validated prefix와 shadow state로 pending job을 시작한다.
3. `step(fragment_budget)`이 공개 pagination을 수정하지 않고 progress만 반환한다.
4. complete commit과 stale/unsupported/error full fallback을 구현한다.

### D2 — WASM/Studio scheduler

1. 내부 WASM begin/step/cancel/drain API를 추가한다.
2. `InputHandler`가 flow boundary에서 동기 full flush 대신 job을 시작한다.
3. 한 task당 fragment/time budget 후 macrotask 또는 rAF로 yield한다.
4. 새 edit는 기존 timer/job을 취소하고 최신 descriptor로 교체한다.
5. save/save-as/print와 명시적 full query는 drain/fallback 후 실행한다.

검증:

- 진행 중 `deferredPaginationPending`과 공개 page count 계약
- job cancel/restart
- save/print completion/fallback
- Studio unit test와 HWP/HWPX browser E2E

완료 결과(2026-07-23):

- `DocumentCore`가 descriptor, measured section, borrow-free renderer context를 pending job으로 소유한다.
- begin/step/cancel/동기 drain과 `none`/`pending`/`complete`/`fallback`/`stale` 상태를 WASM에 노출했다.
- 진행 중 공개 pagination은 유지하고 마지막 fragment에서만 measurement/dirty/cache와 함께 원자 commit한다.
- 새 edit stale 폐기와 최신 revision 재시작, save/save-as/print 동기 barrier를 고정했다.
- Studio는 budget 1을 macrotask별로 실행하고 새 입력에서 예약 task와 core job을 교체한다.
- HWP/HWPX 실제 API 115 steps, 115 fragments, 113 changed cuts와 공개 pagination 비노출 계약을 통과했다.
- release 계측은 begin 약 32ms, step p50 약 10.2ms, p95 약 11.0ms, max 약 22.3ms였다.
- Studio 507 tests와 production build, wasm-pack web binding 생성, wasm32 check가 통과했다.

## 4. 성능 게이트

- Stage A와 같은 환경·fixture·입력 수·반복 수 사용
- full pagination total과 table continuation 시간을 before/after 비교
- 한 step 최대 blocking time과 전체 job completion time 기록
- input-to-2-rAF p50/p95와 paint 사이 yield 여부 기록
- 목표 slice는 16ms 근처 또는 이하이며 correctness를 우선한다.

총 completion time이 소폭 증가해도 browser event loop가 fragment 사이에 실행될 수 있어야 한다. 단순히
기존 full flush를 idle로 미루는 결과는 완료로 인정하지 않는다.

완료 결과(2026-07-23):

- release step p95는 HWP 10.733ms, HWPX 11.002ms였다.
- 실제 브라우저 경계 입력은 HWP/HWPX 6회 모두 동기 flush 0회, begin 1회, step 115회였다.
- 경계 operation은 75.9~81.3ms에 반환했고 공개 pagination은 마지막 step 전까지 유지됐다.
- 완료 뒤 model/tree/layout/cursor/caret, 115쪽, 113 changed cuts와 page 0 PNG가 exact했다.
- 전체 Rust/Studio 회귀, wasm32와 production WASM/Studio build가 통과했다.
