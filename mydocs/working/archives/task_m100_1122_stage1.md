# Task M100-1122 단계 보고서

- 이슈: [#1122](https://github.com/edwardkim/rhwp/issues/1122)
- 브랜치: `local/task1122-equation-over-token`
- 대상: `samples/3-11월_실전_통합_2022.hwp` page 4
- 작성일: 2026-05-26

## 1. 재현과 원인

작업지시자 제공 캡처와 `dump-pages`/`export-svg` 결과를 비교했다.

- 공백이 있는 `{8} over {13}`, `{17} over {26}`는 기존 렌더에서 정상 분수로 표시됐다.
- 공백이 없는 `11 over20`, `3 over5`, `13 over20`, `7 over10`, `3 over4`는 SVG에 `over20`, `over5`, `over10`, `over4` 텍스트로 남았다.
- 원인은 `src/renderer/equation/tokenizer.rs::read_command()`가 ASCII 영문/숫자를 연속으로 읽어 `over20`을 `Command("over20")`으로 만드는 동작이었다.

따라서 HWP 레코드 추출이나 SVG fraction renderer 문제가 아니라, 수식 토큰화 단계의 `over`/`atop` 키워드 결합 분리 누락으로 판단했다.

## 2. 구현

`src/renderer/equation/tokenizer.rs`에 대소문자 무시 ASCII prefix 확인 헬퍼를 추가하고, `read_command()`에서 다음 경우만 좁게 분리하도록 했다.

- `over` 또는 `OVER` 뒤에 숫자가 바로 이어지는 경우
- `atop` 또는 `ATOP` 뒤에 숫자가 바로 이어지는 경우

회귀 방지를 위해 `overlap`, `overline`, `overset`, `\overline{AB}`는 기존처럼 하나의 식별자/명령어로 유지하는 테스트를 추가했다.

`src/renderer/equation/parser.rs`에는 `11 over20`, `3 over5`, `7 OVER10`, `{8} over {13}`가 모두 `EqNode::Fraction`으로 파싱되는 테스트를 추가했다.

## 3. 문 26) 그림 크기 추가 확인

작업지시자가 문 26)이 한컴오피스와 완전히 다르게 보인다고 추가 제보했고, 별도 이슈가 아니라 #1122의 현재 문제라고 확인했다.

`dump-pages` 기준 문 26) 주머니 그림은 `pi=223 ci=0`이며 다음 크기 정보를 가진다.

- `common=3365x9446 HU`
- `shape_attr.current=9014x9446 HU`

기존 SVG는 `common.width`를 사용해 `width="44.866..."`로 출력했고, 이 때문에 한컴오피스보다 그림이 매우 좁았다. `src/renderer/layout/utils.rs`에 `picture_display_size_hu()`를 추가해 `current_width/current_height`가 양수이고 `common`보다 큰 축은 현재 표시 크기를 쓰도록 했다.

적용 위치:

- `src/renderer/layout/picture_footnote.rs::layout_picture_full`
- `src/renderer/layout/picture_footnote.rs::layout_body_picture`
- `src/renderer/layout.rs`의 column 오른쪽 overflow skip 계산

수정 후 page 4 SVG에서 문 26) 주머니 그림은 `width="120.186..." height="125.946..."`로 출력되어 `current=9014x9446 HU` 기준 표시 크기를 사용한다.

## 4. 검증

다음 명령을 실행했다.

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

결과:

- 수식 토크나이저 Task #1122 테스트 2개 통과.
- 수식 파서 Task #1122 테스트 2개 통과.
- picture 표시 크기 유틸 테스트 2개 통과.
- `renderer::equation` 전체 테스트 121개 통과.
- `renderer::layout` 필터 테스트 130개 통과, 1개 ignored.
- `issue_505` 통합 테스트 9개 통과.
- `cargo build` 성공.
- 새 SVG에서 `over20`, `over5`, `over10`, `over4` 문자열 검색 결과 없음.
- 새 SVG에서 보기 ①~⑤의 `11/20`, `3/5`, `13/20`, `7/10`, `3/4`가 `<line>`을 포함한 분수 그룹으로 출력됨을 확인.
- 문 26) 주머니 그림이 `current_width/current_height` 기준 크기로 출력됨을 확인.

검증 중 표시된 Rust warning은 기존 경고로 보이며, 이번 변경 범위와 직접 관련 없는 항목이다.
