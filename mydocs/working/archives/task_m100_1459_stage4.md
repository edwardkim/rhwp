# Task M100 #1459 Stage 4

- 작성일: 2026-06-22
- 모드: 기여자 모드. PR 전 오늘할일 문서는 생성하거나 갱신하지 않는다.
- 이전 커밋: `b694e64a task 1459: 비TAC 그림 커서 진입 제외`

## 목표

그림 속성에서 `글자처럼 취급`을 해제했을 때 같은 문단 안의 그림들이 한컴처럼 재배치되도록 한다.

## 사용자 관찰

- rhwp에서는 TAC 해제 후 렌더링 결과가 한컴 기준과 다르다.
- 선택한 그림은 더 이상 문자처럼 취급되지 않는 자리차지 개체가 되므로, 해당 그림 자체와 같은 문단의 남은 TAC 그림이 새 흐름 기준으로 다시 배치되어야 한다.
- 한컴 기준 스크린샷처럼 해제된 그림이 자리차지 개체로 먼저 배치되고, 남은 TAC 그림은 그 예약 영역 뒤에 이어져야 한다.

## 확인할 지점

1. `setPictureProperties`가 `treatAsChar=false`, `textWrap=TopAndBottom`, 가로/세로 기준을 모델에 정확히 반영하는지 확인한다.
2. 속성 변경 후 `LineSeg` 기반 TAC 그림 위치 계산이 이전 TAC 줄 좌표를 재사용하지 않는지 확인한다.
3. 비-TAC TopAndBottom 예약 영역과 TAC 그림 inline 위치가 같은 문단 안에서 일관된 순서로 계산되는지 확인한다.

## 검증 계획

- #1459 샘플에서 첫 번째 TAC 그림을 `treatAsChar=false + TopAndBottom`으로 변경한 뒤 render tree 순서와 y 좌표를 검증한다.
- 기존 #1459 혼합 문단 테스트를 유지한다.
- #1452 TAC 그림 커서 회귀 테스트를 재실행한다.

## 구현

- `set_picture_properties_native`에서 `treatAsChar=true -> false` 전환도 감지하도록 확장했다.
- 텍스트 없는 그림 전용 문단에서 TAC 해제 시, 비-TAC `TopAndBottom` 개체의 예약 높이를 남은 TAC 줄의 `vertical_pos`로 반영하고 남은 TAC 개체 수만큼 `LineSeg`를 재구성했다.
- renderer composer가 비-TAC 그림/도형/표를 inline 제어 문자 슬롯으로 세지 않도록 조정했다.
- 기존 composer 테스트의 표 inline 판정은 `treat_as_char=true` 조건을 명시하도록 수정했다.

## PR CI 보정

- PR #1460 CI의 `cargo test --verbose`에서 `issue_1198_exam_social_internal_paste_uses_nested_cell_path`가 실패했다.
- #1459에서 non-TAC 개체를 커서 탐색용 논리 inline control에서 제외하면서, 클립보드 range trimming이 `Paragraph::split_at()`에 넘길 offset까지 같은 기준으로 변환해 텍스트 앞 non-TAC 개체가 있는 nested cell 복사에서 첫 글자를 잃었다.
- 클립보드 trimming 전용 변환을 `Paragraph::split_at()`의 movable control 기준으로 분리해, cursor/nav 논리 offset 변경과 paragraph split offset 계산을 독립시켰다.

## 검증 결과

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --profile release-test tac_toggle_true_to_false_restores_empty_picture_para_line_seg -- --nocapture`
- `cargo test --profile release-test test_identify_inline_controls_table -- --nocapture`
- `cargo test --profile release-test --test issue_1459_topbottom_picture_reflow -- --nocapture`
- `cargo test --profile release-test --test issue_1452_saved_caret -- --nocapture`
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate issue_1293_equation_control_is_not_always_treat_as_char -- --nocapture`
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate issue_1139_endnote -- --nocapture`
- `cargo test --profile release-test --lib`
- `wasm-pack build --target web --out-dir pkg`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --test issue_1198_nested_cell_paste issue_1198_exam_social_internal_paste_uses_nested_cell_path -- --nocapture`
- `cargo test --test issue_1198_nested_cell_paste -- --nocapture`
- `cargo test --verbose`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --profile release-test --tests`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --doc`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`
