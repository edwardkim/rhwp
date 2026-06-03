# Task #1261 최종 보고

## 개요

`samples/3-10월_교육_통합_2022.hwp` 5쪽 `문28)` 조건 박스 주변에서 선택지 줄이 한컴오피스 기준보다 위로 올라가 조건 박스 내부 문장과 겹치던 문제를 수정했다.

## 원인

문28 조건 박스는 글자처럼 취급되는 Shape 컨트롤이다. 해당 Shape는 `common.height`보다 `shape_attr.current_height`가 훨씬 큰데, 기존 레이아웃은 일부 경로에서 `common.height`만 사용했다. 또한 같은 문단에 TAC Shape가 있다는 이유로 글상자 안내용 공백 줄까지 폰트 높이로 축소해 다음 줄 y 진행량이 줄었다.

## 변경 내용

- `src/renderer/layout/paragraph_layout.rs`
  - TAC Shape 높이 계산에 `shape_attr.current_height`를 함께 반영했다.
  - 현재 줄에 실제 TAC Shape가 있는 경우에만 Shape 줄 보정을 적용하도록 제한했다.
  - 공백뿐인 글상자 안내 줄은 폰트 높이 축소 대상에서 제외했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 문28 조건 박스 하단보다 선택지 첫 줄이 아래에서 시작하는 회귀 테스트를 추가했다.
- `mydocs/orders/20260603.md`, `mydocs/plans/task_m100_1261.md`, `mydocs/plans/task_m100_1261_impl.md`
  - 이슈와 작업 계획을 기록했다.

## 검증

- `cargo fmt -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1261_2022_oct_page5_question28_choices_stay_below_condition_box -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1245_2022_page7_square_pictures_use_relative_line_vpos -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture`
- `cargo test --lib tac -- --nocapture`
- `cargo test --lib textbox -- --nocapture`
- `wasm-pack build --target web --out-dir pkg`
- 작업지시자 `localhost:7700` 시각 검증 확인

## 후속

`pdf-large/3-09월_교육_통합_2024-미주사이20-2024.pdf`의 문8 겹침 원인은 다음 스테이지에서 별도 조사한다.
