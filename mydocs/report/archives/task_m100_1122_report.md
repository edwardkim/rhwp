# Task M100-1122 최종 보고서

- 이슈: [#1122](https://github.com/edwardkim/rhwp/issues/1122)
- 브랜치: `local/task1124-hwpx-column-line` (#1124 통합 PR 브랜치)
- 대상 샘플: `samples/3-11월_실전_통합_2022.hwp`
- 대상 페이지: page 4 (0-index page 3)
- 작성일: 2026-05-26

## 1. 요약

`3-11월_실전_통합_2022.hwp` page 4에서 `11 over20`, `3 over5`처럼 `over` 뒤 분모 숫자가 붙은 수식이 분수로 표시되지 않고 `over20` 같은 텍스트로 렌더링되는 문제를 수정했다.

원인은 수식 토크나이저가 `over20`을 `over` + `20`으로 분리하지 않고 하나의 명령어로 읽는 동작이었다. `over`/`atop` 뒤에 숫자가 바로 붙는 경우만 분리하도록 좁게 정정해 `overline`, `overlap` 같은 기존 식별자/명령어 회귀를 막았다.

작업지시자가 추가로 지적한 문 26) 주머니 그림의 한컴오피스 대비 표시 차이도 같은 #1122 범위에서 처리했다. 해당 그림은 `common.width`보다 큰 `shape_attr.current_width`를 가지고 있었으나 기존 렌더는 `common.width`만 사용해 지나치게 좁게 표시했다. picture 표시 크기 선택 유틸을 추가해 `current`가 더 큰 축은 현재 표시 크기를 사용하도록 정정했다.

## 2. 변경 파일

- `src/renderer/equation/tokenizer.rs`
  - `over`/`atop` + 숫자 결합 토큰 분리 추가.
  - 대소문자 무시 ASCII prefix 확인 헬퍼 추가.
  - `overline`, `overlap`, `overset`, `\overline{AB}` 회귀 테스트 추가.
- `src/renderer/equation/parser.rs`
  - `11 over20`, `3 over5`, `7 OVER10`, `{8} over {13}`가 `EqNode::Fraction`으로 파싱되는 테스트 추가.
- `src/renderer/layout/utils.rs`
  - picture 표시 크기 선택 유틸 추가.
  - `current_width/current_height`가 더 큰 경우와 `common` 유지 경우 테스트 추가.
- `src/renderer/layout/picture_footnote.rs`
  - picture layout에서 표시 크기 유틸 사용.
- `src/renderer/layout.rs`
  - column 오른쪽 overflow skip 계산에서 picture 표시 크기 유틸 사용.
- `mydocs/orders/20260526.md`
  - Task #1122 진행 상태 기록.
- `mydocs/plans/task_m100_1122.md`
  - 수행 계획서 작성.
- `mydocs/plans/task_m100_1122_impl.md`
  - 구현 계획서 작성.
- `mydocs/working/task_m100_1122_stage1.md`
  - 재현, 구현, 검증 단계 보고서 작성.

## 3. 검증 결과

성공:

```bash
cargo test --lib renderer::equation::tokenizer::tests::test_task1122 -- --nocapture
cargo test --lib renderer::equation::parser::tests::test_task1122 -- --nocapture
cargo test --lib renderer::layout::utils::tests::picture_display_size -- --nocapture
cargo test --lib renderer::equation -- --nocapture
cargo test --lib renderer::layout -- --nocapture
cargo test --test issue_505 -- --nocapture
cargo build
./target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 3 --debug-overlay -o output/task1122_after
./target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 3 --debug-overlay -o output/task1122_after_q26
rg -n "over20|over5|over10|over4" output/task1122_after_q26/3-11월_실전_통합_2022_004.svg || true
```

확인 내용:

- `renderer::equation` 테스트 121개 통과.
- `renderer::layout` 필터 테스트 130개 통과, 1개 ignored.
- `issue_505` 테스트 9개 통과.
- picture 표시 크기 유틸 테스트 2개 통과.
- 새 SVG에서 `over20`, `over5`, `over10`, `over4` 문자열이 남지 않음.
- page 4 보기 수식 `11/20`, `3/5`, `13/20`, `7/10`, `3/4`가 분수선과 분모 텍스트를 가진 SVG 그룹으로 출력됨.
- 문 26) 주머니 그림이 `current_width/current_height` 기준인 `120.186...px x 125.946...px`로 출력됨.

## 4. 참고 사항

`gh issue edit 1122 --add-assignee @me`는 현재 로그인 계정 권한 부족으로 실패했다. 이슈 생성과 구현은 완료했으며, assignee 지정은 권한 있는 계정에서 별도 처리해야 한다.

기존 Rust warning 몇 건이 테스트 출력에 함께 표시됐지만 이번 변경과 직접 관련된 실패는 없었다.

2026-05-26 통합 PR 전 CI 확인:

```bash
git diff --check upstream/main..HEAD
cargo build --verbose
cargo test canvas_layer_tree_matches_legacy --lib --verbose
cargo check --target wasm32-unknown-unknown --lib
cargo test --features native-skia skia --lib --verbose
cargo test --verbose
cargo clippy -- -D warnings
```

위 항목은 모두 통과했다. 작업지시자 지시에 따라 #1122와 #1124는 한 PR로 묶되, 최종 PR 생성은 최종사용자 승인 후에만 수행한다.
