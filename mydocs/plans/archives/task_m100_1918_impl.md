# 구현 계획서 - Task #1918

**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**마일스톤**: M100 / v1.0.0 editing foundation stage
**수행계획서**: `mydocs/plans/task_m100_1918.md`
**작성일**: 2026-07-05

---

## 1. 설계 요지

이번 작업의 1차 목표는 표 입력/삭제 후 발생하는 page-local refresh에서 **변하지 않은 정적 이미지 레이어를
반복 렌더링하지 않도록 하는 것**이다. 표 텍스트 편집 자체는 빠르며, 병목은 편집 후 같은 페이지의
정적 이미지성 요소가 매 입력마다 다시 렌더링되는 경로에 있다.

Stage 1 계측 결과, 문제 샘플은 두 부류로 나뉜다.

- `samples/복학원서.hwp`: 같은 text edit refresh에서 `flow/background/behind/front`가 모두 반복 렌더링되는
  overlay 재생성 병목.
- `samples/253E164F57A1BC6934-empty.hwp`, `samples/143E433F503322BD33.hwp`: 별도 behind/front overlay는
  없지만 flow plane 내부의 정적 이미지/OLE·RawSvg 비용이 커지는 병목.

따라서 Stage 2-3은 overlay 반복 렌더링 제거에 집중하고, Stage 4에서는 이미지 decode 재렌더 안전망과 함께
flow 내부 정적 이미지 비용을 완화하는 별도 경로를 검토한다.

수정은 `rhwp-studio` 렌더 갱신 경로를 중심으로 봉인한다. Rust 렌더러의 이미지 해석이나 HWP 파서
동작은 변경하지 않는다. 다만 이미 존재하는 경량 API인 `getPageOverlayImages`가 page layer summary를
위해 활용 가능한지 검증하고, 부족하면 summary 전용 확장을 최소 범위로 추가한다.

핵심 방향:

- `document-page-invalidated`의 `reason: 'text-edit'`를 page-local text edit fast path로 사용한다.
- `PageRenderer`에 정적 overlay canvas 재사용 경로를 추가한다.
- overlay 재사용 여부는 보수적인 cache key로 판단한다.
- flow canvas는 계속 다시 렌더링하되, background/behind/front overlay canvas는 변하지 않으면 유지한다.
- overlay summary는 replay-plane 기준으로 계산해 부모 layer의 `TextWrap`으로 분류되는 이미지까지 포함한다.
- 이미지 디코드 재렌더 안전망은 유지하되, 동일 이미지 count에서 overlay까지 반복 재렌더링하지 않도록 분리한다.
- overlay가 없는 문서의 flow 내부 이미지/OLE 비용은 Stage 4에서 별도 완화 대상으로 다룬다.

## 2. 현재 코드 기준

라인 번호는 `local/task1918` 생성 시점의 `054be69c` 기준이다.

| 경로 | 현재 동작 |
|------|-----------|
| `rhwp-studio/src/engine/input-handler.ts:2234` | `afterPageLocalEdit`가 `document-page-invalidated`를 `reason: 'text-edit'`와 함께 emit |
| `rhwp-studio/src/view/canvas-view.ts:274` | `refreshInvalidatedPage`가 기존 canvas에 `renderCanvas`를 호출 |
| `rhwp-studio/src/view/canvas-view.ts:156` | `renderCanvas`가 page 위치/scale을 설정하고 `PageRenderer.renderPage` 호출 |
| `rhwp-studio/src/view/page-renderer.ts:25` | `renderPage`가 flow를 렌더한 뒤 `applyOverlays` 수행 |
| `rhwp-studio/src/view/page-renderer.ts:97` | `applyOverlays` 시작 시 기존 overlay canvas를 무조건 제거 |
| `rhwp-studio/src/view/page-renderer.ts:162` | `createFilteredCanvasLayer`가 background/behind/front layer마다 WASM 렌더 호출 |
| `rhwp-studio/src/view/page-renderer.ts:227` | `getLayerPlaneSummary`가 전체 `PageLayerTree` JSON을 받아 parse |
| `rhwp-studio/src/view/page-renderer.ts:344` | 이미지 decode 안전망 재렌더가 flow와 overlay를 모두 다시 렌더 |
| `src/document_core/queries/rendering.rs:761` | `get_page_layer_tree_native`는 전체 tree JSON 반환 |
| `src/document_core/queries/rendering.rs:774` | `get_page_overlay_images_native`는 behind/front image 목록과 imageCount를 반환하는 경량 API 후보 |

