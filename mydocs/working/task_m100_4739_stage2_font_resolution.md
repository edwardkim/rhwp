# Task M100 #4739 Stage 2 - 첫 paint와 정확 face 해소

## 목적

저장된 local-font snapshot을 Canvas2D 첫 paint 전에 준비하고, KoPub style face와 정부상징
legacy/successor/document substitute를 provenance 순서대로 해소한다. 조판 메트릭 상수는 바꾸지 않는다.

## 구현

- `initializeDocument()`가 현재 문서의 non-embedded `substFont` 목록을 등록하고 저장 snapshot을
  `canvasView.loadDocument()`보다 먼저 읽는다. 이미 읽은 snapshot은 prompt 단계에서 다시 읽지 않는다.
- KoPub바탕·KoPub바탕체와 영문 KoPubBatang 변형을 Windows 고정폭 BatangChe보다 먼저 비례폭
  serif로 분류한다.
- local face가 family의 Light/Medium/Bold 같은 style이면 canonical family로 뭉개지 않고 full name을
  Canvas chain 첫 항목으로 사용한다.
- 정부상징 legacy 이름 두 개에 한정해 ROKG family/full/PostScript 이름을 successor로 찾는다.
  exact legacy → 확인된 ROKG → 문서 `substFont` → portable chain 순서를 유지한다.
- Rust style resolver가 non-embedded `substFont`를 내부 family chain에 보존하고 SVG/Canvas chain은
  exact legacy 뒤에 ROKG successor, 그 뒤에 문서 대체 face를 둔다.
- WASM `DocumentInfo.fontSubstitutions`가 source/substitute 쌍을 Studio에 전달한다. CanvasKit
  preflight는 내부 chain 전체를 하나의 family로 오인하지 않고 primary face만 수집한다.
- 문서 글꼴 상태 보고도 ROKG successor가 확인되면 legacy face를 local available로 판정해 불필요한
  재감지 prompt를 띄우지 않는다.

## 검증

- `node tests/font-substitution.test.ts`: 9/9 통과
- `node tests/local-fonts.test.ts`: 14/14 통과
- `node tests/document-font-status.test.ts`: 5/5 통과
- `npx tsc --noEmit`: 통과
- Rust `renderer::style_resolver::tests`: 29/29 통과
- Rust renderer chain, CanvasKit primary family, `DocumentInfo.fontSubstitutions` focused test: 각 1/1 통과
- `document-initialization-order.test.ts`: Stage 2 범위 5건 통과, Stage 3 repaint RED 1건은 의도대로
  계속 실패

## 다음 단계

Stage 3에서 `local-fonts-changed`를 backend별로 한 번의 `document-view-changed`에 연결한다. Canvas2D는
WASM 문서를 다시 열지 않고 즉시 repaint하며, CanvasKit은 local Typeface 준비가 끝난 뒤 같은 view
갱신 이벤트를 한 번만 발행한다.
