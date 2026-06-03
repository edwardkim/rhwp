# Task M100-1211 최종 보고 — rhwp-studio 입력 편집 재렌더 비용 축소

## 요약

`rhwp-studio`의 셀 내부 텍스트 입력이 매 키 입력마다 `document-changed -> refreshPages()` full refresh를 타면서 visible page 전체 재렌더와 `resetImageRetryState()`를 반복하던 비용을 줄였다.

## 선택안과 근거

선택안은 후보 A + C이다.

- 후보 A: 텍스트 입력 전용 `document-page-invalidated` 이벤트를 추가해 구조 변경용 `document-changed`와 의미를 분리했다.
- 후보 C: 이 narrow invalidation 경로에서는 `refreshPages()`를 호출하지 않으므로 `resetImageRetryState()`도 매 입력마다 반복하지 않는다.

이 선택은 #865의 `getPageOverlayImages`와 별개다. #865는 overlay 이미지 목록 추출 시 대형 `PageLayerTree` JSON 왕복을 줄였고, 이번 작업은 그 이후에도 남아 있던 full `document-changed` refresh 비용을 줄인다.

후보 B는 `document-changed`에 reason을 붙이는 방식이라 이벤트 계약이 계속 모호해질 수 있어 제외했다. 후보 D의 flow image 전용 API는 WASM API 표면을 늘리므로, A+C 적용 후에도 병목이 남는 경우 후속 이슈로 다루는 것이 낫다.

## 구현

- `rhwp-studio/src/view/canvas-view.ts`
  - `document-page-invalidated` 이벤트 처리 추가.
  - 기존 canvas를 유지한 채 해당 page만 다시 그리는 `renderCanvas()` 분리.
  - narrow 경로에서는 page info 재수집, layout 재계산, 전체 canvas release, image retry reset을 생략.
  - page count 변동 또는 유효하지 않은 page index는 기존 full refresh로 fallback.

- `rhwp-studio/src/engine/input-handler.ts`
  - command 실행 전/후 위치를 비교해 셀 내부 단일 text insert/delete만 page-local 경로로 보냄.
  - command를 거치지 않는 IME/iOS raw 텍스트 입력용 `afterTextInputEdit()` 라우터 추가.
  - header/footer, footnote, snapshot/paste, 문단/표/객체 구조 변경은 기존 `afterEdit()` full refresh 유지.

- `rhwp-studio/src/engine/input-handler-text.ts`
  - 한글 IME 조합 중 `insertTextAtRaw()` / `deleteTextAt()` 후에도 page-local 라우터를 사용.
  - iOS composition fallback의 debounce 렌더도 같은 라우터를 사용.

- `rhwp-studio/src/engine/input-edit-invalidation.ts`
  - page-local text edit 판정 helper를 순수 함수로 분리.

- `rhwp-studio/tests/input-edit-invalidation.test.ts`
  - 셀 내부 insert/delete 허용, 본문/구조 변경/full refresh fallback 조건 검증.

## 검증

```text
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
```

모두 통과.

수동 확인:

- `http://127.0.0.1:7700/?url=/samples/exam_social.hwp&filename=exam_social.hwp`
- 1쪽 상단 `성명` 칸에 `테스트` 입력.
- 입력 반영 확인, 브라우저 console error 없음.
- Stage 5 보완 후 한글 입력 경로에서도 browser console error 없음.

## 잔여 범위

본문 문단 입력까지 narrow invalidation을 확장하는 일은 이번 PR에서 제외한다. 본문 입력은 페이지 flow 변동 가능성이 크므로, affected page 범위 계산이나 layout invalidation 정책을 더 분명히 한 후 별도 개선으로 진행하는 것이 안전하다.