## 3. 단계 (5단계, stage-gated)

### Stage 1 - 재현 계측과 fast path 경계 고정

목표: 구현 전후를 비교할 기준과 "어떤 refresh에서 overlay 재사용을 허용할지"를 고정한다.

작업:

- `samples/복학원서.hwp`, `samples/253E164F57A1BC6934-empty.hwp`,
  `samples/143E433F503322BD33.hwp`, `samples/table-001.hwp`의 현재 벤치 값을 재확인한다.
- `rhwp-studio`에 임시 계측 또는 개발자 콘솔 계측 지점을 두어 다음 호출 횟수와 시간을 확인한다.
  - `renderPageToCanvasFiltered(..., 'flow')`
  - `renderPageToCanvasFiltered(..., 'background' | 'behind' | 'front')`
  - `getPageLayerTree`
  - `getPageOverlayImages`
- `refreshInvalidatedPage` payload의 `reason === 'text-edit'`만 fast path 대상으로 삼는 설계를 확정한다.
- 표 편집이 page count를 바꾸거나 page index가 유효하지 않을 때는 기존 full refresh로 유지한다.

산출:

- Stage 1 보고서: 기준 벤치, 호출 횟수, fast path 적용 조건.
- 소스 변경이 필요하면 계측 코드는 최종 구현에 남기지 않는다.

검증:

- `target/debug/rhwp bench ... -n 5`
- 수동 또는 E2E 기반 입력 계측 1회.

### Stage 2 - overlay summary 경량화

목표: 매 page-local text edit마다 전체 `PageLayerTree` JSON을 parse하지 않도록 한다.

작업:

- `WasmBridge.getPageOverlayImages(pageNum)`를 `PageRenderer`에서 사용할 수 있도록 summary parser를 추가한다.
- 기존 `LayerPlaneSummary`를 다음 형태로 확장한다.
  - `hasBehind`
  - `hasFront`
  - `imageCount`
  - `rawSvgCount`
  - `signature`
- `getPageOverlayImages`가 제공하는 `hasBehind`, `hasFront`, `imageCount`, `rawSvgCount`를 먼저 사용한다.
- Rust 쪽 `getPageOverlayImages`는 이미지 자체의 `text_wrap`뿐 아니라 부모 `RenderLayerInfo`를 반영한
  replay-plane 기준으로 `hasBehind`/`hasFront`를 계산한다.
- `rawSvgCount` 또는 page background 여부가 부족하면 다음 중 보수적인 쪽을 선택한다.
  - 전체 `PageLayerTree` fallback을 유지한다.
  - Rust에 summary 전용 API를 최소 추가한다.
- summary 실패 시 기존 `getPageLayerTree` 경로로 fallback한다.

가드레일:

- summary API가 부모 layer 기반 behind/front 분류를 빠뜨리지 않는지 `samples/복학원서.hwp`로 확인한다.
- `samples/253E164F57A1BC6934-empty.hwp`는 overlay가 아니라 flow 내부 정적 이미지 비용이 핵심일 수 있으므로,
  Stage 2 완료 판단에서 이 샘플의 지연을 overlay 개선만으로 해결했다고 보지 않는다.
- `Picture` 워터마크만 기준으로 분기하지 않는다.
- RawSvg/OLE는 이미지 decode 재렌더 트리거에 계속 포함한다.

테스트:

