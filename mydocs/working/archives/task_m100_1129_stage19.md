# Task #1129 Stage 19 - 쪽 테두리 기준 렌더링 재정합

## 배경

Stage 18에서는 한컴오피스 대화상자 기준 표시만 `PageBorderUiBasis`로 분리했다. 그러나 작업지시자 시각 비교 결과, `hwp3-sample16-hwp5.hwp`는 rhwp-studio 대화상자에서 쪽 기준으로 표시되지만 실제 외곽선은 종이 기준처럼 렌더링되고 있다.

작업지시자 기준:

- 쪽 기준: 제공 스크린샷 3처럼 외곽선이 본문/쪽 기준 쪽으로 들어온다.
- 종이 기준: 제공 스크린샷 4처럼 외곽선이 종이 가장자리 기준으로 그려진다.

## 원인

Stage 18 구현은 `ui_basis`만 바꾸고, 렌더러가 참조하는 `PageBorderFill::basis`는 HWP5/HWPX에서 계속 `PaperBased`로 유지했다. 따라서 UI는 쪽 기준으로 표시되지만 `layout.rs::build_page_borders()`는 종이 기준 외곽선을 그린다.

## 구현 계획

1. HWP5/HWPX 파서에서 `ui_basis`와 렌더링 `basis`를 같은 기준으로 맞춘다.
   - `attr bit0=0`, `textBorder=CONTENT` → UI 종이 기준, 렌더 `PaperBased`
   - `attr bit0=1`, `textBorder=PAPER` → UI 쪽 기준, 렌더 `BodyBased`
2. `setPageBorderFill`에서 사용자가 종이/쪽 기준을 바꾸면 렌더 `basis`도 함께 변경한다.
3. `getPageInfo`의 `pageBorder*` 계산도 같은 기준을 반영한다.
4. 기존 Stage 18 기준 샘플 테스트에 렌더 basis 차이를 추가한다.

## 검증 계획

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`
- `cargo test test_parse_page_border_fill --lib`
- `cargo test test_parse_page_border_fill_basis_from_text_border --lib`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- 로컬 Playwright 기능 검증
  - `samples/종이기준.hwp` → UI `paper`, `pageBorderTop` = 5.0mm 계열
  - `samples/쪽기준.hwp` → UI `page`, `pageBorderTop` = 쪽 기준 계열
  - `samples/hwp3-sample16-hwp5.hwp` → UI `page`, 쪽 기준 계열
- `cargo test --lib`
- `cargo fmt --all -- --check && git diff --check`

## 구현 결과

- HWP5 `PAGE_BORDER_FILL.attr bit0=1` 파싱 시 `ui_basis=Page`, `basis=BodyBased`로 맞췄다.
- HWPX `textBorder="PAPER"` 파싱 시 `ui_basis=Page`, `basis=BodyBased`로 맞췄다.
- `setPageBorderFill`에서 사용자가 `쪽 기준`을 선택하면 렌더링 기준도 `BodyBased`로 함께 바뀌게 했다.
- 기준 샘플 테스트에서 `쪽기준.hwp`의 `pageBorderTop`이 `종이기준.hwp`보다 안쪽으로 들어오는지 확인하도록 보강했다.

## 검증 결과

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `cargo test test_parse_page_border_fill --lib`: 통과
- `cargo test test_parse_page_border_fill_basis_from_text_border --lib`: 통과
- `cargo test page_border_fill_api_updates_basis_spacing_and_border --lib`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build`: 통과
- 로컬 Playwright 기능 검증: 통과
  - `samples/종이기준.hwp`: `basis=paper`, `pageBorderTop=18.9`
  - `samples/쪽기준.hwp`: `basis=page`, `pageBorderTop=113.4`
  - `samples/hwp3-sample16-hwp5.hwp`: `basis=page`, `pageBorderTop=56.7`
