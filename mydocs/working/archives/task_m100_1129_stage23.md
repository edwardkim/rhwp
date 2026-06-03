# Task m100 #1129 Stage 23

## 문제

`hwp3-sample16-hwp5.hwp` 첫 페이지의 상단 로고는 저장 좌표와 한컴 UI 값이 같다.

- 위치: `종이 / 왼쪽 11.64mm`, `종이 / 위 14.87mm`
- 크기: `33.22mm x 8.28mm`

하지만 rhwp-studio에서는 `쪽 기준` 외곽선 상단이 로고와 거의 같은 위치에 렌더링된다.

- rhwp SVG 로고 bbox: `y=56.21px`
- rhwp 쪽 기준 외곽선 상단 기준선: `y=56.69px`
- 실제 이중선 중심: `y=55.64px`, `57.74px`

즉 그림 좌표 파싱 문제가 아니라, `쪽 기준` 외곽선 기준선이 선 묶음의 시각적 두께를 고려하지 않아 본문/개체 쪽으로 파고드는 문제가 직접 원인이다.

## 수정 방향

- `쪽 기준` 외곽선은 본문 영역에서 바깥쪽으로 배치하되, 선 묶음의 시각적 폭만큼 외곽선 박스를 추가로 바깥쪽 확장한다.
- 이중선/삼중선은 `create_border_line_nodes()`와 같은 선 묶음 폭 계산을 공유한다.
- `getPageInfo()`의 `pageBorder*`도 실제 렌더 기준과 맞춘다.

## 검증 계획

- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp -p 0 -o output/poc/task1129_stage23_verify`
- SVG에서 로고 y와 상단 외곽선 y 좌표 비교
- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `cargo fmt --all -- --check`
- `git diff --check`

## 검증 결과

- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16-hwp5.hwp -p 0 -o output/poc/task1129_stage23_verify`
  - `paper_based=false`, `spacing(L=1420,R=1420,T=1420,B=1420)` 확인
  - 로고 bbox는 기존 저장 좌표 그대로 `y=56.21px`
  - 쪽 기준 상단 이중선 중심은 `y=52.64px`, `54.74px`로 보정되어 로고와 겹치지 않음
- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build` (`rhwp-studio`): 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