- `rhwp-studio/tests/render-backend.test.ts`에 정적 검사 추가 후보:
  - `PageRenderer`가 page-local overlay summary에 `getPageOverlayImages`를 우선 사용한다.
  - 실패 시 `getPageLayerTree` fallback이 남아 있다.
- Rust API를 변경하는 경우 `src/wasm_api/tests.rs` 또는 rendering query 테스트 추가.

검증:

- `cd rhwp-studio && npm test -- render-backend`
- Rust API 변경 시 관련 `cargo test` subset.

### Stage 3 - text edit 전용 overlay canvas 재사용

목표: `reason: 'text-edit'`로 들어온 page-local refresh에서 정적 overlay canvas를 재사용한다.

작업:

- `CanvasView.refreshInvalidatedPage`에서 payload reason을 파싱해 `renderCanvas`에 refresh reason을 전달한다.
- `CanvasView.renderCanvas`와 `PageRenderer.renderPage`에 선택적 render context를 추가한다.
  - 예: `{ reason?: 'text-edit' | 'unknown'; allowStaticOverlayReuse?: boolean }`
- `PageRenderer`에 page/layer 단위 cache key를 둔다.
  - page index
  - render scale
  - canvas physical width/height
  - layer kind
  - summary signature
  - render profile
  - backend
- `applyOverlays`가 기존 overlay canvas를 무조건 제거하지 않고, text edit fast path에서 key가 같으면 유지한다.
- flow canvas는 항상 다시 렌더링하고 margin guide도 다시 그린다.
- background/behind/front overlay는 key miss 또는 non-text refresh에서 기존처럼 새로 렌더링한다.

가드레일:

- zoom, DPR, page size, page left/top/transform이 달라지면 overlay box는 반드시 갱신한다.
- overlay canvas를 재사용하더라도 `applyPageLayerBox`는 매번 호출해 위치와 CSS 크기를 최신화한다.
- page count 변화, pageIndex 불일치, canvas pool release, full `document-changed`에서는 기존처럼 overlay를 제거한다.
- CanvasKit backend는 우선 대상에서 제외하고 기존 경로를 유지한다.

테스트:

- `rhwp-studio/tests/render-backend.test.ts` 또는 신규 `page-renderer-static-overlay.test.ts`:
  - text edit refresh에서 `createFilteredCanvasLayer` 계열 또는 `renderPageToCanvasFiltered(..., 'behind')`가 재호출되지 않는 구조 검증.
  - scale/signature 변경 시 overlay 재렌더가 발생하는 구조 검증.
  - non-text refresh에서는 기존처럼 overlay를 재생성하는 구조 검증.

검증:

- `cd rhwp-studio && npm test`
- 대표 샘플 수동 입력 확인.

### Stage 4 - flow 내부 정적 이미지 비용과 decode 재렌더 안전망 분리

목표: Task #1154/#1456 계열의 이미지 decode 안전망은 유지하되, text edit마다 overlay 또는 flow 내부 정적
이미지/OLE까지 반복 고비용 처리되는 일을 줄인다.

작업:

- `scheduleReRender`와 `reRenderPageCanvases`에 render context 또는 overlay reuse policy를 전달한다.
- text edit fast path에서 이미지 count/signature가 기존과 같으면 지연 재렌더가 flow 중심으로만 동작하도록 분리한다.
- 최초 렌더, imageCount 변화, rawSvgCount 변화, overlay signature 변화 시에는 기존처럼 overlay까지 재렌더한다.
- overlay가 없는 페이지는 `samples/253E164F57A1BC6934-empty.hwp`, `samples/143E433F503322BD33.hwp` 기준으로
  flow 내부 정적 이미지/OLE가 매 입력마다 다시 decode·draw 되는지 확인하고, 가능한 경우 정적 이미지 summary/cache
  또는 image prefetch 경로로 비용을 낮춘다.
