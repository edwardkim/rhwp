# Task 1274 Stage 4

## 대상

- 문서: `samples/3-11월_실전_통합_2022.hwp`
- PDF: `pdf/3-11월_실전_통합_2022.pdf`
- 남은 manifest overflow:
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=0 line=0 y=416.7 col_bottom=407.4 overflow=9.3px`
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=0 line=0 y=352.4 col_bottom=343.0 overflow=9.3px`
- 시각 차이:
  - `compare_012.png` 기준 12쪽 좌/우 단 분기가 PDF와 다르다.

## 목적

Stage3에서 11쪽 `pi=553` compact 미주 하단 bleed 오탐은 제거했다.
이번 단계에서는 남은 `pi=0` overflow가 실제 본문/표지 overflow인지, 특정 그림/표 내부의 본문 하단 판정 오탐인지 페이지를 특정한다.
또한 12쪽 단 분기 차이가 같은 높이 측정 오차에서 비롯되는지 함께 확인한다.

## 진행 원칙

- 진행 중 자동 테스트는 `tests/issue_1139_inline_picture_duplicate.rs`만 사용한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.
- 수정은 문항 번호나 특정 페이지 하드코딩이 아니라, 공통 레이아웃/측정 로직으로 처리한다.

## 원인

남은 `pi=0` 로그는 본문 문단 0이 아니라 글상자/비본문 배치 문맥의 로컬 문단 0에서 발생했다.
해당 문맥은 내부 클립 높이가 본문 단 높이보다 짧으므로, 본문 페이지 overflow 진단과 같은 기준으로 기록하면 manifest에 오탐이 남는다.

Stage3의 compact 미주 하단 bleed 허용도 본문 단 흐름에서만 적용되어야 하므로, 동일한 body-flow 판정을 draw 단계 overflow 로그에도 사용한다.

## 수정

- `LayoutEngine::is_body_flow_col_area`를 추가해 현재 본문 단 높이와 draw 대상 단 높이가 같은지 확인했다.
- `LAYOUT_OVERFLOW_DRAW` 출력은 본문 단 흐름이고 셀 문맥이 아닐 때만 수행하도록 제한했다.
- compact 미주 하단 bleed 허용도 본문 단 흐름에서만 적용되도록 조건을 좁혔다.

## 검증

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice`
  - SVG 21쪽, PDF 21쪽, 비교 PNG 21쪽
  - `output/task1274/2022-11-practice/manifest.json`의 `overflow_lines` 비어 있음

## 남은 작업

- `compare_012.png` 기준 12쪽 좌/우 단 분기 위치가 PDF와 다르다.
- 2쪽, 9쪽도 manifest overflow 오탐은 사라졌지만 실제 흐름 위치 차이가 남아 있다.
- 위 시각 차이는 Stage4 범위에 섞지 않고 다음 스테이지에서 원인을 분석한다.

## 분석 결과

- `pi=0` overflow 2건은 본문 flow 단이 아니라 그림/내부 렌더 경로에서 전달된 `col_area`를 본문 하단처럼 판정한 오탐이다.
- 본문 flow 단의 `col_area`는 현재 페이지의 `body_area`와 y/height가 일치한다.
- 반대로 문제의 `pi=0` 로그는 page body 기준이 아닌 내부 그림 렌더링의 영역 하단을 `col_bottom`으로 사용하면서 생겼다.

## 수정

- `src/renderer/layout.rs`
  - 현재 `col_area`가 실제 본문 flow 단인지 확인하는 `is_body_flow_col_area`를 추가했다.
- `src/renderer/layout/paragraph_layout.rs`
  - 줄 단위 `LAYOUT_OVERFLOW_DRAW` 기록을 실제 본문 flow 단에 한정했다.
  - compact 미주 하단 small bleed 허용도 본문 flow 단에서만 적용되도록 좁혔다.
  - fast path overflow 기록 역시 본문 flow 단에서만 수행하도록 제한했다.

## 검증

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 테스트 통과.
- `cargo build --bin rhwp`
  - 네이티브 SVG export용 바이너리 빌드 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice`
  - SVG/PDF/비교 PNG: 21/21/21.
  - `output/task1274/2022-11-practice/manifest.json`
    - `overflow_lines: []`

## 시각 확인

- `output/task1274/2022-11-practice/compare/compare_011.png`
  - 11쪽 하단의 compact 미주 bleed는 페이지 테두리 안에 남는다.
  - overflow manifest는 비었다.
- `output/task1274/2022-11-practice/compare/compare_012.png`
  - 12쪽 좌/우 단 분기 차이는 남아 있다.
  - `dump-pages -p 11` 기준 `pi=571`이 `lines=0..4` / `lines=4..6`으로 갈라지며, PDF는 왼쪽 단에 더 많은 줄이 남는 형태다.
  - 이 문제는 overflow 오탐과 별개이므로 Stage5에서 `pi=571` split 기준을 별도 분석한다.
