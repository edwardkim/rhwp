# Task 1274 Stage 5

## 대상

- 문서: `samples/3-11월_실전_통합_2022.hwp`
- PDF: `pdf/3-11월_실전_통합_2022.pdf`
- 비교 PNG: `output/task1274/2022-11-practice/compare/compare_012.png`

## 현상

Stage4 이후 `2022-11-practice`는 SVG/PDF/비교 PNG가 21쪽씩 생성되고 manifest overflow 로그도 비어 있다.
그러나 12쪽에서 본문 좌/우 단 분기 위치가 PDF와 다르다.
2쪽과 9쪽도 로그는 사라졌지만 실제 흐름 위치 차이가 남아 있어, 남은 문제는 overflow 오탐이 아니라 본문 높이 누적/단 분기 판단 차이다.

## 목적

- 12쪽 좌/우 단 분기가 PDF와 달라지는 원인을 본문 높이 누적 관점에서 특정한다.
- 문항 번호나 페이지 번호 하드코딩 없이 공통 레이아웃 로직으로 수정한다.
- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 원인

12쪽 왼쪽 단 하단의 `pi=571`은 저장된 LINE_SEG 기준 실제 보이는 content span이 남은 단 높이와 compact 미주 하단 bleed 허용 안에 들어간다.
하지만 pagination의 split 판단은 formatter의 순차 줄 진행량과 마지막 줄 trailing line_spacing까지 포함한 높이를 사용해, 한컴/PDF보다 먼저 `lines=0..4`와 `lines=4..6`으로 나눴다.

## 수정

- compact 미주 기본 간격(`미주 사이` 7mm)에서만 하단 단 분기 판단에 LINE_SEG content span을 사용한다.
- content span이 남은 높이 + compact 미주 하단 bleed 허용치 안에 있으면 문단을 split하지 않고 같은 단에 유지한다.
- `미주 사이 20mm` 변형 파일은 별도 페이지 분기가 필요하므로 이 예외를 적용하지 않는다.

## 검증

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice`
  - SVG 21쪽, PDF 21쪽, 비교 PNG 21쪽
  - `output/task1274/2022-11-practice/manifest.json`의 `overflow_lines` 비어 있음
- `compare_012.png`
  - `pi=571`이 왼쪽 단에서 유지되고, 오른쪽 단은 PDF처럼 `(ⅲ)` 문단부터 시작한다.

## 남은 작업

- 12쪽 오른쪽 단 하단과 2쪽/9쪽의 세부 흐름 차이는 다음 스테이지에서 이어서 확인한다.
