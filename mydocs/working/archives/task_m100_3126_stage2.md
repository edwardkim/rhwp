# Task M100 #3126 Stage 2 — 명시적 print profile과 embedded font 회귀

- 작성일: 2026-07-23
- 판정: 구현·Rust/WASM 검증 완료

## 1. profiled SVG API

- WASM `HwpDocument`와 `HwpViewer`에 `renderPageSvgWithProfile(page, profile)`을 추가했다.
- TypeScript bridge는 기존 `LayerRenderProfile` 문자열 계약을 사용한다.
- 기존 `renderPageSvg(page)` 동작은 바꾸지 않았다.
- Studio print pipeline만 `profile = "print"`를 명시한다.

## 2. editor-only 출력 억제

print/high-quality profile에서는 다음 interactive 표시를 전역 상태 변경 없이 끈다.

- 문단 부호
- 조판 부호
- 투명 테두리
- debug overlay
- 편집 화면용 missing-picture placeholder

투명 테두리 노드는 `RenderNode.editor_only`로 명시하고, `LayerBuilder`가 non-interactive
profile에서 제외한다. screen profile과 DocumentCore의 현재 표시 상태는 그대로 유지된다.

## 3. #2524 회귀 발견과 보강

초기 구현은 print profile을 layer-SVG로 직접 렌더했다. 이 경로는 `GlyphOutline` sidecar를
SVG leaf로 직접 재생하지 않고 `TextRun` fallback을 사용하므로, 문서 내장 bitmap/SVG font를
브라우저가 찾지 못하면 #2524의 두부(□) 문제가 재발할 수 있었다.

해결은 renderer backend를 새로 만들지 않고 #2524의 기존 원칙을 print profile에도 연결했다.

1. profile SVG 렌더 중 실제 사용 font/codepoint를 수집한다.
2. 문서 BinData의 embedded face bytes를 bounded loader로 읽는다.
3. 실제 사용된 embedded face만 원본 전체 data-URI `@font-face`로 삽입한다.
4. 비내장 폰트는 기존 SVG `font-family` fallback을 유지한다.

시스템 font file 탐색이나 subsetter를 호출하지 않으므로 WASM에서도 같은 경로를 사용할 수 있다.

## 4. 검증

| 검증 | 결과 |
|---|---|
| #2524 embedded font 기존 3개 test | 통과 |
| 신규 profiled print SVG data-URI 회귀 | 통과 |
| print profile interactive option 억제·상태 불변 | 통과 |
| editor-only node screen 유지/print 제외 | 통과 |
| #2525 단일-lineseg 재래핑 | 통과 |
| `cargo check --lib` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과 |

`pkg/`는 재현 가능한 ignored 산출물이며 커밋 대상이 아니다.
