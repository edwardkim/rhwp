# Task M100-1124 단계 보고서

- 이슈: [#1124](https://github.com/edwardkim/rhwp/issues/1124)
- 브랜치: `local/task1124-hwpx-column-line`
- 대상: `samples/3-11월_실전_통합_2022.hwpx` page 4
- 작성일: 2026-05-26

## 1. 재현과 원인

작업지시자 제공 캡처와 HWP/HWPX SVG를 비교했다.

- HWP 렌더 SVG에는 좌우 단 사이 `x=396.853...` 위치의 세로 구분선 `<line>`이 출력된다.
- HWPX 렌더 SVG에는 같은 위치의 세로 구분선이 없었다.
- HWPX 원문 `Contents/section0.xml`에는 `<hp:colLine type="SOLID" width="0.12 mm" color="#000000"/>`가 존재했다.
- 원인은 `src/parser/hwpx/section.rs::parse_col_pr()`가 `colPr` 시작 태그의 속성만 읽고 내부 자식 `colLine`을 파싱하지 않는 동작이었다.

따라서 렌더러의 단 구분선 출력 문제가 아니라, HWPX 파서가 `ColumnDef.separator_type/width/color`를 채우지 못하는 문제로 판단했다.

## 2. 구현

`src/parser/hwpx/section.rs`에서 다음을 구현했다.

- `parse_col_pr_with_children()` 추가.
- `hp:colPr` Start 이벤트에서 내부 `hp:colLine`을 소비하고 단 구분선 속성으로 반영.
- `type`, `width`, `color`를 각각 `separator_type`, `separator_width`, `separator_color`로 매핑.
- 기존 Empty `colPr`는 속성만 읽는 경로를 유지.
- `parse_ctrl()`의 Start `colPr` 경로는 새 파서가 종료 태그 소비까지 책임지도록 중복 `skip_element()`를 제거.

회귀 방지를 위해 다음 테스트를 추가했다.

- `test_task1124_col_pr_parses_col_line`
- `test_task1124_col_line_type_and_width_mapping`

## 3. 검증

현재까지 실행한 명령:

```bash
cargo test --lib parser::hwpx::section::tests::test_task1124 -- --nocapture
cargo test --lib parser::hwpx -- --nocapture
cargo build
./target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 3 --debug-overlay -o output/task1124_hwpx_after
rg -n 'x1="396\.85333333333335"' output/task1124_hwpx_after/3-11월_실전_통합_2022_004.svg
cargo test --lib renderer::layout -- --nocapture
cargo test --test hwpx_roundtrip_integration -- --nocapture
git diff --check
```

결과:

- #1124 전용 테스트 2개 통과.
- `parser::hwpx` 필터 테스트 29개 통과.
- `cargo build` 성공.
- HWPX page 4 SVG에 `x=396.853...` 단 구분선 2개가 생성됨.
- `renderer::layout` 필터 테스트 119개 통과, 1개 ignored.
- `hwpx_roundtrip_integration` 17개 통과.
- `git diff --check` 통과.

기존 Rust warning 몇 건이 테스트 출력에 함께 표시됐지만 이번 변경과 직접 관련된 실패는 없었다.
