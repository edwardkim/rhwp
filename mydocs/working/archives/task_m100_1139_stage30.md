# Task M100 #1139 Stage 30

## 목적

Stage29 커밋 이후 `3-09월_교육_통합_2022.hwp` 머릿말/머리말이 한컴오피스 표시와 RHWP 표시에서 다르게 보이는 문제를 별도 스테이지로 추적한다.

## 시작 기준

- 기준 커밋: `6b4d6e3e` (`task 1139: Stage29 2023 문항 겹침 및 미주 분배 보정`)
- Stage29 변경은 커밋 완료했다.
- Stage30 문서는 Stage29 커밋 이후 새 변경으로 생성한다.
- Stage30 소스 수정은 작업지시자 승인 후 진행한다.

## 보고된 문제

- 대상 파일: `samples/3-09월_교육_통합_2022.hwp`
- 기준 비교: 한컴오피스 표시 화면 및 필요 시 `pdf/3-09월_교육_통합_2022.pdf`
- 작업지시자 캡처 기준 첫 페이지 상단 머릿말 영역이 한컴오피스와 RHWP에서 다르게 보인다.
- RHWP 화면에서는 상단 제목/회차 박스/로고 영역과 본문 외곽선의 상대 위치가 한컴오피스 기준과 맞지 않는 것으로 보인다.

## 진행 계획

1. `3-09월_교육_통합_2022.hwp` 1쪽을 RHWP SVG/PNG와 PDF 기준 PNG로 다시 추출한다.
2. SVG를 PNG로 변환할 때는 `rsvg-convert`를 사용한다.
3. 머릿말 영역의 HWP control, shape/table, header/footer 여부, page/master 관계를 dump로 확인한다.
4. 한컴 기준과 RHWP 기준의 제목 박스, 회차 박스, 로고, 본문 외곽선 y/x 좌표 차이를 계측한다.
5. 원인이 header 영역 offset, section/page margin, shape anchor, table/shape wrap, master page 처리 중 어디에 있는지 분리한다.
6. 원인이 확인되면 머릿말 위치 회귀 테스트를 추가한다.
7. Rust/WASM 수정 후 `cargo fmt --all --check`, `cargo build`, 관련 회귀 테스트, `wasm-pack build --target web --out-dir pkg`, `git diff --check`를 실행한다.
8. 1쪽 비교 산출물을 생성해 작업지시자 시각 확인을 받는다.

## 현재 상태

- 2026-05-30: 작업지시자가 Stage29 현상황 커밋과 새 스테이지 시작을 지시했다.
- 2026-05-30: Stage29 커밋 이후 Stage30 문서를 생성했다.
- 2026-05-30: 1쪽 PDF/RHWP 비교 산출물을 `output/task1139_stage30_header_page1/`에 생성했다. SVG→PNG 변환은 `rsvg-convert`를 사용했다.
- 2026-05-30: `pdftocairo` PDF SVG와 RHWP SVG 좌표를 비교해 제목/회차 머릿말 표가 RHWP에서 본문 여백 9mm(`x=34.01px`) 기준으로 배치되고, 한컴/PDF는 paper-based 쪽 테두리 왼쪽 간격 7mm(`x=26.45px`) 기준으로 배치됨을 확인했다. 로고 그림은 별도 Picture 경로로 이미 기준에 가까워 머릿말 전체 이동은 적용하지 않았다.
- 2026-05-30: paper-based 쪽 테두리가 있는 페이지의 머릿말 표 레이아웃 기준 영역만 `PageBorderFill.spacing_left/right` 기준으로 보정했다. Picture/Shape/일반 텍스트 머릿말 경로와 footer 표 경로는 기존 기준을 유지했다.
- 2026-05-30: `issue_1139_exam_2022_page1_header_table_uses_page_border_spacing` 회귀 테스트를 추가해 1쪽 머릿말 제목 표 bbox가 7mm 기준에 맞는지 검증했다.
- 2026-05-30: 검증 명령:
  - `cargo test --test issue_1139_inline_picture_duplicate issue_1139_exam_2022_page1_header_table_uses_page_border_spacing -- --nocapture`
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo fmt --all`
  - `cargo fmt --all --check`
  - `cargo build`
  - `git diff --check`
  - `wasm-pack build --target web --out-dir pkg`
- 2026-05-30: 작업지시자가 Stage30 시각적 검증 완료를 확인했다.
