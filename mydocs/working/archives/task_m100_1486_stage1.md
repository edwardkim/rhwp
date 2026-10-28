# Task M100 #1486 Stage 1: HWPX 분할 표 내부 TAC 표 배치 보정

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall

## 배경

`hwpx_sample2.hwpx` 9쪽에서 페이지 상단의 "구분/조회방법" 중첩 표가 본문 오른쪽 밖으로 밀려난다. 같은 샘플의 HWP 입력과 한컴 PDF 기준 출력에서는 해당 표가 본문 좌측 영역 안에 배치된다.

## Stage 1 목표

- 분할된 외부 표 안에서 `treatAsChar` 중첩 표의 x 위치가 앞 텍스트 너비만큼 과도하게 밀리는 원인을 수정한다.
- 문제 샘플의 9쪽 렌더 트리에서 중첩 표가 본문 폭 안에 들어오는 회귀 테스트를 추가한다.
- 기존 분할 표/중첩 표 렌더링 영향 범위를 최소화한다.

## 구현 계획

1. `src/renderer/layout/table_partial.rs`에서 분할 표 셀 내부 TAC 중첩 표의 남은 줄 폭을 계산한다.
2. TAC 표가 앞 텍스트 뒤 남은 폭에 들어가지 않으면 셀 좌측 기준으로 배치한다.
3. `tests/issue_1486_hwpx_partial_tac_table.rs`에 HWPX 샘플 9쪽의 중첩 표 bbox 검증을 추가한다.

## 검증 계획

- `cargo test --release --test issue_1486_hwpx_partial_tac_table`
- 관련 회귀 테스트 일부 실행
- 수정 후 `export-render-tree`/`export-svg`로 9쪽 중첩 표 bbox와 시각 산출물을 확인
- 최종 단계에서 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`

## 구현 결과

- `src/renderer/layout/table_partial.rs`에서 분할 표 셀 내부 TAC 중첩 표가 앞 텍스트 뒤 남은 폭에 들어가지 않을 때 셀 좌측 기준으로 배치하도록 보정했다.
- `tests/issue_1486_hwpx_partial_tac_table.rs`를 추가해 `hwpx_sample2.hwpx` 9쪽 상단 TAC 중첩 표가 페이지 본문 오른쪽을 넘지 않는지 검증했다.
- 이슈 첨부 샘플과 한컴 PDF 기준 자료를 저장소 샘플/검증 자료로 추가했다.

## 검증 결과

- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture` 통과
  - 수정 후 문제 표 bbox: `x=77.88`, `right=717.68`
- `cargo test --release --test issue_1195_cell_table_empty_line --test issue_1285_tac_sequence_right_align --test issue_1133_nested_table_valign --test issue_1073_nested_table_split` 통과
- `cargo build --release` 통과
- `export-render-tree`/`export-svg` 재생성 및 PNG 확인
  - 산출물: `output/poc/task1486/rt_hwpx_after/render_tree_009.json`
  - 산출물: `output/poc/task1486/svg_hwpx_after/hwpx_sample2_009.svg`
  - 산출물: `output/poc/task1486/svg_hwpx_after/hwpx_sample2_009.png`
- `git diff --check` 통과
- `cargo fmt --check` 통과
- `cargo test --release --lib` 통과: 1923 passed, 6 ignored
- `cargo test --profile release-test --tests` 통과
- `cargo clippy --all-targets -- -D warnings`는 실행 중 사용자가 PDF 불일치 확인 후 중지 지시. 남은 프로세스 없음.

## 남은 문제

- 작업지시자 시각 확인 결과, 아직 한컴 PDF 기준 출력과 완전히 맞지 않는다.
- Stage 1은 "페이지 오른쪽 경계 밖으로 밀림"을 막는 1차 보정으로 커밋하고, PDF 정합 문제는 Stage 2에서 이어서 분석한다.
