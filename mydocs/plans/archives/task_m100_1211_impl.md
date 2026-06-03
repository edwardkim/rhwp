# 구현계획서 — Task M100-1211: 입력 편집 narrow invalidation

## 설계 요약

현재 `InputHandler.afterEdit()`는 모든 편집 후 `document-changed`를 발행한다.
`CanvasView`는 이 이벤트를 full refresh로 해석해 page info 전체 재수집, 모든 visible canvas release, image retry state reset을 수행한다.

이번 작업은 일반 텍스트 입력처럼 변경 범위가 좁은 편집을 별도 invalidation 경로로 분리한다.
구조 변경 가능성이 큰 작업은 기존 `document-changed` full refresh 경로를 유지한다.

선택안은 후보 A + C이다. 이벤트 의미를 분리하는 A를 기본 축으로 삼고, 이 경로에서는 full refresh에 포함된 `resetImageRetryState()`를 호출하지 않아 C의 효과를 함께 얻는다. 후보 B처럼 `document-changed`에 reason을 추가하는 방식은 호출 계약이 계속 넓고 모호해질 수 있어 이번 PR의 기본안에서 제외한다. 후보 D의 flow image 전용 API는 WASM API 표면을 늘리므로 A+C 적용 후에도 병목이 남을 때 후속 이슈로 검토한다.

## Stage 1 — 기준 비용 확인과 이벤트 경계 정리

**목표**: 실제 입력 경로에서 어떤 이벤트와 렌더 호출이 반복되는지 코드/간단 계측으로 확인하고, 소스 변경 범위를 확정한다.

작업:

- `InputHandler.executeOperation()`의 `command` 경로와 `snapshot` 경로를 분리해 확인한다.
- `afterEdit()` 호출자가 어떤 작업인지 분류한다.
- `CanvasView.refreshPages()`가 수행하는 full refresh 항목을 확정한다.
- `PageRenderer.scheduleReRender()` / `prefetchFlowImages()`가 입력 중 반복 예약되는 조건을 확인한다.
- 필요한 경우 개발용 계측 로그를 임시로 사용하되, PR에는 남기지 않는다.

산출:

- `mydocs/working/task_m100_1211_stage1.md`
- 구현 범위 확정:
  - 일반 `InsertTextCommand` / `DeleteTextCommand` / IME 확정 입력을 narrow invalidation 대상으로 볼지
  - snapshot/paste는 Stage 1에서는 full refresh 유지할지

검증:

```text
cd rhwp-studio && npm test
```

## Stage 2 — `CanvasView` narrow page refresh 추가

**목표**: full `refreshPages()`와 별도로 현재 page만 재렌더하는 API를 추가한다.

변경 후보:

- `rhwp-studio/src/view/canvas-view.ts`
  - `document-page-invalidated` 이벤트 처리 추가.
  - 기존 canvas를 유지한 채 다시 그리는 `renderCanvas(pageIdx, canvas)` 분리.
  - page info 전체 재수집과 `recalcLayout()`은 full refresh에만 유지한다.
  - 단일 page refresh에서는 해당 page canvas와 overlay/grid만 갱신한다.
  - 단순 텍스트 입력 경로에서는 `resetImageRetryState()` 전체 초기화를 피한다.

- `rhwp-studio/src/view/page-renderer.ts`
  - 특정 page retry state만 유지/취소할 수 있는 메서드가 필요한지 점검한다.
  - 기존 `cancelReRender(pageIdx)`는 유지하되, full reset과 page-local reset을 분리할 수 있는지 확인한다.

테스트:

- `CanvasView`를 직접 테스트하기 어렵다면 `PageRenderer` 또는 event routing 단위 테스트를 우선 추가한다.
- DOM 의존성이 커서 단위 테스트가 과도하면, 최소한 TypeScript 빌드와 수동 검증으로 제한하고 보고서에 남긴다.

보고서:

- `mydocs/working/task_m100_1211_stage2.md`

## Stage 3 — 입력 편집 경로를 narrow invalidation으로 연결

**목표**: 일반 텍스트 입력 후 full `document-changed` 대신 좁은 invalidation을 사용한다.

변경 후보:

- `rhwp-studio/src/engine/input-handler.ts`
  - `afterEdit()`는 기본 full refresh로 유지한다.
  - 새 메서드 `afterPageLocalEdit()`를 추가한다.
  - `executeOperation({ kind: 'command' })` 중 셀 내부 `insertText` / `deleteText`처럼 범위가 좁은 명령은 새 메서드를 호출한다.
  - snapshot/paste/object/table/page 설정 등은 기존 `afterEdit()` 유지.

- `rhwp-studio/src/engine/input-edit-invalidation.ts`
  - page-local text edit 판정 helper를 순수 함수로 분리한다.
  - 단위 테스트에서 narrow/full refresh 분기 조건을 검증한다.

- 이벤트 이름 후보:

```text
document-page-invalidated
payload: { pageIndex?: number; reason: 'text-edit' | ... }
```

pageIndex는 현재 cursor rect 또는 command 실행 후 cursor rect에서 얻는다.
pageIndex를 얻지 못하면 안전하게 기존 `document-changed` full refresh로 fallback한다.

주의:

- 문단 분할/병합, 표 행 높이 변화, 페이지 넘어감 가능성이 큰 명령은 Stage 3 기본 범위에서 full refresh로 유지한다.
- 단일 텍스트 삽입/삭제도 페이지 흐름을 바꿀 수 있으므로, affected page 단일 갱신이 충분하지 않은지 Stage 1/수동 검증에서 확인한다.
- 이번 1차 적용은 표 셀 내부 동일 cellPath의 insert/delete에 한정한다. 본문 문단 입력은 같은 텍스트 명령이라도 page flow 변동 가능성이 커서 full refresh를 유지한다.

보고서:

- `mydocs/working/task_m100_1211_stage3.md`

## Stage 4 — flow image prefetch 비용 후속 보정 여부 판단

**목표**: Stage 2~3 적용 후에도 이미지 페이지 입력 지연이 남는지 확인하고, `prefetchFlowImages()`의 전체 `getPageLayerTree()` 호출을 다룰지 결정한다.

선택지:

1. 지연이 충분히 개선되면 Stage 4는 조사 보고만 하고 구현 생략.
2. 여전히 큰 비용이면 작은 API를 추가한다.

API 후보:

```text
getPageFlowImages(pageIdx) -> { imageCount, dataUrls? }
```

또는 `getPageOverlayImages` 응답 확장은 기존 의미가 흐려질 수 있으므로 우선 별도 API를 선호한다.

보고서:

- `mydocs/working/task_m100_1211_stage4.md`

## Stage 5 — 최종 검증과 보고

검증:

```text
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
```

필요 시:

```text
wasm-pack build --target web --out-dir pkg
cargo test --lib
```

수동 확인:

```text
samples/exam_social.hwp
1쪽 성명 입력칸 연속 입력
이미지/오버레이 표시 유지
caret 위치 유지
```

최종 보고:

- `mydocs/report/task_m100_1211_report.md`

## 제외 범위

- 모든 `document-changed` 발행 지점을 한 PR에서 전면 교체하지 않는다.
- 페이지네이션/레이아웃 엔진 자체 최적화는 이번 범위가 아니다.
- #1207의 중첩 표 붙여넣기 path correctness는 이 PR에 포함하지 않는다.

## 승인 요청

위 구현계획에 따라 Stage 1을 시작한다.
