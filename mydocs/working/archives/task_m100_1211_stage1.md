# Task M100-1211 Stage 1 완료 보고 — 입력 편집 렌더 비용 원인 정리

## 범위

`samples/exam_social.hwp` 1쪽 상단 `성명` 입력칸에서 텍스트 입력이 느리게 반영되는 원인을 코드 기준으로 재확인했다.

## 확인 내용

- #1207의 중첩 표 붙여넣기 수정은 텍스트 입력마다 새 render 호출을 추가하지 않는다.
- 입력 지연의 직접 경로는 기존 `InputHandler.afterEdit() -> document-changed -> CanvasView.refreshPages()`이다.
- `refreshPages()`는 page info 재수집, layout 재계산, visible canvas release, `resetImageRetryState()`, visible page 재렌더를 매 입력마다 수행한다.
- #865의 `getPageOverlayImages`는 overlay 이미지 목록 추출 시 대형 `PageLayerTree` JSON 왕복을 줄이는 최적화다. 그러나 full `document-changed` 자체가 반복되는 비용, 그리고 full refresh에 딸린 image retry reset 비용은 별도로 남아 있다.

## 선택안

후보 A + C를 선택한다.

- A: 텍스트 입력용 narrow invalidation 이벤트를 추가해 `document-changed`의 구조 변경 의미와 분리한다.
- C: narrow invalidation 경로에서는 `refreshPages()`를 호출하지 않으므로 `resetImageRetryState()`도 매 입력마다 반복하지 않는다.

후보 B는 `document-changed`에 reason을 붙여 내부 분기하는 방식이라 기존 이벤트 의미가 계속 넓어질 위험이 있다. 후보 D는 새 WASM API를 늘리는 작업이므로 A+C 이후에도 flow image prefetch 비용이 남을 때 후속 이슈로 분리한다.

## 다음 단계

`CanvasView`에 page-local refresh 경로를 추가하고, 입력 라우터는 보수적으로 셀 내부 단일 insert/delete 텍스트 편집만 새 경로로 연결한다.
