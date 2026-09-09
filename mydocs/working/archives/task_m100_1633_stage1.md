# Task M100 #1633 Stage 1 완료 보고서

## 범위

- rhwp-studio `셀 테두리/배경 > 대각선` 탭이 선택 셀의 실제 표시 상태를 초기값으로 반영하도록 보정했다.
- `cellzone` overlay가 적용된 셀에서는 셀 고유 `border_fill_id`보다 matching `TableZone.border_fill_id`를 UI 조회용 effective 값으로 우선한다.
- 대각선 선 종류/굵기 선택지와 미리 보기 렌더링을 보강했다.
- `대각선샘플` 첫 줄 두 번째 칸에서 한컴 PDF처럼 꺾인 대각선이 표시되도록 `Crooked=2` 보존과 렌더링을 보강했다.

## 변경 파일

- `src/document_core/commands/table_ops.rs`
- `rhwp-studio/src/ui/cell-border-bg-dialog.ts`
- `src/parser/hwpx/header.rs`
- `src/serializer/hwpx/header.rs`
- `src/renderer/layout/border_rendering.rs`
- `tests/issue_1623_cellzone_diagonal.rs`
- `samples/대각선샘플.hwp`
- `samples/대각선샘플.hwpx`
- `pdf/대각선샘플-2024.pdf`

## 검증

- `cargo test --lib parser::hwpx::header::tests::test_slash_crooked_preserves_two_bit_value`
- `cargo test --lib serializer::hwpx::header::tests::write_border_fill_preserves_slash_crooked_two_bit_value`
- `cargo test --lib renderer::layout::border_rendering::tests::render_slash_crooked_with_backslash_as_bent_backslash`
- `cargo test --test issue_1623_cellzone_diagonal`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`
- `cargo build --release --features native-skia`
- `cargo fmt --check`
- `git diff --check`

## 시각 확인

- `samples/대각선샘플.hwp`를 PNG로 다시 추출해 한컴 PDF 기준 PNG와 비교했다.
- 우하단 굵은 X 셀은 `cellzone` overlay의 `bf=11` 대각선 값이 UI 초기값과 미리 보기에 반영됐다.
- 첫 줄 두 번째 칸은 `Crooked=2`가 유지되어 꺾인 대각선으로 렌더링된다.

## 남은 작업

- 한컴처럼 여러 셀 선택일 때만 `셀 테두리/배경 - 하나의 셀처럼 적용` 메뉴를 활성화하는 정책을 다음 스테이지에서 처리한다.
