# Task #1261 최종 보고

## 개요

한컴 기준과 다르게 겹치거나 단 하단을 넘던 다음 사례를 공통 레이아웃 로직으로 수정했다.

- `samples/3-10월_교육_통합_2022.hwp` 5쪽 `문28)` 조건 박스와 선택지 겹침
- `samples/3-09월_교육_통합_2024-미주사이20.hwp` 10쪽 `문8)` 미주 제목과 직전 수식 겹침
- 같은 10쪽 오른쪽 단 `문12)` 미주 내용 overflow
- PR CI에서 드러난 `hwp3-sample16-hwp5*.hwp` 3쪽 TAC Shape trailing 회귀

## 원인

- 문28 조건 박스는 글자처럼 취급되는 Shape 컨트롤인데, 실제 높이인 `shape_attr.current_height`가 `common.height`보다 훨씬 컸다.
- 문8은 compact 미주 새 문항 제목 보정이 직전 display 수식의 실제 렌더 하단이 아니라 trailing line spacing을 뺀 추정값을 기준으로 삼았다.
- 문12는 단일줄 빈 미주 separator가 이미 `미주 사이 20mm`를 포함한 상태에서 page-path vpos와 제목 gap 보존이 다시 간격을 더해 오른쪽 단 제목들이 누적 하강했다.
- PR CI 회귀는 `자리차지` TAC Shape의 빈 host 줄에서 `LINE_SEG.line_height`가 이미 개체 점유 높이를 포함하는데 trailing spacing을 다시 더한 탓이었다.

## 변경 내용

- `src/renderer/layout/paragraph_layout.rs`
  - TAC Shape 높이 계산에 `shape_attr.current_height`를 함께 반영했다.
  - 현재 줄에 실제 TAC Shape가 있는 경우에만 Shape 줄 보정을 적용하도록 제한했다.
  - 공백뿐인 글상자 안내 줄은 폰트 높이 축소 대상에서 제외했다.
  - 공백뿐인 `자리차지` TAC Shape host 줄은 trailing line spacing을 중복 적용하지 않도록 했다.
- `src/renderer/height_cursor.rs`, `src/renderer/layout.rs`, `src/renderer/typeset.rs`
  - compact 미주 제목 보정이 직전 렌더 콘텐츠 하단과 `미주 사이` 값을 공통 기준으로 사용하도록 정리했다.
  - 확장 미주 사이에서 단일줄 separator 뒤 page-path vpos가 제목을 소폭 아래로 밀 때 간격을 두 번 더하지 않도록 vpos base를 보정했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 문28 조건 박스 하단보다 선택지 첫 줄이 아래에서 시작하는 회귀 테스트를 추가했다.
  - 문8이 직전 수식 하단 뒤 `미주 사이 20mm` 간격을 유지하는지 검증했다.
  - 문10/문11/문12 제목과 문12 꼬리가 한컴 PDF bbox 및 단 하단 안에 남는지 검증했다.
- `mydocs/orders/20260603.md`, `mydocs/plans/task_m100_1261.md`, `mydocs/plans/task_m100_1261_impl.md`
  - 이슈와 작업 계획을 기록했다.
- `pdf-large/`
  - 2024년 9월 기준 PDF 2건을 비교 기준 자료로 포함했다.

## 검증

- `cargo fmt -- --check`
- `cargo test --lib height_cursor -- --nocapture`
- `cargo test --test issue_1116 -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1261_2022_oct_page5_question28_choices_stay_below_condition_box -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `git diff --check`
- `미주사이20` p10 SVG/PNG와 한컴 PDF bbox 비교
- 작업지시자 `localhost:7700` 시각 검증 확인

## 후속

PR #1264의 GitHub Actions를 다시 확인한다. WASM 산출물은 작업지시자가 수동 빌드/시각 검증한다.
