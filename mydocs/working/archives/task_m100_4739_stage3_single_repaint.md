# Task M100 #4739 Stage 3 - local font 단일 repaint

## 목적

새 local-font snapshot을 문서 재파싱이나 전체 재로드 없이 현재 backend에 한 번 반영한다. 같은
snapshot 세대가 중복 통지되더라도 반복 repaint하지 않는다.

## 구현

- 현재 문서의 `fontsUsed`를 초기화 시점에 보존하고, `local-fonts-changed`를 Studio의 view 갱신
  경로에 연결했다.
- Canvas2D는 `document-view-changed` 한 번만 발행해 visible view를 다시 그린다.
- CanvasKit은 현재 문서의 local SFNT face 준비가 끝난 뒤 기존 helper가 같은 view 갱신 이벤트를
  한 번 발행한다.
- snapshot의 `detectedAt`, `source`, `count`로 세대 토큰을 만들고 같은 세대의 중복 이벤트를
  무시한다.
- 두 backend 모두 `canvasView.loadDocument()`를 호출하지 않으므로 WASM 재파싱·재페이지네이션을
  일으키지 않는다.

## 검증

- `node tests/document-initialization-order.test.ts`: 6/6 통과
- `node tests/font-substitution.test.ts`: 9/9 통과
- `node tests/local-fonts.test.ts`: 14/14 통과
- `npx tsc --noEmit`: 통과

## 다음 단계

Stage 5 광범위 검증에서 Studio 전체 테스트와 빌드, 실제 Chrome CDP의 Canvas2D·CanvasKit 동작을
확인한다. Stage 4 layout metric A/B와 전역 metric 변경은 별도 승인 전까지 수행하지 않는다.
