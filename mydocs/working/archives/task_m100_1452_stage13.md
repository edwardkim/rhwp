# Task M100 #1452 Stage 13 작업 기록

## 배경

Stage 12 커밋 후 전체 회귀 테스트를 이어서 수행하던 중 `cargo test --release --lib`에서 기존 TAC 인라인 표 분할 회귀 테스트가 실패했다.

## 재현

- 명령: `cargo test --release --lib`
- 실패 테스트: `wasm_api::tests::test_create_inline_tac_table`
- 단독 재현: `cargo test --release --lib wasm_api::tests::test_create_inline_tac_table -- --nocapture`
- 증상:
  - pi=1에 인라인 TAC 표 1개가 생성된 뒤 셀 텍스트를 채우고 문단 끝에서 Enter를 수행한다.
  - Enter 후 pi=1에 인라인 표가 남아야 하지만 `controls.len() == 0`이 되어 표가 새 문단 쪽으로 이동한다.

## 가설

- `Paragraph::split_at()`은 인라인 컨트롤을 포함한 logical offset 기준으로 분할하도록 변경되어 있다.
- 그러나 일부 호출 경로와 기존 테스트는 여전히 `para.text.chars().count()`를 "문단 끝" offset으로 넘긴다.
- 인라인 컨트롤이 포함된 문단에서는 text length와 logical paragraph length가 달라지므로, 텍스트 끝 offset이 문단 끝이 아니라 컨트롤 앞 위치로 해석될 수 있다.

## 목표

- 문단 끝 Enter 호출에서 인라인 TAC 표/그림이 의도치 않게 다음 문단으로 이동하지 않도록 보정한다.
- 두 TAC 그림 사이/앞/뒤 Enter의 logical offset 분할 동작은 유지한다.
- 전체 회귀 테스트를 재개해 실패 범위를 확인한다.

## 진행

- `test_create_inline_tac_table`는 `split_paragraph_native` 호출 offset을 `text.chars().count()` 대신 `logical_paragraph_length()`로 맞춰 현재 API 의미와 일치시켰다.
- 수정 후 단독 테스트와 `cargo test --release --lib`는 통과했다.
- `cargo test --profile release-test --tests` 컴파일 중 `tests/issue_516.rs`의 `ImageAttr` 테스트 리터럴이 새 `transparency` 필드를 채우지 않아 실패했다.
- `ImageAttr` 기본 투명도인 `transparency: 0`을 추가했다.
- 통합 테스트 재실행 중 `issue_1139_inline_picture_duplicate`의 미주 수식 cursor/selection rect 테스트 4개가 실패했다.
- 이전 커밋 `d5ffa5a5` 임시 worktree에서도 같은 테스트 파일이 동일한 4개 실패로 재현되어, 이번 Stage 12/13 변경으로 생긴 회귀는 아닌 것으로 분리했다.
- `cargo clippy --all-targets -- -D warnings`에서 `Paragraph::split_at()`의 `needless_bool_assign`가 발견되어 동등한 boolean 대입식으로 정리했다.
- 사용자가 기존 실패도 먼저 해결하라고 지시하여 clippy를 중지하고 `issue_1139_inline_picture_duplicate`를 우선 수정 대상으로 전환했다.
- 미주 수식-only 문단은 텍스트가 비어 있고 Equation control만 있으므로, 수식 좌/우 경계를 2칸 단위 virtual offset으로 다루도록 navigation과 cursor rect mapping을 보강했다.
- 선택 사각형 계산에서도 Equation 노드 좌/우를 cursor hit 후보로 사용하고, y 좌표는 캐럿 보정값이 아니라 실제 Equation bbox y를 사용하도록 보정했다.
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate` 통과 (85 passed).
- `cargo test --test issue_1452_saved_caret -- --nocapture` 통과 (8 passed).
- 전체 통합 테스트 재실행 중 `issue_1198_nested_cell_paste`의 내부 클립보드 붙여넣기 테스트가 실패했다.
- 원인은 `copy_selection_native`의 텍스트 범위 클리핑 helper가 텍스트 offset을 그대로 `Paragraph::split_at()`에 넘기던 점이었다. `split_at()`은 현재 인라인 컨트롤을 포함한 logical offset 기준이므로, 앞쪽에 TAC 표/도형이 있는 문단의 첫 글자 복사에서 클립보드 텍스트가 빈 문자열이 됐다.
- 클립보드 helper에서 `split_at()` 호출 직전에 텍스트 offset을 logical offset으로 변환하도록 보정했다.
- `cargo test --profile release-test --test issue_1198_nested_cell_paste -- --nocapture` 통과 (2 passed).
- `cargo fmt --check`, `git diff --check`, `cargo test --profile release-test --tests` 통과.
- `cargo clippy --all-targets -- -D warnings`는 사용자의 중지 지시에 따라 완료하지 않았으며, PR 준비 단계에서 재실행이 필요하다.
- `upstream/devel` 동기화 중 `rhwp-studio/src/main.ts` import 충돌은 로컬 폰트 import와 `userSettings` import를 모두 유지해 해결했다.
- rebase 후 `cargo build --release`, `cargo test --release --lib`, `cargo test --profile release-test --tests`, `cargo fmt --check`, `git diff --check` 통과.
- PR 준비 중 `cargo clippy --all-targets -- -D warnings`가 `manual_is_multiple_of`를 지적해 `char_offset.is_multiple_of(2)`로 동치 정리했다.
- 동치 정리 후 `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings` 통과.
