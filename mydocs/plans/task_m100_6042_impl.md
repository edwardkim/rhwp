# 구현 계획 — Task M100 #6042

- 작성일: 2026-08-31 KST
- 상태: **Stage 5 측정 중단, Stage 4 보정 재승인 대기**. 역방향 exact cache thrash를 확인했다.
- 현재 기준: #6467 `23b5bcf73f6e8659a90b25ebfde1311e1965364f`
- 역사 Stage 1 측정 기준: #6467 `ba68cd655aed5fd94804f725c033cf615231ce4b`
- branch: `codex/issue-6042-page-virtualization`
- 수행 범위·단계·fixture·수용 기준: [수행 계획서](task_m100_6042.md)
- Stage 2 결과: [완료 보고](../working/task_m100_6042_stage2.md)
- Stage 3 결과: [완료 보고](../working/task_m100_6042_stage3.md)
- Stage 4 결과: [완료 보고](../working/task_m100_6042_stage4.md)
- Stage 5 결과: [중단 보고](../working/task_m100_6042_stage5.md)

## 1. 경계와 파일별 변경안

기존 `VirtualScroll → CanvasView → PageRenderer` 경계를 유지한다. 좌표 모델·줌 renderer를 새로 만들지
않고, 계산 결과 재사용과 렌더 작업의 소유·순서만 분리한다. 파일명/타입명은 설계안이다.

| 파일 (`rhwp-studio/` 아래) | 변경안 | 단계 |
| --- | --- | --- |
| `src/dev/page-scroll-probe.ts`, `src/dev/scroll-observation.ts` (신규) | opt-in 관찰·bounded counters/marks·snapshot export. #6521 전체 이식 금지 | 1 완료 |
| `src/main.ts` | 기존 DEV 설치 경로의 최소 loader만. production에서는 미설치 | 1 |
| `src/view/virtual-scroll.ts` | row tops/bottoms/pages 인덱스, horizontal X 탐색, immutable visibility snapshot, 기존 wrapper 유지 | 2 |
| `src/view/canvas-view.ts` | snapshot 한 번 소비, page bundle/cache lifecycle, 명시적 render reason에 따른 스케줄 경로 | 2~4 |
| `src/view/page-surface-lru.ts` (신규) | DOM 비의존 keyed LRU·누적 pixel ledger·예산/무효화 | 3 |
| `src/view/page-renderer.ts`, `src/view/canvas-pool.ts` | main/overlay bundle 소유·detach/attach/dispose, 완료/교체/취소 식별 | 3~4 |
| `src/view/page-render-scheduler.ts` (신규) | clock/host 주입 큐·visible 우선·bounded idle·generation | 4 |
| `tests/`의 관련 기존/신규 suite | 순수 계산 + 실제 method를 실행하는 fake host/lifecycle 통합 테스트 | 각 단계 |

`viewport-manager.ts`, `zoom-anchor.ts`, `coordinate-system.ts`, `caret-renderer.ts`, `ruler.ts`는 원칙적으로
동작 변경하지 않는다. 현행 ruler는 focused page 한 장만 그리며 visibility query를 직접 하지 않는다.
snapshot을 Ruler에 억지로 직접 주입하거나 row decoration을 추가하지 않는다. 필요 이벤트는 기존
active/focused snapshot 경로로 전달하고, ruler rAF를 raster 완료 뒤로 미루지 않는다.

## 2. Stage 1 — 기준선·완료 시점부터 고정

[완료 보고](../working/task_m100_6042_stage1.md): 기준선 120회, off/on 교대 관찰 비용, 실제 문서 6종,
줌 경계·이미지 완료 smoke, 무수정 서버의 page-info 경고 재현, 98개 집중 테스트와 production asset
동일성 확인. compositor timing·DPR 1·backend 전체 수명 통합 검증은 완료한 것으로 주장하지 않는다.

1. 무수정 #6467 서버와 같은 SHA에 관찰만 추가한 서버를 분리한다. opt-in off/on에서 좌표·raster
   횟수·오류·시간이 관찰 허용 범위 안인지 확인한다. 조건·WASM·font hash를 고정한다.
2. scroll, zoom, document load, mutation, renderer fallback의 호출·이벤트 순서를 기록한다. 특히
   `refreshRenderSurfacePlan(true)`의 기존 Canvas rerender, 이미지 완료 후 재렌더를 main render와 별도로 센다.
