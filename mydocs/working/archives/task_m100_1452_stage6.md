# Task M100 #1452 Stage 6 시작 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `1698d69e task 1452: 외부 그림 드롭 한컴 동작 정합 보정`

## 1. 배경

`samples/투명도0-50.hwp`를 rhwp-studio에서 열면 두 번째 그림(투명도 50)이 한컴과 다르게 표시된다.
한컴 기준으로는 첫 번째 그림은 투명도 0, 두 번째 그림은 투명도 50으로 더 옅게 합성된다.

같은 샘플에서 문단표시도 한컴보다 흐릿하게 보여, 화면에서 더 또렷하게 식별되도록 보정이 필요하다.

Stage 4에서 파일 파싱/속성 roundtrip과 ImageNode opacity 전달은 구현했지만, 실제 렌더링 결과가
한컴 시각 결과와 아직 맞지 않는지 확인해야 한다.

## 2. 작업 범위

- `samples/투명도0-50.hwp`와 `samples/투명도0-50.hwpx`의 그림 투명도 파싱값을 재확인한다.
- RenderTree/ImageNode에 전달되는 `opacity`, PNG 자체 alpha, 밝기/대비/효과 값을 비교한다.
- Canvas/SVG/Skia 렌더러 중 rhwp-studio 표시 경로에서 전체 투명도와 PNG alpha가 한컴처럼 합성되는지
  확인한다.
- 한컴 기준과 다른 합성 경로가 확인되면 Stage 6에서 렌더러 또는 모델 변환을 보정한다.
- 문단부호(`↵`, 강제 줄바꿈 표시 등)가 한컴처럼 또렷하게 보이도록 색상/크기/렌더링 경로를 점검한다.

## 3. 초기 확인

- 샘플 파일 위치:
  - `samples/투명도0-50.hwp`
  - `samples/투명도0-50.hwpx`
- 기존 테스트 `issue1452_picture_transparency_samples_parse_as_ui_percent`는 두 샘플의 첫 두 그림
  투명도를 `[0, 50]`으로 확인한다.
- 현재 `ImageAttr::opacity()`는 `1.0 - transparency / 100.0`으로 계산한다.
- rhwp-studio 표시 경로는 ImageNode `opacity`와 이미지 자체 alpha가 최종 화면에서 어떻게 곱해지는지
  추가 검증이 필요하다.
- WebCanvas/SVG 문단부호는 현재 `#4A90D9` 계열 색으로 TextRun 렌더링 시 함께 그린다.
- RenderTree 확인 결과 HWP 샘플에서 실제 그림은 2개인데 ImageNode가 3개 생성되는 상태였다.
  - `FullParagraph` 경로가 빈 문단의 TAC 그림 2개를 그린 뒤, `PageItem::Shape` fallback이 두 번째
    그림을 한 번 더 그려 투명도 50 이미지가 한컴보다 진하게 합성됐다.
  - 중복 fallback을 막으면 이번에는 두 줄 모두 첫 번째 picture control로 매핑되는 문제가 드러났다.
    원인은 텍스트 없는 HWP 문단에서 여러 `LINE_SEG`가 같은 `text_start`를 갖는데, 줄별 TAC 필터가
    두 줄 모두 첫 번째 TAC만 선택한 것이다.

## 4. 검증 계획

- `cargo test --lib issue1452 -- --nocapture`
- 샘플 RenderTree dump에서 두 ImageNode의 opacity/brightness/contrast/effect 확인
- 필요 시 SVG/Canvas 경로의 샘플 렌더를 한컴 스크린샷과 수동 비교
- 문단부호 색상/크기 변경 시 스크린샷 기준으로 가독성을 수동 확인
- 수정 후 `wasm-pack build --target web --out-dir pkg`

## 5. 구현 내용

- `src/renderer/layout.rs`
  - `FullParagraph`가 있는 문단의 TAC 그림은 `paragraph_layout`이 줄 안에 배치하도록 하고,
    `layout_shape_item`의 빈 문단 fallback은 중복 ImageNode를 만들지 않도록 제한했다.
- `src/renderer/layout/paragraph_layout.rs`
  - 같은 `char_start`를 공유하는 반복 빈 TAC 줄에서 TAC 위치가 `start..start+줄수` 범위로 순차
    배치된 경우, 줄 순번에 맞는 TAC 하나만 선택하도록 보정했다.
  - `투명도0-50.hwp`의 두 줄은 각각 picture control 2/3으로 렌더되고, opacity는 1.0/0.5로
    유지된다.
- `src/renderer/web_canvas.rs`, `src/renderer/svg.rs`, `src/renderer/html.rs`,
  `src/renderer/skia/text_replay.rs`
  - 문단표시 색상을 기존 `#4A90D9`에서 더 또렷한 `#0066FF`로 통일했다.
- `src/document_core/commands/object_ops.rs`
  - `samples/투명도0-50.hwp`/`.hwpx`의 렌더 트리가 ImageNode 2개만 만들고 opacity 100/50을
    보존하는 회귀 테스트를 추가했다.

## 6. 검증 결과

- `cargo fmt --check` 통과
- `cargo test --lib issue1452 -- --nocapture` 통과
  - 9 passed
- `git diff --check` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `cargo run --quiet --bin rhwp -- export-svg samples/투명도0-50.hwp -p 0 --show-para-marks -o output/poc/task1452_stage6_svg_after`
  - SVG `<image>` 2개 확인
  - 두 번째 그림 `opacity="0.500"` 확인
  - 문단표시 `#0066FF` 확인
