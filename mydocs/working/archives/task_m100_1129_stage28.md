# Task 1129 Stage 28 - 쪽 기준 페이지 테두리 세로 기준 재보정

## 사용자 지적

- 외곽선이 한컴오피스와 다른 위치에 그려진다.
- `hwp3-sample16-hwp5.hwp`에서 쪽 기준인데 테두리 상단이 너무 위로 올라가 보인다.
- 추가 디버그 기준: 첨부 스샷은 `종이 기준 + 3mm` 격자 단위이다.

## 확인 내용

- `samples/hwp3-sample16-hwp5.hwp`의 첫 페이지 설정:
  - 용지: 59528 x 84188 HWPUNIT
  - 여백: 좌/우 4252, 상/하 2836
  - 머리말/꼬리말 여백: 2836
  - 쪽 테두리 간격: 1420 HWPUNIT
  - UI 기준: 쪽 기준
- `BodyBased` 세로 기준을 `body_area`로 바꾸는 가설을 검증했으나,
  sample16 첫 로고가 외곽선에 더 가까워져 사용자 지적과 반대 방향임을 확인했다.
- `export-svg --show-grid=3mm`는 현재 회색 선 격자를 삽입하므로, 한컴오피스의
  점 격자/종이 기준 비교용 디버그에는 부적합하다.

## 수정 계획

- 실제 페이지 테두리 좌표는 현 단계에서 추가 이동하지 않는다.
- `export-svg --show-grid=Nmm`를 한컴오피스 디버그 기준에 맞춰 종이 원점의 점 격자로 변경한다.
- 기존 HWP5/HWPX 쪽/종이 기준 샘플 테스트를 유지한다.
- 한컴오피스 재비교 결과, HWP5/HWPX의 UI `쪽 기준`은 렌더도 `BodyBased`를 유지해야 한다.
  - UI: `attr bit0=1` / `textBorder=PAPER` → `쪽 기준`
  - 렌더: `BodyBased`
  - 이 문서는 위쪽 여백 10mm, 테두리 간격 5.01mm라 상단 외곽선이 4~5mm 근처로 보일 수 있다.
  - 따라서 종이/쪽 기준 판정은 상단만 보지 말고 좌측 외곽선 위치까지 함께 확인한다.

## 수정 내용

- `insert_grid_overlay()`를 회색 선 격자에서 1px 파란 점 격자로 변경했다.
- 점 격자는 SVG 원점에서 시작하며, `--show-grid=3mm`이면 종이 기준 3mm 간격으로 반복된다.
- HWP3 page border 주석은 HWP3 원본에 종이 기준 선택이 없다는 사용자 피드백과 맞게 보정했다.
- HWP5/HWPX `PAGE_BORDER_FILL.attr bit0=1` / `textBorder=PAPER` 파싱은 UI `쪽 기준`, 렌더 `BodyBased`를 유지한다.
- `setPageBorderFill`도 `basis:"page"` 저장 시 UI 기준과 렌더 기준을 함께 `쪽 기준` 계열로 유지한다.
- Stage 28 중간의 `PaperBased` 가설은 사용자 시각 확인에서 실패했으므로 폐기했다.

## 검증 계획

- `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp --page 0 --show-grid=3mm`
- `target/debug/rhwp export-svg samples/종이기준.hwp --page 0 --show-grid=3mm`
- `cargo test -q page_border_fill_sample_basis_matches_hancom_ui`
- `cargo test -q test_parse_page_border_fill --lib`
- `cargo test -q test_parse_page_border_fill_basis_from_text_border --lib`
- Rust 변경이므로 `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `cargo fmt --all -- --check`
- `git diff --check`

## 검증 결과

- `cargo run --quiet --bin rhwp -- export-svg samples/종이기준.hwp --page 0 --show-grid=3mm`
  - `rhwp-grid` pattern width/height = `11.3386px` (3mm)
  - 점 = `1px` 파란 rect, 종이 원점 기준
  - 종이 기준 테두리 상단 = `18.8933px` (5mm)
- `cargo test -q test_parse_page_border_fill --lib`: 통과
- `cargo test -q test_parse_page_border_fill_basis_from_text_border --lib`: 통과
- `cargo test -q page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp --page 0 --show-grid=3mm`
  - 최종 기준: `paper_based=false`, `attr=0x00000001`, spacing `1420`
  - 외곽선 좌측: 약 `8.9~9.5mm`
  - 외곽선 상단: 약 `3.9~4.5mm`
  - 첫 로고 위치: `x=11.642mm`, `y=14.873mm`
  - 로고-외곽선 간격: 좌측 약 `2.72mm`, 상단 약 `10.95mm`
- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