3. #6521의 문서 전환 page info 경고는 무수정 상태/관찰 adapter/문서 load epoch별로 원인 분리한다.
   baseline 기존 문제면 별도 제한으로 기록하고, 도구가 유발하면 고친 뒤에만 계측을 신뢰한다.
4. 지표 수명은 `document identity + revision + interaction generation + request id`로 구분한다.
   한 interaction의 최신 목표를 고정하고 후속 입력은 이전 측정을 canceled/superseded 처리한다.
5. preview 완료는 page geometry/CSS와 ruler 갱신의 관측 경계 둘을 만족해야 한다. rAF callback 실행을
   화면 presentation 완료로 단정하지 않는다. focused가 비가시면 focused-sharp는 N/A다.
6. visible-stable은 최종 배율의 대상 visible bundle과 알려진 이미지/RawSvg 후속 작업이 안정된 시점,
   retained-complete는 그 세대에서 승인한 working set의 완료다. 단순 `renderPage` 반환·빈 큐를
   최종 완료로 기록하지 않는다. timeout/fallback/오류/취소를 빠른 성공으로 집계하지 않는다.

계측은 제품 scheduling을 소유하지 않는다. hot path에는 상수 시간 카운터·시간 mark만 두고 DOM scan,
PNG readback, trace 직렬화는 별도 명시 실행으로 뺀다. 고정 용량 ring buffer를 사용한다. #6521의
`zoom-performance.ts`는 계약 참고 자료일 뿐 cold image·fallback·document switch 검증 없이 복사하지 않는다.

## 3. Stage 2 — 행 인덱스와 visibility snapshot (완료)

### 3.1 Geometry의 단일 권위

`setPageDimensions`의 배치 산식은 그대로 둔다. 이미 생성된 row별 page list에서 `rowTop`,
`rowBottom = max(pageTop + pageHeight)`를 저장한다. mixed-size 페이지·표지 blank slot·미완성 마지막
행을 다룬다. blank slot은 페이지로 만들지 않는다. 구축 비용은 기존 layout 시점의 O(pages)이다.

세로 모드: `rowBottom > viewportTop`의 첫 행과 `rowTop >= viewportBottom`의 끝 행을 이진 탐색한 뒤
후보 page에 기존 strict AABB 교차 조건을 적용한다. 비용은 O(log rows + 후보 행의 pages)이며
임의의 넓은 grid에서 후보 열 수까지 O(1)이라고 주장하지 않는다.

가로 모드: 모든 page가 row 0이므로 monotonic left/right를 X 이진 탐색하고 page별 Y 교차를 검사한다.
가로 쪽의 세로 중앙 정렬은 변경하지 않는다. 무한 가로 viewport sentinel 등 기존 API 경계도 보존한다.

`getPageAtY`의 행 마지막 page 반환, `getRowFirstPageAtY`의 첫 page, gap/바깥 좌표 fallback과
`getPageAtPoint`의 X 선택을 참조 구현과 비교한다. 앵커가 다른 쪽을 선택하게 만드는 의미 변경은 금지한다.

### 3.2 Snapshot 계약

snapshot key: `geometryRevision + scrollX + scrollY + viewportWidth + viewportHeight`.
geometryRevision은 document/layout/zoom/resize 재계산 시 갱신한다. '같은 frame'만으로 재사용하지 않는다.
동일 frame 안에 geometry가 바뀌면 다른 snapshot을 반환한다. 최근 key/result 하나만 보존해 스크롤 좌표
종류만큼 메모리가 늘어나지 않게 한다. 외부에는 readonly arrays·정보를 제공한다.

결과: visible pages/rows, 첫·끝 visible row, 인접 후보, 조회 통계. 조회 통계와 hit 카운터는 소비자 수에
따라 실제 조사량이 부풀지 않도록 분리한다. 기존 getVisiblePages/getPrefetchPages wrapper도 같은 결과를
소비한다. CanvasView는 한 snapshot으로 current visible·active page·working set을 정한다.

새 snapshot이 없을 때 이전 문서 snapshot을 peek해 쓰지 않는다. 빈 문서/교체 중에는 유효한 empty를
반환하고 Wasm page count 밖의 page info를 조회하지 않는다. reset 이벤트의 기존 관측 계약도 테스트한다.
Stage 1에서 무수정 #6467의 문서 전환 page-info 경고를 재현했다. `pages=[]`와 이전 VirtualScroll
geometry가 공존할 때 늦은 viewport-scroll이 render를 시도하는 경로를 실행 기반 테스트로 고정하고,
새 empty snapshot으로 무효화한다. placeholder의 중앙 정렬·줌 앵커 의미는 바꾸지 않는다.