- `prefetchLayerImages`가 전체 `PageLayerTree` JSON을 정규식으로 훑는 비용을 유지해야 하는지 재검토한다.
  - 가능한 경우 Stage 2의 경량 overlay JSON을 사용한다.
  - rawSvg data URL 추출이 필요하면 기존 fallback을 유지한다.

가드레일:

- 이미지가 최초 로드되지 않아 빈 상태로 남는 #1154/#1456 회귀를 만들지 않는다.
- OLE/rawSvg가 있는 `samples/143E433F503322BD33.hwp`는 반드시 확인한다.
- behind/front overlay가 없는 페이지에서는 추가 비용이 생기지 않아야 한다.

테스트:

- 기존 `rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs`를 유지 실행한다.
- 필요 시 renderer contract 테스트에 `scheduleReRender`가 overlay 재사용 정책을 유지한다는 정적 검사 추가.

검증:

- `cd rhwp-studio && npm test`
- `cd rhwp-studio && npm run e2e:renderer-contract`
- 필요 시 `node e2e/issue-1456-chart-rerender.test.mjs`.

### Stage 5 - 성능 검증, 회귀 확인, 보고

목표: 샘플별 체감 지연과 렌더 호출이 실제로 줄었는지 확인하고 문서화한다.

작업:

- 문제 샘플 3종에서 표 입력/삭제 후 다음 항목을 before/after로 기록한다.
  - flow 렌더 횟수
  - background/behind/front 렌더 횟수
  - `getPageLayerTree`/`getPageOverlayImages` 호출 횟수
  - 1회 입력 후 refresh 소요 시간
- `samples/table-001.hwp`에서 일반 표 편집 무회귀 확인.
- 시각 확인:
  - 워터마크/배경 이미지가 편집 전후 유지되는지 확인
  - 커서/선택 표시가 overlay 뒤/앞에 가려지지 않는지 확인
  - zoom 변경 후 overlay 위치가 틀어지지 않는지 확인
- 단계별 보고서와 최종 보고서 작성.

검증 명령:

```bash
cargo test
```

```bash
cd rhwp-studio
npm test
npm run build
```

필요 시:

```bash
cd rhwp-studio
npm run e2e:renderer-contract
node e2e/issue-1456-chart-rerender.test.mjs
```

## 4. 캐시 키 초안

초기 구현은 보수적인 문자열 signature를 사용한다.

```text
page={pageIdx}
scale={renderScale}
width={canvas.width}
height={canvas.height}
profile={renderProfile}
backend={backend}
hasBehind={summary.hasBehind}
hasFront={summary.hasFront}
imageCount={summary.imageCount}
rawSvgCount={summary.rawSvgCount}
behindSig={behind image bbox/wrap/effect/brightness/contrast/opacity/crop/transform/base64 length or hash}
frontSig={front image bbox/wrap/effect/brightness/contrast/opacity/crop/transform/base64 length or hash}
```

주의:

- base64 전체를 key에 넣지 않는다. 길이와 짧은 hash를 사용한다.
- key 계산이 이미지 재렌더보다 비싸지 않게 한다.
- Rust summary가 hash를 제공하지 않으면 Stage 2에서는 JSON 문자열 자체의 간단한 hash를 사용하고,
  Stage 3 이후 필요할 때 Rust summary hash를 추가한다.

## 5. 회귀 리스크

| 리스크 | 대응 |
|--------|------|
| overlay stale로 워터마크/앞쪽 객체 위치가 틀어짐 | scale/page size/transform/signature mismatch 시 재렌더 |
| 편집으로 페이지 구조가 바뀌었는데 overlay 재사용 | pageCount/pageIndex 유효성 검사 실패 시 기존 full refresh |
| `BorderFill` 이미지 채움 누락 | `253E164F57A1BC6934-empty.hwp`를 Stage 2/5 필수 검증 샘플로 사용 |
| OLE/rawSvg 재렌더 회귀 | `143E433F503322BD33.hwp`와 `issue-1456-chart-rerender` 확인 |
| 이미지 decode 안전망 회귀 | 최초 렌더와 imageCount/signature 변화 시 기존 재렌더 유지 |
| 메모리 증가 | page 단위 overlay cache만 사용하고 `releaseAllRenderedPages`/`removePageLayers`/`dispose`에서 해제 |
| CanvasKit 경로 회귀 | 이번 단계에서는 Canvas2D overlay 경로에 한정, CanvasKit은 기존 동작 유지 |

