# Task M100-1124 최종 보고서

- 이슈: [#1124](https://github.com/edwardkim/rhwp/issues/1124)
- 브랜치: `local/task1124-hwpx-column-line`
- 대상 샘플: `samples/3-11월_실전_통합_2022.hwpx`
- 대상 페이지: page 4 (0-index page 3)
- 작성일: 2026-05-26

## 1. 요약

HWPX 렌더에서 다단 세로 구분선이 표시되지 않는 문제를 수정했다.

HWPX 원문에는 `hp:colPr` 내부에 `hp:colLine`이 존재했지만, 기존 HWPX 파서는 `colPr`의 속성만 읽고 자식 `colLine`을 무시했다. 그 결과 `ColumnDef.separator_type`이 0으로 남아 렌더러가 단 구분선을 그리지 않았다.

`hp:colLine`의 `type`, `width`, `color`를 `ColumnDef.separator_type`, `separator_width`, `separator_color`에 반영하도록 파서를 보강했다.

## 2. 변경 파일

- `src/parser/hwpx/section.rs`
  - `parse_col_pr_with_children()` 추가.
  - `hp:colLine` 속성 파싱 추가.
  - Start `colPr` 처리 시 자식 요소까지 소비하도록 수정.
  - #1124 회귀 테스트 2개 추가.
- `mydocs/orders/20260526.md`
  - Task #1124 진행 상태 기록.
- `mydocs/plans/task_m100_1124.md`
  - 수행 계획서 작성.
- `mydocs/plans/task_m100_1124_impl.md`
  - 구현 계획서 작성.
- `mydocs/working/task_m100_1124_stage1.md`
  - 재현, 구현, 검증 단계 보고서 작성.

## 3. 검증 결과

성공:

```bash
cargo test --lib parser::hwpx::section::tests::test_task1124 -- --nocapture
cargo test --lib parser::hwpx -- --nocapture
cargo build
./target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 3 --debug-overlay -o output/task1124_hwpx_after
rg -n 'x1="396\.85333333333335"' output/task1124_hwpx_after/3-11월_실전_통합_2022_004.svg
cargo test --lib renderer::layout -- --nocapture
cargo test --test hwpx_roundtrip_integration -- --nocapture
git diff --check
git diff --check upstream/main..HEAD
cargo build --verbose
cargo test canvas_layer_tree_matches_legacy --lib --verbose
cargo check --target wasm32-unknown-unknown --lib
cargo test --features native-skia skia --lib --verbose
cargo test --verbose
cargo clippy -- -D warnings
```

확인 내용:

- #1124 전용 테스트 2개 통과.
- `parser::hwpx` 필터 테스트 29개 통과.
- `renderer::layout` 필터 테스트 119개 통과, 1개 ignored.
- `hwpx_roundtrip_integration` 17개 통과.
- HWPX page 4 SVG에 HWP와 같은 `x=396.853...` 단 구분선이 생성됨.
- 통합 PR 전 CI 항목 전체 통과.

## 4. 참고 사항

사용자 지시에 따라 #1122와 #1124는 한 PR로 묶어 진행한다. 단, 최종 PR 생성은 최종사용자 승인 후에만 수행한다.