### 3.3 검증

- 기존 선형 predicate를 test oracle로 삼아 single/double/facing/multiple/auto/horizontal, 서로 다른
  높이/폭, 경계·gap·빈 문서·음수 scroll·같은 frame layout 변경을 비교한다.
- 큰 합성 geometry는 알고리즘 조사량 테스트 전용이며 실문서 성능 자료로 쓰지 않는다.
- `active-page*`, `virtual-scroll-*`, `page-scroll-step`, `viewport-manager-smooth-zoom`,
  `ruler-document-load-refresh`, `ruler-label-geometry`, `canvas-view-page-arrangement` 회귀를 실행한다.

## 4. Stage 3 — 페이지 표면 LRU와 예산

### 4.1 Bundle과 단일 소유권

`PageSurfaceBundle`은 main Canvas, 생성된 background/behind/front/flow-static Canvas, 필요한 page
장식/diagnostics 메타데이터, 정확한 raster key, 완료 상태, lease id를 한 묶음으로 소유한다.
CanvasKit이 Canvas를 replacement로 바꾸면 실제 반환된 Canvas와 모든 ledger를 원자적으로 갱신한다.
diagnostics 삭제는 GPU 자원 해제의 증거가 아님을 구분한다.

제안 수명: `active visible/prefetch → completed detached LRU → active`, 또는 `→ disposed/pool`.
retained bundle을 익명 CanvasPool에 동시에 반환하지 않는다. LRU로 보낼 때 overlay를 삭제하는 기존
`removePageLayers` 경로와 구분한다. cache hit는 전체 bundle을 올바른 z-order로 재부착하고 현재
geometry·CSS·dataset·grid decoration을 갱신한다. raster는 호출하지 않는다.