## 6. 커밋 계획

단계별 완료보고서와 함께 stage 단위로 커밋한다.

| Stage | 커밋 메시지 |
|-------|-------------|
| 1 | `Task #1918: Stage 1 - 정적 이미지 레이어 편집 지연 계측` |
| 2 | `Task #1918: Stage 2 - overlay summary 경량화` |
| 3 | `Task #1918: Stage 3 - text edit overlay canvas 재사용` |
| 4 | `Task #1918: Stage 4 - flow 정적 이미지 재렌더 정책 분리` |
| 5 | `Task #1918: Stage 5 - 성능 검증 및 보고` |
| 6 | `Task #1918: Stage 6 - WASM runtime 검증` |
| 7 | `Task #1918: Stage 7 - text edit 렌더 coalescing` |
| 8 | `Task #1918: Stage 8 - text edit fast path 안전성 보강` |

기능 변경과 문서/보고서 변경은 stage 단위 안에서만 함께 묶고, 무관한 포맷 변경은 포함하지 않는다.

## 7. Stage 7 후속 구현 계획

### 배경

Stage 6까지의 개선으로 정적 overlay와 flow-static 반복 렌더링은 줄었지만,
빠른 연속 키 입력에서는 여전히 `document-page-invalidated`가 입력 이벤트 안에서 즉시 동기 렌더링을 실행한다.

2026-07-05 추가 계측 결과:

| 샘플 | mutation only | mutation + page invalidation |
|------|---------------|------------------------------|
| `samples/복학원서.hwp` | 20회 평균 0.3ms/key | 20회 평균 63.2ms/key |
| `samples/253E164F57A1BC6934-empty.hwp` | 20회 평균 0.09ms/key | 20회 평균 15.7ms/key |
| `samples/143E433F503322BD33.hwp` | 20회 평균 0.08ms/key | 20회 평균 10.7ms/key |
| `samples/통합재정통계(2011.10월).hwp` | 20회 평균 0.35ms/key | 20회 평균 6.55ms/key |

즉 표 텍스트 mutation은 충분히 빠르며, 문제는 각 키 입력이 렌더 완료를 기다리는 구조다.

### 목표

- text-edit page-local invalidation을 즉시 렌더하지 않고 animation frame 단위로 합친다.
- 같은 페이지에 빠르게 들어온 text-edit invalidation은 중간 렌더를 생략하고 최신 상태만 그린다.
- text-edit에서 overlay summary가 변하지 않는 동안 `getPageOverlayImages`를 매 키마다 호출하지 않는다.

### 작업

1. `CanvasView.refreshInvalidatedPage`에 text-edit 전용 pending queue를 추가한다.
   - `requestAnimationFrame`으로 페이지별 invalidation을 coalesce한다.
   - non-text invalidation, full refresh, zoom/resize/reset에서는 pending 작업을 취소한다.
2. `PageRenderer`에 page/layer summary cache를 추가한다.
   - text-edit + static reuse 허용 시, 같은 page/scale/canvas size/render profile/backend key가 맞으면 이전 summary를 재사용한다.
   - full refresh, non-text refresh, overlay 제거, `resetImageRetryState`, `dispose`에서 캐시를 제거한다.
3. 정적 계약 테스트를 보강한다.
   - CanvasView가 `requestAnimationFrame` 기반 text-edit coalescing을 가진다.
   - non-text 경로는 즉시 refresh를 유지한다.
   - PageRenderer가 text-edit summary cache를 조건부로 사용하고 reset 경로에서 비운다.
