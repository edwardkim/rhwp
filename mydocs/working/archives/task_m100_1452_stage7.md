# Task M100 #1452 Stage 7 시작 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `48d4efe6 task 1452: 투명도 그림 렌더링 중복 보정`

## 1. 배경

`samples/투명도0-50.hwp`에서 두 번째 그림 뒤에 커서를 두고 Enter를 누르면 두 그림이 모두
`글자처럼 취급`이므로 원래 문단에는 두 그림이 그대로 남고, 커서는 다음 빈 줄로 내려가야 한다.

현재 rhwp-studio에서는 Enter 후 두 번째 그림이 사라져 보인다.

## 2. 초기 원인 추정

- 샘플 문단은 텍스트가 없고 TAC 그림 2개를 `LINE_SEG` 2개로 표현한다.
- `split_paragraph_native()`는 `Paragraph::split_at()` 후 양쪽 문단에 `reflow_line_segs()`를 호출한다.
- 현재 `reflow_line_segs()`는 텍스트 없는 문단을 무조건 단일 `LineSeg`로 재계산한다.
- 따라서 Enter 후 원래 문단의 두 번째 TAC 그림 줄 정보가 사라지고, Stage 6의 줄별 TAC 매핑도 두 번째
  그림을 렌더링할 줄을 찾지 못한다.

## 3. 개선 목표

- 텍스트 없는 문단에 여러 TAC 그림/도형/표/수식이 있을 때, 개체 폭이 사용 가능 너비를 넘으면 한컴처럼
  다음 줄로 배치되도록 `line_segs`를 재계산한다.
- `투명도0-50.hwp`에서 두 번째 그림 뒤 Enter 후 원래 문단에 그림 2개가 모두 남는지 회귀 테스트를 추가한다.

## 4. 검증 계획

- `cargo test --lib issue1452 -- --nocapture`
- `cargo test --lib issue1151 -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- Rust/WASM 변경 후 `wasm-pack build --target web --out-dir pkg`

## 5. 구현 내용

- `src/renderer/composer/line_breaking.rs`
  - 텍스트가 비어 있고 TAC 그림/도형/표/수식/양식 개체가 있는 문단에서, 개체 폭을 사용 가능 너비에
    맞춰 줄 단위로 다시 패킹하도록 보정했다.
  - 각 줄의 `text_start`는 해당 줄의 첫 TAC 순번으로 지정한다. Stage 6의 반복 빈 TAC 줄 매핑이 이 값을
    기반으로 각 그림 control을 한 줄에 하나씩 대응한다.
  - 기존 `LineSeg`가 있으면 줄간격, segment width, tag를 최대한 보존해 한컴 저장 레이아웃과의 차이를
    줄인다.
- `src/document_core/commands/object_ops.rs`
  - `samples/투명도0-50.hwp`에서 두 번째 TAC 그림 뒤 `split_paragraph_native(0, 0, 2)`를 수행한 뒤에도
    원래 문단에 두 그림 줄과 두 ImageNode가 모두 남는 회귀 테스트를 추가했다.

## 6. 검증 결과

- `cargo fmt --check` 통과
- `cargo test --lib issue1452_enter_after_second_tac_picture_keeps_both_pictures -- --nocapture` 통과
- `cargo test --lib issue1452 -- --nocapture` 통과
  - 10 passed
- `cargo test --lib issue1151 -- --nocapture` 통과
  - 3 passed
- `git diff --check` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
