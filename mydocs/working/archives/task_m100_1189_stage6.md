# Task M100 #1189 Stage 6

## 목적

Stage5 커밋 이후 `3-11월_실전_통합_2022.hwp` 1쪽 `문1)`과 첫 수식 사이의 가로 간격이 한컴오피스 기준과 다르게 보이는 문제를 PDF/한컴 기준 산출물과 비교해 보정한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- 작업 브랜치: `local/task_m100_1189`
- 선행 커밋: `6f192f01` (`task 1189: 10쪽 미주 하단 오버랩 보정`)
- 대상 문서: `samples/3-11월_실전_통합_2022.hwp`
- 대상 페이지: 1쪽
- 사용자 시각 판정: `문1)` 뒤 첫 수식이 한컴오피스 표시보다 오른쪽으로 멀거나 시작 간격이 다르다.

## 초기 판단

1. 문제는 미주 본문 흐름이 아니라 일반 본문 1쪽 첫 문항의 인라인 수식 배치이다.
2. `문1)` 텍스트 뒤 inline equation TAC의 x 위치, control 앞 공백 처리, 수식 bbox 원점 보정 중 하나가 원인일 가능성이 높다.
3. 같은 문서 10~17쪽 미주 후속 보정과 섞이지 않도록 1쪽 `문1)` 인라인 수식만 좁게 확인한다.

## 진행 계획

1. PDF 1쪽과 현재 SVG 1쪽을 각각 PNG로 변환해 `문1)` 수식 위치를 비교한다.
2. `dump`/`dump-pages`로 `문1)` 문단의 텍스트, 컨트롤, char offset, line segment, equation bbox를 확인한다.
3. 원인이 수식 전체 배치인지 수식 내부 좌측 여백인지 분리한다.
4. 좁은 보정과 회귀 테스트를 추가하고, `issue_1139_inline_picture_duplicate` 대상 테스트로 확인한다.

## 현재 상태

- 2026-06-01: 작업지시자가 1쪽 `문1)`과 수식 사이 간격 불일치를 보고했다. Stage6 문서를 만들고 PDF/한컴 기준 비교를 시작한다.
- 2026-06-01: PDF 1쪽과 현재 SVG/PNG를 비교했다. `문1)` 문단의 본문 텍스트 앞에 HWP5 인라인 컨트롤 placeholder `U+FFFC`가 2개 포함되어 있었고, SVG 출력에서는 이 문자가 보이지 않지만 텍스트 폭 측정에는 포함되어 첫 수식 x 좌표가 `95.0px`대로 밀렸다.
- 2026-06-01: `U+FFFC`를 텍스트 측정 경로에서 0폭으로 처리하도록 보정했다. 수정 후 `문1)` 첫 수식 x 좌표는 `71.0px`대로 이동해 PDF/한컴 기준의 문항 번호 뒤 간격에 맞아졌다.

## 검증 기록

- PDF 기준 PNG:
  - `output/task1189_stage6_3-11_page1/pdf/pdf_page-01.png`
- 수정 후 rhwp PNG:
  - `output/task1189_stage6_3-11_page1/fixed/rhwp_page1.png`
- 수정 전 debug:
  - `TAC_LINE pi=0 ... run_tacs=[(4, 61.013333333333335, 4)]`
  - paragraph base x를 포함한 첫 수식 bbox x가 약 `95.0px`.
- 수정 후 검증:
  - `cargo fmt --all --check` 통과.
  - `cargo test test_inline_object_placeholder_has_zero_advance -- --nocapture` 통과.
  - `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_nov_page1_question1_marker_gap_matches_pdf -- --nocapture` 통과.
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과 (`33 passed`).
  - 커밋 전 `cargo test --tests` 통과.
  - 커밋 전 `wasm-pack build --target web --out-dir pkg` 통과.

## 커밋 전 남은 검증

- 작업지시자 시각 확인 완료 후 커밋 진행.
