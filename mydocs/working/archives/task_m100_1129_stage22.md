# Task m100 #1129 Stage 22

## 문제

Stage 21에서 `hwp3-sample16-hwp5.hwp`의 상단 로고와 외곽선 충돌을 피하려고 `attr bit0=1` / `textBorder=PAPER`의 렌더 기준을 `PaperBased`로 되돌렸다.

그러나 작업지시자 시각 확인 결과, 파일 최초 로드 시 대화상자 기준이 `쪽 기준`인데 실제 렌더가 `종이 기준`처럼 표시되는 회귀가 발생했다.

## 기준

한컴오피스 도움말 기준:

- `종이 기준`: 편집 용지 가장자리에서 안쪽 방향으로 쪽 테두리를 배치
- `쪽 기준`: 본문 영역 가장자리에서 바깥쪽 방향으로 쪽 테두리를 배치

따라서 HWP5/HWPX 저장값이 한컴 UI에서 `쪽 기준`으로 표시되는 경우, 렌더 기준도 `BodyBased`여야 한다.

## 수정 계획

- HWP5 `PAGE_BORDER_FILL.attr bit0=1` → UI `쪽 기준`, 렌더 `BodyBased`로 복구
- HWPX `pageBorderFill@textBorder="PAPER"` → UI `쪽 기준`, 렌더 `BodyBased`로 복구
- Stage 21에서 바꾼 테스트 기대값을 `쪽 기준` 렌더 기준에 맞게 되돌림
- 주석에 Stage 22 결정을 남김

## 검증 계획

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`
- `cargo test test_parse_page_border_fill --lib`
- `cargo test test_parse_page_border_fill_basis_from_text_border --lib`
- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp -p 0 -o output/poc/task1129_stage22_verify`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `cargo fmt --all -- --check`
- `git diff --check`

## 검증 결과

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `cargo test test_parse_page_border_fill --lib`: 통과
- `cargo test test_parse_page_border_fill_basis_from_text_border --lib`: 통과
- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp -p 0 -o output/poc/task1129_stage22_verify`: 통과
  - `PAGE_BORDER: attr=0x00000001 bit0=1 ... paper_based=false`
- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build` (`rhwp-studio`): 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
