# Task m100 #1129 Stage 21

## 문제

`hwp3-sample16-hwp5.hwp`는 한컴오피스 쪽 테두리/배경 대화상자에서 위치가 `쪽 기준`으로 표시된다. 그러나 첫 페이지 로고 객체의 좌표와 크기는 한컴 UI 값과 동일하게 파싱되고 있으므로, 로고가 외곽선에 너무 붙어 보이는 직접 원인은 그림 좌표가 아니라 외곽선 렌더 기준이다.

Stage 19에서 `attr bit0=1` / `textBorder=PAPER`을 UI `쪽 기준`과 렌더 `BodyBased`로 함께 해석했다. 그 결과 외곽선이 본문 영역 기준으로 안쪽에 그려져 첫 페이지 로고와 충돌했다.

## 결정

- 한컴 UI 표시 기준과 실제 렌더 기준을 다시 분리한다.
- `attr bit0=1` / `textBorder=PAPER`
  - UI: `쪽 기준`
  - 렌더: `PaperBased`
- `attr bit0=0` / `textBorder=CONTENT`
  - UI: `종이 기준`
  - 렌더: `PaperBased`

이 결정은 `hwp3-sample16-hwp5.hwp`의 상단 로고가 저장 좌표 그대로 렌더될 때 외곽선과 겹치지 않아야 한다는 실제 샘플 비교를 우선한다.

## 수정 계획

- HWP5 `PAGE_BORDER_FILL` 파서에서 UI 기준과 렌더 기준 분리 복구
- HWPX `pageBorderFill@textBorder` 파서도 동일하게 복구
- 관련 단위 테스트 기대값 수정
- 렌더 기준 주석을 Stage 21 결정으로 보완

## 검증 계획

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`
- `cargo test test_parse_page_border_fill --lib`
- `cargo test test_parse_page_border_fill_basis_from_text_border --lib`
- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp -p 0 -o output/poc/task1129_stage21_verify`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `cargo fmt --all -- --check`
- `git diff --check`

## 검증 결과

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `cargo test test_parse_page_border_fill --lib`: 통과
- `cargo test test_parse_page_border_fill_basis_from_text_border --lib`: 통과
- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp -p 0 -o output/poc/task1129_stage21_verify`: 통과
  - `PAGE_BORDER: attr=0x00000001 bit0=1 ... paper_based=true`
  - SVG 상단 외곽선 y 좌표: 약 `18.9px`
  - 첫 페이지 로고 y 좌표: 약 `56.2px`
- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build` (`rhwp-studio`): 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
