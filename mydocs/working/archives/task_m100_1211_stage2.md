# Task M100-1211 Stage 2 완료 보고 — CanvasView page-local refresh

## 변경 내용

- `CanvasView`가 `document-page-invalidated` 이벤트를 구독하도록 추가했다.
- 기존 `renderPage()`의 실제 렌더 로직을 `renderCanvas(pageIdx, canvas)`로 분리했다.
- narrow invalidation에서는 page info 전체 재수집, `recalcLayout()`, `releaseAllRenderedPages()`, `resetImageRetryState()`를 수행하지 않고 기존 canvas를 유지한 채 해당 page만 다시 그린다.
- page index가 유효하지 않거나 page count가 달라진 경우에는 기존 `refreshPages()` full refresh로 fallback한다.

## 보수 장치

- 렌더 실패 시 해당 canvas를 release하고 visible page 갱신으로 fallback한다.
- 페이지 수가 바뀌는 경우 narrow 경로를 사용하지 않는다.
- overlay/grid는 `PageRenderer.applyOverlays()`와 `renderGridOverlay()`가 기존 page 단위 제거/갱신 로직을 재사용한다.

## 다음 단계

입력 명령 라우터에서 셀 내부 단일 텍스트 insert/delete만 `document-page-invalidated`로 연결한다.