4. 성능 probe를 재실행한다.
   - `복학원서` 20회 연속 입력에서 key handler 평균 시간이 크게 내려가는지 확인한다.
   - `통합재정통계` 일반 표 편집 무회귀를 확인한다.

### 가드레일

- 표 구조 변경, 문단 분할/병합, full `document-changed`는 coalescing 대상이 아니다.
- cursor/caret 갱신은 기존 입력 처리 후 바로 수행한다. 지연되는 것은 페이지 canvas 렌더뿐이다.
- zoom, resize, canvas release 이후 stale 렌더가 늦게 실행되지 않도록 pending queue를 취소한다.
- CanvasKit backend는 기존 경로를 유지한다.

## 8. Stage 8 안전성 보강 계획

### 배경

Stage 7 검증 후 작업지시자가 직접 `rhwp-studio`에서 워터마크 포함 문서의 빠른 표 입력 지연이
만족스럽게 개선되었음을 확인했다. 다만 PR 전 리뷰에서 다음 두 가지 잔여 리스크가 확인되었다.

1. 같은 셀 `insertText`/`deleteText`가 항상 페이지 로컬 변경이라는 보장은 없다.
   - 긴 단일 줄 paste, 페이지 경계 근처 표 셀, 자동 행 높이 변화가 큰 셀은 다른 페이지 배치를
     움직일 수 있다.
2. 정적 레이어 재사용은 flow 이미지/OLE/rawSvg 개수와 캔버스 크기 중심의 key를 사용한다.
   - 개체 개수는 같지만 bbox만 바뀌는 문서에서는 다음 full refresh 전까지 stale layer가 보일 수 있다.

### 목표

- 이번 PR 안에서 text-edit fast path의 직접 부작용 가능성을 줄이는 최소 안전장치를 추가한다.
- 성능 개선 효과를 유지하되, 레이아웃 변화 가능성이 큰 입력은 기존 full refresh로 되돌린다.
- PR 본문에 남길 수 있도록 처리 범위와 후속 일반화 범위를 명확히 분리한다.

### 작업

1. page-local 판정 가드 보강
   - 같은 셀 text edit이라도 단일 명령 텍스트가 줄바꿈/탭을 포함하거나 길이가 큰 경우 full refresh로 보낸다.
   - plain text paste fallback처럼 `InsertTextCommand`를 쓰는 경로가 과도하게 fast path를 타지 않도록 한다.
2. 정적 레이어 stale 방어
   - text-edit fast path에서 정적 레이어 재사용 후 idle/trailing full page refresh 또는 보수적 재검증을 적용할지 검토한다.
   - 성능 회귀가 작고 구현이 단순한 쪽을 우선한다.
3. 회귀 테스트 보강
   - page-local 허용 조건이 `insertText`/`deleteText`의 작은 같은 셀 편집으로 제한되는지 테스트한다.
   - paste/Enter/선택 삭제/구조 변경 경로가 기존 full refresh에 남는지 정적 계약 테스트를 추가한다.
4. 문서화
   - Stage 8 완료 보고서에 이번 PR에서 처리한 안전장치와 후속 이슈로 남길 일반화 항목을 분리 기록한다.
   - 최종 보고서와 오늘 할일을 갱신한다.

### 가드레일

- Stage 8은 안전성 마감 작업이므로 대규모 레이아웃 diff/signature 설계는 포함하지 않는다.
- 워터마크 문서의 빠른 연속 입력 성능 개선을 되돌리지 않는다.
- source 수정 전 작업지시자 승인을 받은 뒤 구현한다.

## 9. 승인 이력

- Stage 1 시작 승인: 2026-07-05
- Stage 2-6 진행 승인: 2026-07-05
- Stage 7 후속 개선 진행 승인: 2026-07-05
- Stage 8 안전성 보강 계획 수립: 2026-07-05
- Stage 8 안전성 보강 구현 진행 승인: 2026-07-05
