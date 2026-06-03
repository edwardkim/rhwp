# Task 1129 Stage 29 - 쪽 기준 상단 외곽선 세로 기준 보정

## 사용자 지적

- Stage 28 이후에도 `samples/hwp3-sample16-hwp5.hwp`의 상단 외곽선 위치가 한컴오피스와 다르다.
- 비교 기준:
  - 첫 번째 스샷: rhwp-studio
  - 두 번째 스샷: 한컴오피스
- 다음부터 시각 판단 요청 전에는 반드시 `wasm-pack build --target web --out-dir pkg`를 Codex가 먼저 수행한다.
- 추가 보정 후 작업지시자가 시각적 판단 완료를 확인했다.

## 재분석

- 현재 `쪽 기준(BodyBased)` 외곽선의 가로 기준은 `body_area.x - spacing` 계열이라 종이 기준과 구분된다.
- 반면 세로 기준은 `margin_top - spacing` 계열이라 상단 외곽선이 너무 위에 있다.
- 한컴오피스 스샷은 상단 외곽선이 현재 rhwp-studio보다 아래에 있으며, `body_area.y - spacing` 계열에 더 가깝다.

## 수정 계획

- `src/renderer/layout.rs::build_page_borders()`에서 `BodyBased` 세로 기준을 `layout.body_area.y`와 `layout.body_area.height`로 변경한다.
- `src/document_core/queries/rendering.rs::get_page_info_native()`의 `pageBorderTop/Bottom`도 같은 세로 기준으로 맞춘다.
- `쪽 기준` 페이지 테두리의 이중선/삼중선은 선 묶음이 본문 영역 안쪽으로 파고들지 않도록, 위치 계산에서 선 묶음 시각 폭을 한컴 비교 기준에 맞춰 바깥쪽으로 추가 반영한다.
- Stage 28의 `export-svg --show-grid=Nmm` 1px 종이 기준 점 격자는 유지한다.

## 검증 계획

- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp --page 0 --show-grid=3mm`
- `cargo test -q page_border_fill_sample_basis_matches_hancom_ui --lib`
- `cargo test -q test_parse_page_border_fill --lib`
- `cargo test -q test_parse_page_border_fill_basis_from_text_border --lib`
- `cargo fmt --all -- --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- rhwp-studio는 Vite dev 서버가 실행 중이므로 TypeScript/CSS 변경은 실시간 반영된다. 이번 보정은 Rust/WASM 렌더링 경로 변경이라 `wasm-pack build --target web --out-dir pkg`를 필수 검증으로 수행한다.

## 검증 결과

- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp --page 0 --show-grid=3mm`
  - `paper_based=false`, `attr=0x00000001`, spacing `1420`
  - 1차 보정 후 외곽선 좌측: 약 `8.9~9.5mm`
  - 1차 보정 후 외곽선 상단: 약 `13.9~14.5mm`
  - 첫 로고 위치: `x=11.642mm`, `y=14.873mm`
  - 로고-외곽선 간격: 좌측 약 `2.72mm`, 상단 약 `0.95mm`
- `cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp --page 0 --show-grid=3mm -o /tmp/rhwp-stage29-border-outset`
  - 3mm 격자 pitch: `11.3386px`
  - 첫 로고 위치 유지: `x=11.642mm`, `y=14.873mm`, `w=33.218mm`, `h=8.283mm`
  - 이중선 보정 후 외곽선 좌측: 약 `8.13~8.68mm`
  - 이중선 보정 후 외곽선 상단: 약 `13.13~13.69mm`
  - 로고-외곽선 간격: 좌측 약 `2.96~3.52mm`, 상단 약 `1.18~1.74mm`
- `cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp --page 0 --show-grid=3mm -o /tmp/rhwp-stage29-factor25`
  - 한컴오피스 스샷과의 미세 차이 보정을 위해 `쪽 기준` 이중선/삼중선 페이지 테두리 outset 계수를 `2.0`에서 `2.5`로 조정
  - 3mm 격자 pitch 유지: `3.000mm`
  - 첫 로고 위치 유지: `x=11.642mm`, `y=14.873mm`, `w=33.218mm`, `h=8.283mm`
  - 추가 보정 후 외곽선 좌측: 약 `7.73~8.28mm`
  - 추가 보정 후 외곽선 상단: 약 `12.74~13.29mm`
  - 로고-외곽선 간격: 좌측 약 `3.36~3.91mm`, 상단 약 `1.58~2.14mm`
- `cargo test -q page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `cargo test -q test_parse_page_border_fill --lib`: 통과
- `cargo test -q test_parse_page_border_fill_basis_from_text_border --lib`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- 작업지시자 시각적 판단: 완료
