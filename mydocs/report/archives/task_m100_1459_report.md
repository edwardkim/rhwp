# Task M100 #1459 작업 보고서

- 이슈: #1459 `rhwp-studio: 자리차지 그림 속성 변경 후 같은 문단 글자처럼 취급 그림 재배치 누락`
- 기준 브랜치: `upstream/devel`
- 작업 브랜치: `local/task_m100_1459`
- 작성일: 2026-06-22
- 작업 모드: 기여자 모드. 오늘할일 문서는 생성하거나 갱신하지 않았다.

## 요약

같은 빈 문단에 있는 그림 중 하나의 `글자처럼 취급`을 해제해 `TopAndBottom` 자리차지 그림으로 바꿀 때, rhwp가 기존 TAC 줄 정보를 그대로 사용해 한컴과 다른 위치로 렌더링되는 문제를 보정했다.

## 주요 변경

- `set_picture_properties_native`에서 `treatAsChar=true -> false` 전환을 감지한다.
- 텍스트 없는 그림 전용 문단에서 TAC 해제 시 `LineSeg`를 남은 TAC 개체 기준으로 재구성한다.
- 비-TAC `TopAndBottom` 개체의 예약 높이를 남은 TAC 줄의 `vertical_pos`에 반영한다.
- renderer composer가 비-TAC 그림/도형/표를 inline control slot으로 세지 않도록 수정했다.
- `samples/투명도0-50.hwp`에서 첫 번째 TAC 그림을 자리차지 그림으로 바꾸는 실제 속성 변경 경로 회귀 테스트를 추가했다.
- 기존 한컴 저장본 `samples/투명도0-50-2nd그림글차처럼off.{hwp,hwpx}`의 혼합 그림 배치와 커서 진입 제외 테스트를 유지했다.

## 커밋

- `7e0b2efe task 1459: 자리차지 그림 혼합 문단 렌더 보정`
- `5f9df1ad task 1459: TopAndBottom TAC 간격 중복 보정`
- `b694e64a task 1459: 비TAC 그림 커서 진입 제외`
- `ebad53a0 task 1459: TAC 해제 그림 재흐름 보정`

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

## 남은 절차

- 원격 push와 PR 생성은 사용자 승인 후 진행한다.
- PR 생성 전 전체 macOS 검증을 더 엄격하게 맞추려면 `cargo build --release`, `cargo test --release --lib`, `cargo test --profile release-test --tests`를 추가 실행한다.
