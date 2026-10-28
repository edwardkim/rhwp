# task 1510 stage6: CI issue_1073 회귀 보정

## 배경

- PR #1518 head `1c17defd`의 GitHub Actions `Build & Test`가 실패했다.
- 실패 테스트: `tests/issue_1073_nested_table_split.rs::kps_ai_nested_table_split_no_title_duplication`
- 실패 메시지: `첫 조각(page 65)에 표 제목 누락 — 분할 미발생 의심`

## 원인

- stage5에서 #1510 샘플의 작은 visible float 표를 PDF와 맞추기 위해 `MeasuredTable` 행 높이를 선언 높이에 맞추도록 보정했다.
- 이 보정이 `samples/kps-ai.hwp`의 큰 표 `pi=329`에도 적용되어 측정 높이가 약 `1704px`에서 선언 높이 약 `858px`로 압축되었다.
- 그 결과 문서 전체 페이지 수가 78쪽에서 77쪽으로 줄고, `pi=674` 중첩 표 첫 조각이 기존 page 65가 아니라 page 64로 당겨졌다.

## 수정 방향

- 선언 높이 보정은 측정 높이와 선언 높이가 가까운 fixed-size visible float 표에만 적용한다.
- 측정된 본문 높이가 선언 본문 높이보다 과도하게 큰 표는 실제 콘텐츠가 더 큰 표로 보고 압축하지 않는다.
- #1510 샘플의 0~1px PDF 정합은 유지하면서 #1073 중첩 표 분할 페이지 배정을 복원한다.

## 검증 계획

- `cargo test --test issue_1073_nested_table_split -- --nocapture`
- `cargo test --test issue_1510 -- --nocapture`
- `cargo test --test svg_snapshot issue_157_page_1 -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- 필요 시 `cargo clippy --all-targets -- -D warnings`
- 필요 시 `wasm-pack build --target web --out-dir pkg`

## 구현

- `fit_measured_table_to_declared_height`에 안전 범위를 추가했다.
- 선언 본문 높이가 측정 본문 높이의 `0.75..=1.35` 범위를 벗어나면 보정하지 않는다.
- #1510처럼 작은 fixed-size visible float 표의 근소한 PDF 드리프트는 보정하고, #1073처럼 실제 콘텐츠가 선언 높이보다 훨씬 큰 분할 표는 압축하지 않는다.

## 로컬 검증 결과

- PASS: `cargo test --test issue_1073_nested_table_split -- --nocapture`
- PASS: `cargo test --test issue_1510 -- --nocapture`
- PASS: `cargo test --test svg_snapshot issue_157_page_1 -- --nocapture`
- PASS: `cargo fmt --check`
- PASS: `git diff --check`
- PASS: `wasm-pack build --target web --out-dir pkg`
- STOPPED: `cargo clippy --all-targets -- -D warnings`
  - 사용자가 대기 중 중지를 지시해 code 130으로 중단했다.
  - 중단 시점까지 새 경고/오류 출력은 없었다.
- #1510 PDF 라인 검출(144dpi, 1쪽)
  - HWPX Hancom/current: `[204,264,324,543,599,655]` / `[204,264,324,544,600,656]`
  - HWP Hancom/current: `[164,224,283,543,599,655]` / `[164,224,284,544,600,656]`

## GitHub Actions

- stage6 push 후 PR #1518의 Actions 상태를 별도로 확인한다.
- 로컬 검증 결과와 GitHub Actions 결과는 섞어 쓰지 않는다.
