# Task M100 #1139 Stage 20

## 목적

Stage 19 커밋 이후 남은 `3-09월_교육_통합_2022.hwp` 15쪽 하단 overflow와 17~18쪽 새 페이지 헤더 오버랩 문제를 한컴 정답지 기준으로 보정한다. 추가로, 같은 문서의 미주/수식 영역에서 드래그 선택과 `문단 모양`, `개체 속성`이 본문처럼 동작하지 않는 근본 원인을 해결한다.

## 시작 기준

- Stage 19에서 12쪽 하단과 13쪽 시작부의 미주 시작 vpos 정합을 보정했다.
- 작업지시자 WASM 시각 검증으로 Stage 19 문제는 해결 처리했다.
- 새로 확인된 문제는 15쪽 하단에서 내용이 페이지/단 경계를 자연스럽게 넘지 못하고 overflow 되는 현상이다.
- 추가 확인된 17~18쪽 문제는 음수 `start_height`가 새 페이지 렌더 시작점에 그대로 적용되어 머리말 영역을 침범하는 현상이다.
- 16쪽 미주 영역의 “서로 자리를 바꾸는 경우의 수” 등 수식 포함 문단은 드래그 선택 자체가 되지 않고, 우클릭 `문단 모양`을 눌러도 해당 미주 문단 속성 팝업이 열리지 않는다.
- 각 수식도 본문 수식처럼 개별 `개체 속성`을 확인할 수 있어야 하나, 현재 미주 내부 수식은 개체 선택/속성 조회 경로가 본문 문단 기준으로만 동작한다.

## 작업지시자 기준

- 15쪽 하단이 한컴 정답지처럼 다음 쪽/단으로 자연스럽게 이어져야 한다.
- 17~18쪽은 이전 쪽 말미가 다음 쪽 머리말/본문과 겹치지 않아야 한다.
- 미주 내부 텍스트도 드래그로 선택되고, 선택/커서 위치의 `문단 모양`이 정확히 열려야 한다.
- 미주 내부의 각 수식도 클릭/우클릭 후 `개체 속성`을 열 수 있어야 한다.
- SVG/PNG 산출물이 생기면 PNG 경로를 우선 보고한다.

## 분석 계획

1. `dump-pages`로 15쪽과 다음 쪽의 미주 문단/그림 배치 목록을 확인한다.
2. `RHWP_VPOS_DEBUG=1 export-svg`로 하단 overflow가 발생하는 문단과 vpos 보정 여부를 확인한다.
3. Stage 19 보정과 같은 compact endnote flow 특수 조건인지, 별도 그림/수식 split 문제인지 분리한다.
4. 음수 `start_height`는 글자처럼 취급 그림만 있는 빈 문단으로 시작하는 미주 단에만 시각 보정으로 적용하고, 일반 텍스트로 시작하는 새 페이지 미주 단은 본문 영역 위로 올리지 않는다.
5. 같은 판정을 페이지네이션에도 적용한다. 일반 텍스트 미주로 새 단/쪽이 시작하면 음수 논리 높이를 0으로 정규화해, 렌더러와 페이지네이터의 가용 높이 판단을 맞춘다.
6. rhwp-studio의 미주 커서 모드가 드래그 선택, 문단모양 조회/적용, 컨텍스트 메뉴에 연결되는지 확인한다.
7. `getPageControlLayout`과 개체 hit-test가 미주 내부 수식의 원본 위치를 식별하는지 확인하고, 빠진 경우 미주 문단/컨트롤 경로를 노출한다.

## 현재까지 반영된 수정

- `src/renderer/height_cursor.rs`, `src/renderer/layout.rs`, `src/renderer/typeset.rs`에서 미주/각주 흐름의 음수 `start_height`와 페이지 이월 조건을 추가 보정했다.
- 새 페이지 또는 새 단이 일반 텍스트 미주로 시작하는 경우, 이전 단계의 그림 전용 backtrack 보정이 머리말 영역까지 침범하지 않도록 제한하는 방향으로 조정했다.
- `src/document_core/commands/footnote_ops.rs`에서 `Footnote`뿐 아니라 `Endnote` 내부 문단도 문단 속성 조회/적용 대상이 되도록 확장했다.
- `src/document_core/queries/cursor_rect.rs`에 미주/각주 내부 선택 영역의 줄별 selection rect 계산 API를 추가했다.
- `src/wasm_api.rs`에 미주/각주 내부 문단 속성 조회/적용 및 selection rect 조회 API를 노출했다.
- `rhwp-studio/src/core/wasm-bridge.ts`에 위 WASM API 래퍼를 추가했다.
- `rhwp-studio/src/engine/cursor.ts`에 미주/각주 내부 선택 anchor와 정렬된 선택 범위 상태를 추가했다.
- `rhwp-studio/src/engine/input-handler.ts`에서 미주/각주 내부 드래그 선택, selection 렌더링, 문단 모양 조회/적용 경로를 본문/표 셀과 분리해 처리하도록 연결했다.
- `rhwp-studio/src/engine/input-handler-mouse.ts`에서 미주/각주 영역 클릭 시 커서 이동만 하고 끝나지 않고, 선택 드래그 시작 anchor까지 설정하도록 보정했다.
- `examples/diag_1139_para_shape.rs`를 추가해 문제 문단의 문단 모양/간격 확인을 위한 진단 경로를 준비했다.

## 남은 항목

- 현재 변경은 아직 `cargo fmt`, `cargo build`, WASM 빌드, PNG 시각 검증 전 상태이다.
- 미주/각주 내부 수식의 `개체 속성` 경로는 분석 중이다. 본문 수식/그림처럼 hit-test가 원본 컨트롤 경로를 안정적으로 돌려주도록 `getPageControlLayout`과 스튜디오 개체 선택 경로를 추가 확인해야 한다.
- 15쪽 하단 overflow, 17~18쪽 헤더 오버랩, 9쪽/12쪽 회귀 여부는 빌드 후 다시 PNG 기준으로 확인해야 한다.

## 검증 대기

- `cargo fmt`
- `cargo build`
- 15쪽/16쪽/17쪽/18쪽 PNG 시각 비교
- 필요 시 `wasm-pack build --target web --out-dir pkg`
