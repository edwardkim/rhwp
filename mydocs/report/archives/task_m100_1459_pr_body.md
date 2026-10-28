## 요약

Closes #1459

- 그림 속성에서 `글자처럼 취급`을 해제해 `TopAndBottom` 자리차지 그림으로 바꿀 때 같은 문단의 남은 TAC 그림이 한컴처럼 다시 흐르도록 보정했습니다.
- renderer composer가 비-TAC 그림/도형/표를 inline control slot으로 세지 않도록 수정했습니다.
- 실제 샘플에서 속성 변경 직후 render tree 순서와 간격을 검증하는 회귀 테스트를 추가했습니다.

## 주요 변경

- TAC 해제 경로
  - `treatAsChar=true -> false` 전환 감지
  - 텍스트 없는 그림 전용 문단의 `LineSeg`를 남은 TAC 개체 기준으로 재구성
  - 비-TAC `TopAndBottom` 예약 높이를 남은 TAC 줄의 `vertical_pos`에 반영
- 렌더러 inline 판정
  - 비-TAC `Picture`/`Shape`/`Table`은 inline control slot에서 제외
  - TAC 개체만 composer의 inline control과 marker 보정 대상으로 사용
- 테스트
  - `samples/투명도0-50.hwp`에서 첫 번째 그림의 TAC를 해제하는 실제 속성 변경 경로 검증
  - 기존 한컴 저장본 HWP/HWPX 혼합 그림 배치 검증 유지
  - 비-TAC 자리차지 그림이 커서 stop으로 잡히지 않는 회귀 검증 유지

## 검증

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
