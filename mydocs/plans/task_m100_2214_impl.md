# 구현 계획서 — Task M100 #2214

> 2026-07-13 보정: 최신 `upstream/devel@c7864c62` cold/warm matrix와
> cache-only 통제로 직접 원인을 warm `cell_units_cache` 무효화 누락으로 좁혔다.
> production 계약은 편집 셀·소유 표의 scoped cache coherence와 실제 cell-flow 경계의
> pre-cursor 1회 flush를 분리한다.

## 1. 작업 기준

- 이슈: [#2214](https://github.com/edwardkim/rhwp/issues/2214)
- 수행계획서: `mydocs/plans/task_m100_2214.md`
- 작업 브랜치: `issue-2214-page-local-repaint`
- 작업 worktree: `/private/tmp/rhwp-task2214`
- 기준 브랜치: `upstream/devel@48c33455`
- 작성일: 2026-07-11
- 최신 교차검증: `upstream/devel@c7864c62`, `/private/tmp/rhwp-task2214-latest`
- 보정일: 2026-07-13
- 직접 대상: Canvas2D 기반 Studio page-local text-edit 표시 정확성
- 재현 픽스처:
  - `samples/issue1949_giant_cell_nested_tables_perf.hwp`
  - `samples/issue1949_giant_cell_nested_tables_perf.hwpx`

## 2. 구현 원칙

1. 구현 순서는 **결정적 계측 → 모델 GREEN/화면 RED 계약 → 확정된 단일 경계 수정 →
   브라우저·성능 회귀 → 광역 게이트**로 고정한다.
2. HWP는 Stage 1에서 현재 증상을 결정적으로 재현해야 한다. HWPX는 같은 절차로 현재
   RED 또는 기존 GREEN을 분류하고, 재현되지 않는 형식을 억지로 RED로 만들지 않는다.
   수정 후에는 두 형식 모두 동일한 최종 GREEN 계약을 만족해야 한다.
3. 데이터 보존은 모델·native 저장 재로드를, 표시 정확성은 page text layout·layer tree·
   브라우저 합성 화면을 권위로 삼는다.
4. Enter는 문단 분할과 pagination을 함께 수행하므로 자동 테스트의 정답지로 사용하지 않는다.
   입력 모델을 바꾸지 않는 explicit deferred pagination flush와 full refresh를 대조군으로 둔다.
5. production 수정은 Stage 1의 최초 stale 경계와 Stage 2의 RED assertion이 같은 원인을
   가리킨 뒤에만 수행한다. 여러 후보를 한 번에 수정하지 않는다.
6. 매 입력마다 115쪽 전체 pagination, 전역 layout-cache clear, 800ms 검증 timer 제거,
   Enter 또는 무조건 full refresh 주입은 허용하지 않는다.
7. 일반 입력은 scoped cache eviction 후 flush 0회를 유지한다. target cell의 상대 flow
   advance가 바뀌는 실제 경계에서만 cursor 조회 전에 full flush를 정확히 1회 허용한다.
8. line count 자체를 경계 신호로 쓰지 않는다. `last.vpos + line_height + line_spacing -
   first.vpos`처럼 후속 flow에 영향을 주는 상대 advance의 pre/post 변화를 원자적으로 반환한다.

## 3. 현재 계약과 구현 시 주의점

### 3.1 page-local 판정과 deferred mutation은 서로 다른 계약이다

현재 짧은 셀 삽입은 refresh 판정보다 먼저
`insertTextInCellDeferredPagination()`을 실행한다. 반면
`deferredPaginationPending=true`는 `afterPageLocalEdit()`에서만 등록된다.

따라서 line-count 변화만 page-local 거부 조건에 추가한 뒤 기존 `afterEdit()`로 보내는
방식은 안전하지 않다. 현재 mutation이 아직 pending으로 등록되지 않아 full 경로에서도
pagination이 flush되지 않을 수 있다. before/after page 변화, 강제 full refresh, redo,
delete undo에서도 같은 계약 누락이 발생할 수 있다.

Stage 3에서 deferred mutation을 full 경로로 재분류해야 하는 결과가 나오면 다음을 먼저
만족해야 한다.

- mutation이 실제로 pagination을 지연했는지 명시적인 effect로 전달
- refresh 판정보다 먼저 deferred pending 등록
- flush 성공 뒤 cursor rect 재계산
- 즉시 pagination된 mutation에는 거짓 pending을 만들지 않음
- flush 실패 시 pending을 보존

보정 계획에서는 deferred mutation 결과가 `cellFlowChanged`를 반환하고, Studio가 history
execute 직후·cursor 이동 전에 이를 소비한다. `false`이면 page-local 경로를 유지하고,
`true`이면 pending을 먼저 등록한 뒤 full flush를 정확히 1회 수행하고 cursor rect를 새
pagination에서 조회한다. IME/iOS raw insert 경로도 같은 결과 계약을 사용해야 한다.

### 3.2 기존 E2E helper는 실제 앱 로드 경로를 완전히 재현하지 않는다

기존 `loadHwpFile()`은 `wasm.loadDocument()`와 `canvasView.loadDocument()`만 호출해
문서별 폰트 로딩, 설정 적용, input handler 활성화를 우회한다. #2214는 폰트·입력·Canvas
시간축이 핵심이므로 `open-document-bytes`와 `open-document-bytes:done` 이벤트를 사용한
실제 앱 로드 helper를 둔다.

기존 `captureCanvasScreenshot()`은 main flow canvas 하나만 캡처하므로 `flow-static`,
background/behind/front canvas와 DOM caret을 누락한다. 브라우저가 합성한 페이지 영역을
clip 캡처하고, 구조적 text/run 단언을 pixel 비교보다 우선한다.

### 3.3 800ms 검증은 조건부 경로다

static verification은 static flow 또는 behind/front layer를 실제 재사용한 경우에만
예약된다. 두 기준 SHA의 HWP/HWPX 재현에서 예약 0회였고 full-layer render도 복구하지
못했으므로 production 후보에서 제외한다.

### 3.4 warm layout cache와 pagination은 서로 다른 계약이다

`invalidate_page_tree_cache_from(0)`은 page tree와 layer JSON만 비우며,
`invalidate_page_tree_cache()`만 `clear_layout_caches()`를 호출한다. 최신 통제 결과는 다음과
같다.

| 상태 | tree max | cursor | cut | bounds h |
|------|---------:|--------|----:|---------:|
| cold 44자 | 174 | `(569.7,341.9)` | 37 | 945.9 |
| prewarm 44자 | 129 | fallback `(84.1,238.7)` | 37 | 945.9 |
| cache clear only | 174 | `(569.7,341.9)` | 37 | 945.9 |
| full flush | 174 | `(569.7,341.9)` | 38 | 971.5 |

따라서 old cut은 직접 원인의 충분조건이 아니며, cache clear만으로 visible tree/cursor는
복구된다. 그러나 `cellBounds`와 cut까지 완전한 geometry를 확정하려면 실제 flow 경계의
pagination이 필요하다. global clear는 #1949/#2063 캐시를 문서 전체에서 잃으므로 generic
partial invalidation에는 추가하지 않는다. unrelated cache 보존은 flush 0회인 일반 입력의
scoped 경로 계약이며, boundary full flush가 기존 전역 clear를 수행하는 것은 허용한다.

## 4. 작업환경 격리

다른 진행 중 작업과 충돌하지 않도록 모든 산출물과 서버를 #2214 worktree 안에 격리한다.

1. Stage 1 commit `f0cb99f0`은 보존한다. 구현 승인 뒤 `issue-2214-page-local-repaint`를
   당시 최신 `upstream/devel` 위로 갱신하고, detached latest 진단 worktree의 dirty probe를
   production branch에 그대로 복사하지 않는다.
2. `rhwp-studio/node_modules`는 worktree에서 `npm ci`로 별도 준비한다.
3. `.env.docker`가 없을 때만 `.env.docker.example`을 복사하고, 해당 worktree에서 Docker
   WASM을 빌드해 `pkg/`를 생성한다.
4. Vite는 `127.0.0.1:7714 --strictPort`로 실행하고 E2E는
   `VITE_URL=http://127.0.0.1:7714`를 사용한다.
5. headless Chrome은 고정 viewport 1280×900, DPR 1, zoom 100%로 실행하고
   `document.fonts.ready`와 2 rAF를 기다린다.
6. `node_modules`, `.env.docker`, `pkg/`, `output/poc/task2214/`, E2E screenshot은 Git에
   포함하지 않는다.

## 5. 예상 변경 파일

### 공통·테스트 파일

| 파일 | 역할 |
|------|------|
| `rhwp-studio/e2e/issue-2214-page-local-repaint.test.mjs` | 시간축 진단, RED→GREEN, HWP/HWPX 합성 화면 권위 E2E |
| `rhwp-studio/e2e/helpers.mjs` | 필요할 때만 실제 앱 샘플 로드·2 rAF·합성 crop helper 추가 |
| `tests/issue_2214_page_local_repaint.rs` | 모델·`LINE_SEG`·flush·원본 형식 저장 재로드 GREEN 핀 |
| 기존 TS test 한 곳 | Stage 1에서 확정된 경계의 최소 계약 테스트 |
| `mydocs/working/task_m100_2214_stage{N}.md` | 각 Stage 증거와 검증 결과 |
| `mydocs/report/task_m100_2214_report.md` | 최종 원인, 수정, 정확성·성능 결과 |
| `src/renderer/layout.rs` | 편집 cell/table scoped cache eviction API |
| `src/document_core/commands/text_editing.rs`, `src/wasm_api.rs` | flow advance 비교와 typed mutation result |
| `rhwp-studio/src/core/wasm-bridge.ts`, `rhwp-studio/src/engine/command.ts` | `cellFlowChanged` 전달·보존 |
| `rhwp-studio/src/engine/input-handler.ts`, `input-handler-text.ts` | 일반·raw 입력의 pre-cursor boundary flush 단일 소비 |

### Stage 1·교차검증 결과별 production 계약

기존 Canvas 후보는 기각됐다. 아래 두 행은 대안이 아니라 서로 다른 일관성 층이다.

| 확정 결과 | 허용 후보 파일 | 최소 방향 |
|-----------|----------------|-----------|
| warm cell units가 stale하고 cache-only로 visible tree/cursor 복구 | `layout.rs`, `text_editing.rs` | cached owner flag가 불변이면 edited cell만 evict; cached false→true일 때만 owner cells evict+flag update; unrelated cache 보존 |
| 상대 cell-flow advance 변화에서 bounds/cut 갱신 필요 | `text_editing.rs`, `wasm_api.rs`, `wasm-bridge.ts`, `command.ts`, `input-handler.ts` | `cellFlowChanged`를 원자적으로 반환하고 cursor 조회 전 1회 flush |
| 모델 text 또는 `LINE_SEG`부터 손실 | 현재 표시 이슈 범위 밖 | Studio 수정 중단, document-core 편집/reflow 이슈로 재분리 |

global `clear_layout_caches()`를 `invalidate_page_tree_cache_from()`에 직접 추가하는 한 줄
수정은 진단 oracle로만 허용한다. production에서 generic partial invalidation의 의미를 문서
전체 cache clear로 바꾸지 않는다.

## 6. 구현 단계

### Stage 1. 실제 앱 경로의 결정적 재현과 최초 stale 경계 계측

#### 목표

production 코드를 수정하지 않고 HWP 증상을 자동 재현하고 HWPX의 현재 상태를 분류한다.
모델, page fragment/layer tree, cursor, Canvas 중 최초 불일치 경계를 하나로 좁힌다.

#### 작업

1. `rhwp-studio/e2e/issue-2214-page-local-repaint.test.mjs`에 `--diagnose` 모드를 만든다.
2. 샘플을 `/samples/<filename>`에서 fetch한 뒤 다음 이벤트로 실제 앱 로드 경로를 탄다.
   - `open-document-bytes` emit: `requestId`, `skipUnsavedGuard: true`
   - 대응하는 `open-document-bytes:done` 대기
   - source format, 115쪽, font ready, Canvas, input handler 활성 확인
3. 캐럿을 다음 full cell path에 직접 배치한다.

```text
sectionIndex=0, paragraphIndex=5, charOffset=130
parentParaIndex=0, controlIndex=2, cellIndex=2, cellParaIndex=5
cellPath=[{ controlIndex=2, cellIndex=2, cellParaIndex=5 }]
```

4. `cursor.clearSelection()` → `cursor.moveTo()` → `resetPreferredX()` → `updateCaret()` →
   `focus()` 순서로 입력 상태를 고정한다.
5. 첫 로드에서 `1`을 한 글자씩 입력해 4줄→5줄 전환 문자 수 `N`을 최대 128자 안에서
   찾는다. `N`을 하드코딩하지 않는다.
6. 문서를 다시 로드하고 `N-1`자까지 입력한 뒤 전환 직전 상태를 수집한다.
7. `N`번째 문자는 hidden textarea의 동기 `InputEvent('input')`으로 dispatch해 첫 rAF
   이전 상태를 기록한다. 별도로 `page.keyboard.type('1')` 사용자 경로 smoke도 남긴다.
8. 다음 체크포인트를 수집한다.
   - 동기 input 직후
   - 2 rAF 뒤
   - 100ms 뒤
   - 850ms 뒤
   - 1.6초 뒤
   - 추가 1자와 2자 입력 후 2 rAF
   - explicit flush와 full refresh 뒤
9. E2E 내부 wrapper로 다음 호출과 이벤트를 계측하고 원본 함수로 위임한다.
   - `flushDeferredPagination`
   - `renderPageToCanvasFiltered`
   - `PageRenderer.renderPage`
   - `document-page-invalidated`
   - `document-changed`
10. 각 체크포인트에 다음을 기록한다.
    - exact cell text와 length, cursor position/rect/`cellOverflowed`
    - `getLineInfoInCell()`로 수집한 line start/end/count
    - deferred pending과 page count
    - `getPageTextLayout()`의 대상 cell paragraph run
    - PageLayerTree의 상관 TextRun과 bbox
    - invalidated page, render context/result, filtered layer kind
    - static verification 예약 여부
    - DOM caret 위치와 브라우저 합성 crop
11. 통제 대조를 분리한다.
    - pagination 없이 full-layer page render
    - explicit flush + full refresh
    - verification이 예약된 경우 800ms 직전·직후
12. JSON/PNG는 `output/poc/task2214/stage1/`에 저장하고 커밋하지 않는다.

#### 원인 판정

| 관찰 | 판정 |
|------|------|
| 모델/LINE_SEG는 최신이고 fresh full-layer page render만으로 복구 | static reuse 또는 page-local render context 경계 |
| layer tree는 최신이나 invalidated page/Canvas/caret만 불일치 | page target 또는 합성 경계 |
| 첫 rAF 화면은 최신이나 850ms 뒤 후퇴 | delayed verification/render generation 경계 |
| Stage 1 브라우저 통제에서는 explicit pagination만 복구 | 당시 중단·재계획; 최신 cache-only 통제가 zero-pagination visible 복구를 추가 입증 |
| 모델 text·저장 데이터부터 손실 | 표시 이슈가 아님 — 중단·재분리 |

#### 검증

```bash
cd /private/tmp/rhwp-task2214/rhwp-studio
npm ci
```

```bash
cd /private/tmp/rhwp-task2214
docker compose --env-file .env.docker run --rm wasm
```

```bash
cd /private/tmp/rhwp-task2214/rhwp-studio
npm test
npx vite --host 127.0.0.1 --port 7714 --strictPort
```

```bash
cd /private/tmp/rhwp-task2214/rhwp-studio
VITE_URL=http://127.0.0.1:7714 \
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
CHROME_EXTRA_ARGS="--force-device-scale-factor=1" \
node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless --diagnose
```

#### 완료·중단 조건

- HWP에서 동일 체크포인트의 증상이 반복 재현되고, HWPX는 RED 또는 기존 GREEN으로
  결정적으로 분류돼야 한다.
- 최초 불일치 경계가 판정표 한 행으로 좁혀져야 한다.
- 후보 둘 이상이 독립 결함이거나 production 진단 API가 필요하면 Stage를 완료하지 않고
  범위 승인을 요청한다.
- `mydocs/working/task_m100_2214_stage1.md`와 진단 E2E를 같은 커밋에 포함하고 승인받기 전
  Stage 2로 가지 않는다.

커밋 메시지:

```text
Task #2214: Stage 1 - 연속 입력 표시 불일치 경계 계측
```

Stage 1의 “explicit pagination only”는 당시 브라우저에 cache-only 통제가 없어서 내린
관측 범위의 결론이다. 2026-07-13 최신 native 교차검증이 이를 보정했으며, Stage 2 이후에는
warm cache coherence와 cell-flow geometry를 분리한 계약을 따른다.

### Stage 2. warm cache RED와 cell-flow 계약 고정

#### 목표

production을 수정하지 않고 cold/warm 원인, cache-only와 full-flush oracle, 실제 flow 경계
신호를 결정적 회귀 계약으로 고정한다.

#### 작업

1. 최신 진단 probe를 정리해 HWP/HWPX에서 다음 상태를 기록·단언한다.
   - cold 44자: tree max 174, exact cursor
   - prewarm 44자 및 30+20자: tree max 129, 첫 줄 fallback
   - cache-clear-only: max 174, exact cursor, cut 37, bounds 945.9
   - full flush: max 174, exact cursor, cut 38, bounds 971.5
   - direct/path-near 및 batch/sequential의 같은 판정
2. 28자 batch와 순차 1~50번째 입력의 target paragraph 상대 flow advance를 비교한다.
   - 28자: 변화 없음, 향후 `cellFlowChanged=false`
   - 1~43/45~50번째: 매 key 변화 없음, 44번째만 `+1920HU`
3. 115쪽 `PartialTable` structured cut chain과 exact cell-path UTF-16 render-tree range를 별도로
   수집해 각각 gap/overlap을 검사한다.
4. 실제 deferred call site와 global-clear 대조를 실행 가능한 crate-internal ignored RED로
   추가하고 Stage 3에서 같은 assertion을 GREEN으로 전환한다.
   - flag 불변 stable/44번째: target cell만 evict, sibling·owner flag identity 보존
   - local false→true지만 cached owner flag=true: target cell만 evict, 다른 owner entries 보존
   - empty nested host insert의 false→true: owner table 모든 cell units evict, owner flag=true 갱신
   - 두 branch 모두 unrelated/nested table cache identity 보존
   - global clear가 위 scope를 깨는 invariant/flag-changing synthetic 대조
5. 기존 #2185 한 글자 `[0,44,84,122]`, `vpos=17160`, 115쪽과 저장·재로드
   기준을 그대로 유지한다.
6. Studio E2E의 현재 HWP/HWPX warm RED, fallback 좌표, 약 2초 path-near 비용을 고정한다.

#### 검증

아래 intentional RED 개별 이름은 Stage 2 commit `7272115c`의 역사적 계약이며 Stage 3에서
non-ignored GREEN으로 승격하면서 제거됐다. 현재 브랜치에서 옛 `_red`/`--ignored` 필터를
재실행하지 않고 `mydocs/working/task_m100_2214_stage2.md`의 당시 결과를 증거로 사용한다.

```bash
cargo test --profile release-test --test issue_2214_cache_matrix_probe -- --ignored --nocapture
cargo test --profile release-test --test issue_2214_page_local_repaint issue_2214_cell_flow_transition_baseline -- --nocapture
cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture
```

```bash
cd rhwp-studio
node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless --diagnose --runs=3
```

#### 완료·중단 조건

- 두 형식의 cold/warm/cache-only/full-flush 네 상태가 결정적으로 분리돼야 한다.
- `cellFlowChanged`의 28자 false·44번째 true 근거가 line count가 아닌 상대 flow advance로
  설명돼야 한다.
- 전 페이지 range를 수집할 수 없거나 scoped invalidation의 owner/ancestor 범위를 정할 수
  없으면 Stage 3으로 가지 않는다.
- Stage 2 보고서와 test-only probe를 같은 커밋에 포함하고 승인받기 전 Stage 3으로 가지 않는다.

커밋 메시지:

```text
Task #2214: Stage 2 - warm cache 및 cell-flow 회귀 계약 고정
```

### Stage 3. Rust scoped cache coherence와 flow 경계 신호

#### 목표

모든 deferred 셀 편집의 layout-cache coherence를 국소적으로 회복하고, pagination이 필요한
실제 cell-flow 변화만 원자적 mutation result로 반환한다. 이 Stage에서는 Studio flush를
아직 연결하지 않는다.

#### 작업

1. `LayoutEngine`에 편집 cell과 소유 table을 받는 scoped invalidation API를 추가한다.
   - table-wide predicate 계산은 단일 helper에만 두고 test-only scan counter도 그 helper에 둔다.
     mutation 구현에 별도 paragraph/control predicate 전수 스캔을 추가하지 않는다.
     cached false→true branch의 direct owner-cell key 제거 순회는 predicate scan이 아니라
     필요한 eviction 범위이며 허용한다.
   - full-table rescan 대신 edited paragraph의 pre/post local contribution
     (`trim-visible text && nested table`)을 mutation과 함께 계산
   - cached owner flag=true 또는 local contribution 불변: edited cell entry만 제거, cached
     owner flag와 same-table sibling identity 보존
   - deferred insert가 cached false를 true로 바꾸는 경우에만 owner table의 모든 cell entries를
     제거하고 owner flag cache를 `true`로 갱신
   - false→true인데 owner flag cache가 없으면 cell-units cache도 없다는 invariant에 따라
     edited cell만 제거하고 local witness로 owner flag `true`를 기록
   - unrelated table cell/flag cache는 두 branch 모두 보존
2. deferred cell insert에서 mutation 전후 target paragraph의 상대 flow advance를 계산한다.

```text
last.vertical_pos + last.line_height + last.line_spacing - first.vertical_pos
```

3. 성공한 mutation을 다음 순서로 처리한다.
   - text mutation
   - target paragraph reflow
   - 후속 paragraph vpos 재계산
   - edited paragraph local contribution과 cached owner flag에 따른 scoped cache eviction/update
   - section/page-tree invalidation
4. deferred insert JSON에 `cellFlowChanged`를 추가한다. line starts만 바뀌고 flow advance가
   같으면 false, 줄 추가·삭제나 높이 변화로 advance가 바뀌면 true다.
5. HWP/HWPX native에서 warm tree/cursor를 GREEN으로 바꾸고, pagination은 0회·cut 37·
   bounds 945.9인 transient 상태를 명시적으로 확인한다.
6. flush 0회인 일반 입력에서 unrelated cache hit가 유지되는 단위 테스트로 generic partial
   invalidation의 global clear 회귀를 차단한다. same-table sibling과 owner flag는 flag 불변
   및 cached-true local-change fixture에서 identity/value를 보존하고, empty nested host가
   cached false를 true로 바꾸는 fixture에서는 owner table 전체 cell units가 재계산되고 flag가
   true로 갱신되는지 확인한다.
   실제 boundary full flush의 전역 clear는 허용한다.

#### 금지 변경

- `invalidate_page_tree_cache_from()` 안의 global `clear_layout_caches()` 호출
- 이 Stage에서의 full pagination 또는 Studio cursor 순서 변경
- line count만을 `cellFlowChanged`로 반환
- parser/serializer, font metric, Canvas timer 변경

#### 검증

```bash
cargo test --profile release-test --test issue_2214_page_local_repaint -- --nocapture
cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture
cargo test --profile release-test --lib issue2214 -- --nocapture
cargo test --profile release-test --lib test_insert_text_in_cell -- --nocapture
cargo test --profile release-test --lib issue2214_deferred_table_caption_reports_flow_change -- --nocapture
cargo test --profile release-test --lib issue2214_invalid_shape_cell_index_does_not_mutate_text -- --nocapture
cargo test --profile release-test --test issue_2063 -- --nocapture
cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture
```

`--lib layout_cache` 필터는 기존 이름과 일치하지 않아 `0 passed; 1 ignored` false-green이므로
Stage 3 검증에서 사용하지 않는다.

#### 완료·중단 조건

- warm direct/path-near가 최신 offset을 exact hit하고 115쪽 fallback을 타지 않아야 한다.
- 28자 `cellFlowChanged=false`, 44번째 `true`여야 한다.
- unrelated table cache는 항상 보존하고, same-table sibling은 flag 불변 시 identity를
  보존하며 owner flag cache도 유지해야 한다. flag 변화 시 owner cells는 반드시 재계산하고
  owner flag cache는 새 값으로 갱신해야 한다.
- stable key마다 owner table 전체를 다시 스캔하지 않아야 한다.
- table-wide predicate가 단일 helper 밖에서 중복 구현되지 않아야 한다.
- intentional RED는 같은 desired assertion의 non-ignored GREEN으로 승격한다.
- edited paragraph local contribution과 cached flag만으로 안전하게 판정할 수 없거나 owner cell
  집합을 안전하게 제거하려면,
  또는 ancestor generation/owner graph 설계가 필요하거나 전 페이지 range가 깨지면 중단하고
  별도 설계 승인을 요청한다.
- Stage 3 보고서와 production/test 변경을 같은 커밋에 포함하고 승인받기 전 Stage 4로 가지 않는다.

커밋 메시지:

```text
Task #2214: Stage 3 - 셀 layout cache coherence 및 flow 신호 정합
```

### Stage 4. pre-cursor 경계 1회 flush와 Studio/WASM 검증

#### 목표

`cellFlowChanged`를 Studio까지 전달해 실제 flow 경계에서 cursor 조회 전에 full pagination을
정확히 1회 수행한다. #2214 정확성과 #1918의 안정 입력 fast path를 함께 보존한다.

#### 작업

1. `WasmBridge.insertTextInCellDeferredPagination()`이 typed mutation result를 반환하게 한다.
2. `InsertTextCommand`가 결과의 `cellFlowChanged`를 보존하고, operation router가
   `history.execute()` 직후·`cursor.moveTo()` 전에 이를 읽는다.
   - effect는 실행마다 초기화하고 한 번만 소비한다.
   - history merge가 이전 `true`를 재사용하지 않으며 redo는 실제 mutation 결과를 다시 계산한다.
3. `cellFlowChanged=true`이면 deferred pending을 먼저 등록하고
   `flushDeferredPagination()`을 정확히 1회 실행한 뒤 cursor를 이동하고 full refresh한다.
4. IME/iOS raw insert 경로도 같은 typed result와 pre-cursor 경계를 사용한다.
   - 한 composition/지연 입력 묶음의 `cellFlowChanged`는 OR로 누적한다.
   - 최초 exact cursor 조회 전에 누적 effect를 한 번만 소비하고 flush한다.
   - IME와 iOS raw 경로별 단위 테스트에서 0→1회 전이와 이중 flush 부재를 고정한다.
5. boundary flush 뒤 page-local/overflow fallback이 다시 flush하지 않도록 단일 소비 계약을 둔다.
   기존 `cellOverflowed` guard는 별도 안전망으로 보존한다.
6. Docker WASM을 다시 빌드하고 HWP/HWPX E2E를 각각 세 번 실행한다.
7. 50자 입력 trace를 다음 구조로 단언한다.
   - 1~43자: flush 0회, cut 37, bounds 945.9, 최신 text/rect
   - 44번째 flow 경계: cursor 조회 전 flush 1회, cut 38, bounds 971.5
   - 45~50자: 추가 flow 경계 전 flush 0회, 전체 누적 flush 1회
8. 2 rAF, 100ms, 850ms, 1.6초의 text/tree/caret/crop과 115쪽을 확인한다.
9. key handler·cursor query·boundary flush 시간을 Stage 1과 같은 환경에서 기록한다.
   - 일반 입력 p95와 native tree/rect: 관찰 기준 100ms 이하
   - boundary handler: 현재 full flush 기준을 고려한 관찰 기준 1.5초 이하
   - CI hard gate는 시간 대신 flush 횟수, exact hit와 cache scope를 사용
10. #1949/#2185/#2222, renderer contract와 일반 edit pipeline을 실행한다.
11. Stage 1/2의 대형 진단 E2E에서 영구 assertion만 focused GREEN runner로 분리한다.
    - HWP/HWPX warm 입력, exact tree/cursor, 2 rAF·850ms·1.6초 안정성
    - 안정 입력 flush 0회, 44번째 경계 1회, 115쪽 유지
    - timeline/PNG 전수 dump는 optional `--diagnose` 모드에만 남긴다.

#### 검증

```bash
cd /private/tmp/rhwp-task2214
docker compose --env-file .env.docker run --rm wasm
cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture
cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture
cargo test --profile release-test --test issue_2214_page_local_repaint -- --nocapture
```

```bash
cd rhwp-studio
npm test
npm run build
npm run e2e:renderer-contract
VITE_URL=http://127.0.0.1:7714 \
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
CHROME_EXTRA_ARGS="--force-device-scale-factor=1" \
node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless
```

#### 완료·중단 조건

- boundary cursor 조회 전에 flush가 완료되고 115쪽 exact-miss 약 2초 탐색이 0회여야 한다.
- 일반 입력은 flush 0회, 44번째는 정확히 1회, 50자 누계는 1회여야 한다.
- 정확성 GREEN과 함께 stable-line fast path 호출 수·median/p95가 구조적으로 퇴행하지 않아야 한다.
- fresh scoped units와 old cut에서도 exact run이 없고 `cellOverflowed=false`인 별도 경계가
  발견되면 optional `exactHit` 신호를 임의로 추가하지 않고 중단·재승인한다.
- Canvas2D는 통과하지만 renderer contract, 정적 layer 표시 또는 rapid input이 퇴행하면
  Stage 3으로 돌아가 재승인한다.
- focused E2E의 기본 실행이 `--diagnose` 없이 HWP/HWPX GREEN이어야 한다.
- `mydocs/working/task_m100_2214_stage4.md`와 최종 E2E/test 보강을 같은 커밋에 포함하고
  승인받기 전 Stage 5로 가지 않는다.

커밋 메시지:

```text
Task #2214: Stage 4 - pre-cursor 경계 flush 및 Studio 정합
```

### Stage 5. 광역 게이트와 최종 보고

#### 목표

전체 Rust·Studio 게이트를 통과시키고, 원인·수정·정확성·성능 결과와 후속 범위를 문서화한다.

#### 작업

1. 전체 Rust test, clippy, fmt check를 실행한다.
2. Studio unit/build, renderer contract와 #2214 E2E를 다시 실행한다.
3. Stage 4 이후 production source가 바뀌지 않았고 동일 WASM을 검증했는지 hash/시각으로
   확인한다. 바뀌었다면 WASM을 다시 빌드하고 브라우저 검증을 반복한다.
4. `mydocs/working/task_m100_2214_stage5.md`와
   `mydocs/report/task_m100_2214_report.md`를 작성한다.
5. 작업 완료일의 `mydocs/orders/yyyyMMdd.md`에 #2214 상태와 검증 요약을 갱신한다.
6. #2193 종합 성능, boundary full flush를 bounded/partial paginator로 대체하는 후속과
   #2215 드래그 selection은 별도 범위임을 최종 보고서에 명시한다.
7. Stage 2 진단을 최종 PR용 영구 GREEN 계약으로 정리한다.
   - 95초 full matrix는 최종 production으로 1회 실행하되, case별 기대 assertion을 추가하기
     전에는 correctness GREEN이 아니라 diagnostic completion으로만 기록
   - cold/prewarm, direct/path 및 44/50자 대표 case는 빠른 non-ignored 회귀로 축약
   - cache-only/115쪽 probe의 핵심 assertion을 non-ignored GREEN으로 이관
   - intentional RED와 필수 `#[ignore]`, `--diagnose`-only acceptance가 남지 않았는지 확인
8. 검증 실패로 production 수정이 필요하면 Stage 5에 섞지 않고 해당 Stage로 돌아가
   변경 계획을 다시 승인받는다.

#### 검증

```bash
cargo fmt --check
cargo test --profile release-test --tests
cargo test --profile release-test --test issue_2214_cache_matrix_probe -- --ignored --nocapture
cargo clippy --all-targets --all-features -- -D warnings
```

```bash
cd rhwp-studio
npm test
npm run build
npm run e2e:renderer-contract
VITE_URL=http://127.0.0.1:7714 \
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
CHROME_EXTRA_ARGS="--force-device-scale-factor=1" \
node e2e/issue-2214-page-local-repaint.test.mjs --mode=headless
```

```bash
git diff --check
git status --short
```

`cargo fmt` 실행은 금지하고 `cargo fmt --check`만 사용한다. 기능과 무관한 전체 포맷 diff를
만들지 않는다.

#### 완료·승인 조건

- 모든 게이트가 GREEN이고 계획된 tracked 산출물 외 변경이 없어야 한다.
- matrix·수동 probe·대형 E2E의 핵심 계약이 정상 non-ignored GREEN 게이트로 이관되고,
  최종 PR acceptance에 expected RED나 `--diagnose` 전용 실행이 남지 않아야 한다.
- Stage 5 보고서, 최종 보고서와 오늘할일 갱신을 같은 커밋에 포함한다.
- 최종 승인 전 이슈 close, push, PR, 통합을 수행하지 않는다.

커밋 메시지:

```text
Task #2214: Stage 5 - 전체 회귀 검증 및 결과 보고
```

### Stage 6. E2E 발견성과 재발 방지 문서 보강

#### 목표

Stage 5 뒤 PR 자산 감사를 반영해 #2214 영구 E2E의 실행 진입점과 syntax gate를 추가하고,
지연 셀 편집의 cache/pagination/history 계약을 이슈 보고서 밖의 재사용 문서에 고정한다.

#### 작업

1. `rhwp-studio/package.json`에 headless #2214 E2E npm script를 추가한다.
2. Render Diff workflow의 기존 script syntax 단계에서 #2214 E2E도 `node --check`한다.
   전체 약 95초 E2E는 Linux 통제 환경을 별도로 검증하기 전 필수 CI 실행으로 추가하지 않는다.
3. `mydocs/troubleshootings/deferred_cell_edit_cache_coherence.md`에 다음 재발 진단법을 정리한다.
   - model/LINE_SEG, cell layout cache, page tree, cursor, Canvas의 층별 판정
   - cold/warm 및 direct/path 비교
   - stable deferred edit와 cell-flow 경계 분리
   - mutation → pending → boundary flush → cursor → refresh 순서
   - global clear, post-cursor flush, 환경 의존 timing hard gate 금지
4. `mydocs/tech/edit_action_undo_redo_architecture.md`에 실제 `TextMutationEffects` side-channel,
   one-shot 소비, redo 재계산, IME/iOS 누적과 pre-cursor flush 계약을 추가한다.
5. `mydocs/manual/edit_command_review_checklist.md`에 page-local cache scope와 normal/redo/raw
   입력의 effect·flush·reset 검토 항목을 추가한다.
6. Stage 6 보고서, 최종 보고서와 오늘할일을 같은 커밋에서 갱신한다.
7. raw JSON/PNG/timeline, `pkg/`, `target/`, `node_modules`, 새 fixture와 paginator 설계는
   포함하지 않는다.

#### 검증

```bash
cd rhwp-studio
node --check e2e/issue-2214-page-local-repaint.test.mjs
npm test
npm run build
npm run e2e:renderer-contract
```

Vite 7714와 기존 Stage 5 WASM을 사용해 npm script가 인자를 전달하는지 HWP/HWPX 각 1회로
검증한다.

```bash
VITE_URL=http://127.0.0.1:7714 \
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
CHROME_EXTRA_ARGS="--force-device-scale-factor=1" \
ISSUE2214_OUTPUT_ROOT=../output/poc/task2214/stage6-smoke \
npm run e2e:issue-2214 -- --runs=1
```

```bash
git diff --check
git status --short
```

#### 완료·승인 조건

- production Rust/Studio 동작 source와 Stage 5 WASM hash가 변하지 않아야 한다.
- #2214 E2E는 npm script로 발견 가능하고 Render Diff workflow에서 syntax를 검사해야 한다.
- 세 문서는 이슈별 수치 복제가 아니라 재사용 계약과 진단 순서를 담아야 한다.
- 검증 산출물은 ignored `output/`에만 생성하고 tracked golden으로 추가하지 않는다.
- Stage 6 보고서, 최종 보고서와 오늘할일 갱신을 같은 커밋에 포함한다.
- 별도 승인 전 push, PR 생성, 이슈 close를 수행하지 않는다.

커밋 메시지:

```text
Task #2214: Stage 6 - E2E 발견성 및 재발 방지 문서 보강
```

## 7. 전역 검증 계약

### 모델·저장

- 매 입력 뒤 exact suffix와 cursor offset이 증가한다.
- #2185 한 글자 `[0,44,84,122]`, `vpos=17160`, 115쪽 기준을 유지한다.
- 반복 입력 뒤 기존 네 line start는 prefix로 남고, 새 줄·다음 문단 vpos는 explicit flush와
  재로드 결과에 일치한다.
- HWP/HWPX 원본 형식 저장·재로드 뒤 text, line tuple, vpos, 115쪽이 동일하다.

### 화면·시간축

- 1~2 rAF 안에 대상 TextRun, 합성 화면과 caret 위치가 최신 입력과 일치한다.
- 850ms와 1.6초 뒤에도 최신 상태가 되돌아가지 않는다.
- caret opacity는 blink 때문에 실패 조건으로 사용하지 않고 display와 위치를 확인한다.
- 동일 세션의 pre/post flush 합성 crop은 허용 오차 안에서 동등하다.

### 성능·호출 구조

- 일반 입력은 scoped cache eviction과 기존 page-local/static reuse 경로를 유지하며 flush 0회다.
- 44번째 cell-flow 경계는 cursor 조회 전에 flush 정확히 1회, 50자 누계도 1회다.
- flush 0회인 일반 입력에서는 unrelated table cache hit가 유지된다. same-table sibling은 owner
  flag 불변일 때만 identity를 보존하고, flag 변화 시 owner table cell units를 재계산한다.
- cursor exact hit는 page 0에서 끝나며 115쪽 fallback scan은 0회다.
- render 호출 수와 median/p95가 Stage 1 기준 대비 구조적으로 퇴행하지 않는다. 시간 상한은
  고정 환경 보고 게이트로 두고 CI는 cache scope와 호출 횟수를 단언한다.

## 8. 전역 중단 조건

- HWP에서 결정적 재현이 되지 않거나 재현 위치·폰트 상태가 매 실행 달라진다.
- 모델 text·`LINE_SEG` 또는 원본 형식 저장 데이터가 시간 경과로 손실된다.
- scoped eviction 뒤에도 warm tree/cursor가 정확해지지 않는다.
- edited paragraph local contribution과 cached flag만으로 판정할 수 없거나, flag 변화 시
  owner-table-wide cell eviction/flag update를 unrelated cache 보존과 함께 구현할 수 없다.
- cell-flow boundary flush 뒤에도 115쪽 range, bounds 또는 cut이 oracle과 일치하지 않는다.
- 중첩 표 cache coherence에 ancestor generation/owner graph 재설계가 필요하다.
- 전체 renderer/paginator 또는 일반 증분 pagination 설계가 필요하다.
- HWP/HWPX parser·serializer, `korean_break_unit`, font metric 변경이 필요하다.
- #1918 fast path를 전면 철회하거나 매 입력 전체 refresh가 필요하다.

중단 조건이 발생하면 해당 Stage 보고서에 증거를 남기고 수행계획 또는 구현계획 개정 승인을
받는다.

## 9. 단계별 커밋과 승인 원칙

- 각 Stage의 source/test와 `mydocs/working/task_m100_2214_stageN.md`를 같은 커밋에 포함한다.
- 커밋 메시지는 `Task #2214: ...` 형식을 사용한다.
- 각 Stage 완료 후 작업지시자 승인 없이 다음 Stage로 넘어가지 않는다.
- 기능에 무관한 포맷, 다른 worktree 산출물, `scripts/frontend-metrics/`는 포함하지 않는다.
- 최종 보고서와 오늘할일 상태는 Stage 5 커밋에 포함한다. PR 자산 감사로 승인된 Stage 6에서는
  Stage 6 보고서와 함께 최종 보고서·오늘할일을 보정한다.
- 이슈 close, 원격 push, PR 생성, 브랜치 통합은 별도 승인 전 수행하지 않는다.

본 구현계획 승인 전에는 소스 코드와 테스트를 수정하거나 런타임 의존성을 설치하지 않는다.