in-flight/부분 결과는 완성 cache hit가 아니다. 화면 밖으로 보내면서 작업을 취소한 bundle은 폐기하거나
명시적인 incomplete 상태로 취급하고, 재진입 시 필요한 render를 수행한다. 캐시 hit 통계를 올리지 않는다.
기존 텍스트 편집 static layer 검증·image retry signature 계약(#3315 등)을 LRU 이동마다 초기화하지 않는다.

예산 퇴거 시 main/overlay를 소유권별로 해제하고, 익명 pool로 넘길 Canvas는 backing store를 0-size로
비우는 안을 우선 검증한다. DOM 요소 pool 개수도 제한한다. 이 비용이 왕복 성능을 해치는지 측정하며
CanvasKit context/software fallback이 재할당 가능한지 확인한다. 안전하지 않은 backend는 명시적 miss로
돌리고 별도 승인 없이 자원 재사용을 강행하지 않는다.

### 4.2 Cache key와 무효화

키는 최소 다음 항목을 포함한다.

- 문서 digest/generation + content revision (미확정이면 보수적 document epoch)
- page index와 page geometry/content version
- 실제 backend·renderer instance/selection epoch·render profile·결과에 영향을 주는 view options
- **#6467이 정한 최종 effective DPR → clampRenderScale 결과와 실제 integer width/height**
- raster에 영향을 주는 font/resource generation 및 layer 구성

`balanced` 같은 tier 이름만 같거나 scale을 거칠게 반올림했다는 이유로 hit를 내지 않는다. CSS 표시
zoom과 bitmap scale을 혼동하지 않는다. invalidation 범위를 확실히 모르는 편집은 전체 문서 cache를
버린다. 정확한 부분 revision 계약을 만들기 위해 Rust API까지 넓히지 않는다.

document replacement/reload, mutation/undo/redo, backend/profile/font 변화, 기존 zoom/resize release,
dispose에서 cache·예약·queue를 무효화한다. 순수 scroll generation은 bitmap content key가 아니다.
스크롤이 바뀌었다는 이유만으로 완성된 유효 bitmap까지 모두 miss로 만들지 않는다.

### 4.3 예산 합성 — 캐시 때문에 화질을 더 낮추지 않는다

기존 상수 `DEFAULT_VISIBLE_SURFACE_PIXEL_BUDGET=32_000_000`,
`DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET=40_000_000`을 정본으로 사용한다. per-Canvas safety cap
67,108,864px를 새 총 캐시 예산으로 복제하지 않는다.

1. 현재 visible와 기존 범위의 prefetch 후보에 대해 #6467 planner가 결정한 DPR을 구한다. 이전 DPR
   hysteresis·focused 보호도 그대로 둔다. 과거 LRU 항목을 planner 입력에 추가해 강등을 유발하지 않는다.
2. active bundle + 승인한 pending render의 예상 비용을 예약하고, **남은 headroom에만** 최근 bundle을
   보존한다. 새 작업 전 reservation으로 LRU를 퇴거해 allocation 뒤에만 뒤늦게 한도를 맞추지 않는다.
3. 초기 prefetch 후보 범위는 기존 ±1행/가로 인접 page 이내로 유지한다. 방향/속도는 우선순위·중단·실행
   시점에 반영하며 먼저 범위를 늘리지 않는다. 캐시에 밀려 visible 화질이 낮아지는 역전을 금지한다.
4. 추정 비용은 최종 clamp·정수 dimension·layer 상한으로 예약하고, 생성/교체/삭제 때 실제 모든 Canvas의
   `width×height`로 조정한다. optional layer의 추정 상한과 실제 생성량을 각각 기록한다.
5. active / detached LRU / idle pool / in-flight reservation을 중복 없이 합산한다. 누적 total을 갱신해
   매 eviction마다 전체 cache를 다시 합산하지 않는다. Map 순서 또는 연결 목록으로 touch/삭제 O(1),
   eviction은 실제 victim 수에 비례하게 한다. DOM query는 lifecycle 경계에 한정한다.
6. 기존 보호 쪽/출력 profile 때문에 mandatory 비용만으로 한도를 넘으면 extra LRU=0·선택 prefetch를
   억제하고 `overBudgetMandatory`를 진단한다. 같은 페이지를 무한 퇴거/재할당하거나 DPR을 추가 강등하지 않는다.

Stage 3에서는 planner 입력의 prefetch 후보 범위를 유지한 상태로 baseline DPR 동등성을 검사한다.
후속 prefetch 억제가 필요한 경우 품질이 더 나빠지지 않는지 별도 비교한다. total ledger는 RGBA surface
환산이지 WASM heap·decoded image cache·GPU/RSS 전체가 아니며, 실제 해제 지연도 별도 한계다.

### 4.4 검증

- `page-surface-lru.test.ts`: 정확한 key, LRU 순서, 누적 비용, reserve/reconcile, 보호 쪽 초과, eviction·clear.
- `canvas-view-surface-cache.test.ts`: 실제 method와 fake renderer/pool로 다층 attach/detach, Canvas replacement,
  cold/incomplete miss, 완성 warm hit raster 0, mutation/backend/zoom/document/dispose 무효화.
- 실제 KTX 4-layer와 21쪽 다층 문서에서 main만 보이는 누락·wrong z-order·잔상·풀 이중 소유가 없는지 확인.
- image decode 완료, raw SVG early timer, fallback, 취소 후 callback을 인위적으로 늦춰 stale write가 없는지 검사.
  테스트는 소스 문자열에 이름이 존재하는지 확인하는 데 그치지 않는다.

## 5. Stage 4 — 일반 스크롤 scheduler부터 연결

### 5.1 호출 경로를 명시적으로 나눈다

`updateVisiblePages`에 `scroll | initial | zoom-settled | resize | mutation | strict` 같은 이유를
전달하는 작은 경계를 둔다. 초기/편집/줌/resize/strict의 동기 visible 의미는 유지한다. strict
`refreshDocumentAgentMutation()`은 응답 전에 현재 visible render를 확인하므로 큐에 넣고 성공을 반환하지 않는다.

scroll 모드에서만 newly visible와 `refreshRenderSurfacePlan(true)`의 DPR 변경 raster를 동일 page별
큐로 합친다. 후자만 동기로 먼저 실행하면 분할 효과가 사라진다. visible·현재 focus의 필수 갱신은 우선하고
offscreen 변경은 낮은 우선순위로 미룬다. tier label만 바뀌고 실제 scale/내용이 같으면 추가 raster하지 않는다.
다른 render reason의 즉시 편집/품질 보호 계약은 함께 비동기로 바꾸지 않는다.

### 5.2 우선순위·시간 예산·진행성

우선순위: 필요한 visible focused page → viewport 중심/노출 면적 기준 visible → 나머지 visible →
방향 prefetch. 유효한 cache hit는 raster 큐 없이 재부착한다. 비가시 focus를 새로 렌더해 visible보다 앞세우지 않는다.

시간 기반 slice를 사용하며 `3열 이상` gate로 정책을 정하지 않는다. Stage 1 탐색 후보는 visible slice
4ms, 한 slice 최대 2개 page, idle 한 번 최대 1개 page다. **이 수치는 아직 채택되지 않았고** 기준선
측정으로 확정·승인한다. 작고 즉시 처리 가능한 1·2 visible 작업은 기존 동기 표시를 유지하는 fast path를 둔다.
initial은 열 수와 무관하게 기존 경로를 보존한다. page 내부 WASM 호출은 선점 불가하므로 시간 상한은 soft다.
한 쪽만으로 long task가 생기면 이 scheduler가 해결했다고 주장하지 않고 별도 병목으로 기록한다.

Stage 4에서는 위 수치를 page-boundary 실험값으로 적용했다. 실문서에서 한 쪽 raster가 4ms를 넘는 조건을
확인했으므로 최종 정책 채택과 조정은 [Stage 4 보고](../working/task_m100_6042_stage4.md)를 입력으로 Stage 5
A/B·사용자 검증 뒤 결정한다.

host에 now/rAF/cancel/idle/timer를 주입한다. 예산 소진/새 입력/idle deadline 부족 시 후속 task로 양보한다.
기다리는 visible가 있는데 prefetch가 선행하지 않게 하고, timeout fallback도 한 번에 한정 작업만 한다.
연속 입력 중에도 최신 visible의 최소 진행을 보장한다. 같은 page가 계속 필요하면 queue 중복 생성 대신
우선순위를 갱신하고, 무조건 매 frame 취소·재예약해서 표시가 영원히 밀리지 않도록 한다.

direction/speed는 scroll delta와 monotonic time의 상수 시간 요약으로 계산한다. 수직은 row, 수평은 X
page 진행을 사용한다. 빠른 스크롤·반전·zoom 시작에는 낡은 prefetch를 우선 취소한다. 속도 때문에
overscan 범위를 늘리거나 backend/DPR 정책을 새로 결정하지 않는다.

### 5.3 비동기 세대와 lease

큐 항목은 document/content epoch, geometry epoch, desired-work generation, page raster key, bundle lease를
가진다. dispatch 직전과 결과를 화면/diagnostics에 반영하기 전에 유효성을 확인한다. 새로운 scroll은
현재 필요 없는 작업을 취소하고 필요한 작업은 최신 우선순위로 승계할 수 있다. document/backend/zoom
경계는 더 강한 무효화로 구분한다.

PageRenderer의 기존 job identity/token 가드를 유지하되, `canvas.parentElement`만으로 현재 소유를 판정하지
않는다. 반환된 Canvas를 다른 페이지가 사용하거나 이전 decode callback이 늦게 도착하는 경우를 lease로
차단한다. 공유 image decoder 자체를 무조건 취소하지 않고 해당 렌더 작업의 결과 적용 권한을 취소한다.
reset/dispose는 예약·가드를 먼저 무효화해 reentrant focus 이벤트가 이전 작업을 부활시키지 못하게 한다.

### 5.4 Stage 5 발견에 따른 보정안 — 재승인 대기

Stage 5의 `exam_kor` 4열·34% 역방향 20회에서 Stage 4가 기준선보다 매회 main raster 두 쪽을 더
수행했다. 대표 trace는 Stage 3의 `18 → 19 → 4`와 exact hit `5 → 6 → 7`이 Stage 4에서 exact hit
`7`, raster `6 → 5 → 4 → 19 → 18`로 바뀐다. retained-complete p95는 227.8ms에서 310.5ms로
36.3% 악화됐다.

원인은 scroll DPR 전환을 미룬 뒤 cache admission 순서를 그대로 둔 데 있다. target에서 작아질 active
surface가 아직 이전 큰 actual 크기인 동안 새 detached bundle을 `put`·trim해, 곧 생길 headroom보다
먼저 exact bundle을 퇴거한다. 이후 실제 downscale로 여유가 생겨도 퇴거한 bundle은 복구되지 않는다.
또한 `overBudgetMandatory` 상태에서도 선택 prefetch를 dispatch 전에 중단하는 gate가 없다.

보정 범위는 다음으로 제한한다.

1. scheduler work를 visible 필수, active target-DPR 전환, 선택 prefetch로 구분해 진단과 admission을
   서로 섞지 않는다. 화질·DPR planner·좌표·줌 경로는 바꾸지 않는다.
2. content/layer 구성이 같은 active DPR 전환은 current layer 수와 target integer surface 크기로
   target-state pixel을 예약한다. 확대 delta는 allocation 전에 확보하고 축소는 완료 직후 actual로
   reconcile한다. descriptor의 최대 layer 추정이나 이전 큰 actual 중 하나만으로 cache를 조기 trim하지
   않는다.
3. 한 update에서 막 detach한 exact bundle은 target-state reconcile이 끝날 때까지 별도 pending admission으로
   ledger에 한 번만 센다. 이는 추가 메모리 예산이 아니며 새 allocation을 허용하기 위한 headroom으로
   중복 계산하지 않는다. generation 취소·오류·문서 경계에서는 즉시 일반 LRU admission 또는 폐기한다.
4. 선택 prefetch의 `isValid`/dispatch 경계에서 실제 headroom과 target bundle 비용을 다시 확인한다.
   mandatory target 예약이 budget을 넘거나 완료 bundle을 보존할 수 없으면 skip하고 진단 카운터를 남긴다.
5. 실행 테스트는 downscale 직전 exact cache 4쪽, visible DPR 전환 2쪽, 역방향 복귀를 구성한다. 임시
   actual 크기 때문에 exact 두 쪽을 잃지 않는지, 확대 전환이 allocation 전에 퇴거하는지,
   `overBudgetMandatory` prefetch가 raster 0인지 확인한다.

같은 A/B를 다시 실행해 `exam_kor` 역방향 raster가 기준선 3회 이하이고 p50/p95가 사전 경보선 안이어야
한다. 동시에 178쪽 cold first-visible·visible-stable·retained 개선, 최종 ledger, 1·2쪽 zoom fast path를
다시 검증한다. 이 보정은 [Stage 5 보고](../working/task_m100_6042_stage5.md)의 사용자 재승인 전 구현하지
않는다.

## 6. Stage 5~6 검증과 철회 조건

순수 scheduler fake-clock tests와 CanvasView 실행 기반 integration tests를 함께 쓴다. 최소 항목:

- 현재 visible 우선, 1·2쪽 초기 첫 표시 보존, 많은 visible의 분할, idle timeout fallback, 시간 부족·방향 반전.
- 같은 page의 중복 raster 제거, 최종 DPR 결정 변경 직후 stale job 기각, 빠른 입력 중 starvation 없음.
- scroll 도중 zoom 시작/종료, resize, document load, edit·undo·redo, renderer fallback, dispose.
- document-agent strict 응답, 캐럿/선택/hit-test와 현재 쪽/눈금자 focus 의미 보존.
- image pending/failure/RawSvg fallback이 완료 시간·캐시 완성 상태를 거짓으로 앞당기지 않음.
- active+LRU+pool+예약 ledger가 생성·교체·퇴거·오류 모든 경로에서 일관됨.

실제 문서 조건·반복 횟수·캡처는 수행 계획의 matrix를 따른다. 시간 수치 외에 queue/raster 카운터로 원인을
설명한다. 같은 displayed scale의 문서를 흐리게 하거나 늦게 표시해 얻은 시간을 개선으로 포장하지 않는다.
늘어난 메모리·visible-stable 지연·prefetch 완료 지연이 있으면 절감한 CPU와 함께 비교해 수용 여부를 받는다.

화질/좌표/문서 수명 회귀 또는 1·2쪽 초기 UX 악화가 생기면 해당 Stage의 변경을 추가로 쌓지 않고 원인을
분리한다. 승인 없이 안전한 하단 stack이나 사용자 변경을 reset하지 않는다. LRU/scheduler를 도입할 이득이
입증되지 않으면 인덱스 개선만 유지하는 축소안도 **계획 변경 승인 후** 선택한다.

최초 계획 커밋은 계획만 포함했고, Stage 1은 별도 관찰·검증 커밋으로 잇는다. Stage 6 완료 뒤에도
push·native Draft PR 생성은 별도 승인,
기존 #6444 처리·전체 stack Ready·merge는 또 다른 승인 경계다.
