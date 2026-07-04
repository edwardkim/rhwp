# Task M100 #1443 Stage 16 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `dcc9ee33 task 1443: 안 여백 텍스트 정합 확인`

## 1. 목표

셀 속성에서 `안 여백 지정`을 끈 경우 한컴처럼 셀 고유 padding 값을 렌더/리플로우에 적용하지 않도록 수정한다.

사용자가 새 샘플을 제공했고, 시각 검증에 사용했다. 검증 완료 후 해당 파일들은 회귀 테스트 fixture로 커밋하지 않고 제외한다.

- `samples/셀보호2-안여백지정off.hwp`
- `samples/셀보호2-안여백지정off.hwpx`
- `pdf/셀보호2-안여백지정off-2024.pdf`

## 2. 현상

`셀보호2.hwp`에서 마지막 행 첫 번째 셀은 좌우 10mm 안 여백이 켜진 상태다. rhwp Studio에서 해당 셀의 `안 여백 지정`을 끄면 UI의 체크박스는 꺼지지만, 셀 안 내용은 여전히 10mm 여백 기준처럼 좁은 폭으로 줄바꿈된다.

한컴에서는 `안 여백 지정`을 끈 뒤 저장한 파일을 다시 열면 다음처럼 동작한다.

- UI에는 왼쪽/오른쪽 10mm 값이 비활성 상태로 남아 있다.
- 렌더링은 해당 값을 적용하지 않고 표 기본 안 여백 기준으로 표시한다.
- `12345`가 한 줄로 표시된다.

## 3. 샘플 분석

`samples/셀보호2-안여백지정off.hwp` 덤프 결과:

- 대상 셀: 마지막 행 첫 번째 셀, cell index 20
- `apply_inner_margin=false`
- `padding=(2834,2834,0,0)` 값은 파일에 남아 있음
- 문단 line segment는 1개
- 텍스트는 `12345`

즉 `apply_inner_margin=false`일 때는 저장된 셀 padding 값을 보존하되, 레이아웃 계산에는 쓰지 않아야 한다.

## 4. 현재 코드 원인

다음 경로에 과거 휴리스틱이 남아 있다.

- `src/document_core/commands/text_editing.rs::reflow_cell_paragraph`
  - `apply_inner_margin=false`여도 `cell.padding > table.padding`이면 cell padding을 사용한다.
  - Studio에서 체크박스를 off로 바꿀 때 10mm 값이 남아 있으므로, reflow가 여전히 좁은 폭으로 계산된다.
- `src/renderer/layout/table_layout.rs::resolve_cell_padding`
  - 같은 휴리스틱으로 `apply_inner_margin=false`인 셀의 큰 padding 값을 렌더에 사용할 수 있다.
- `src/renderer/height_measurer.rs`
  - 일부 행 높이 측정 경로가 같은 휴리스틱을 사용한다.

Stage 13에서 `체크 해제는 padding 원값을 지우지 않고 적용 플래그만 끈다`는 동작은 맞지만, 그 원값을 레이아웃에 계속 쓰는 것은 새 off 샘플과 한컴 동작에 맞지 않는다.

## 5. 수정 계획

- `apply_inner_margin=true`
  - 셀 고유 padding을 사용한다.
  - 0mm도 명시값으로 존중한다.
- `apply_inner_margin=false`
  - 셀 padding 값을 보존한다.
  - 렌더링, 행 높이 측정, 편집 reflow 폭 계산에는 표 기본 padding을 사용한다.
- 회귀 테스트 추가:
  - on 샘플에서 `setCellProperties({"applyInnerMargin":false})`를 호출하면 padding 값은 보존되지만 대상 셀 문단은 한 줄로 reflow되는지 확인한다.

## 6. 검증 계획

- `cargo test --test issue_493_cell_attrs -- --nocapture`
- `cargo test -p rhwp --lib renderer::layout::table_layout::row_cut_tests::test_shrink_cell_padding_preserves_explicit_cell_margin -- --nocapture`
- `cargo fmt --check`
- `wasm-pack build --target web --out-dir pkg`
- 필요 시 off 샘플 PDF/PNG 확대 비교

## 7. 구현 내용

- `tests/issue_493_cell_attrs.rs`
  - on 샘플에서 `setCellProperties({"applyInnerMargin":false})` 호출 후 line segment가 한 줄로 재계산되는지 검증했다.
  - 별도 off 저장본을 테스트 fixture로 커밋하지 않도록, 기존 `셀보호2.hwp` 기반 토글 테스트로 회귀를 고정했다.
- `src/document_core/commands/text_editing.rs`
  - 셀 문단 reflow 폭 계산에서 `apply_inner_margin=false`이면 저장된 셀 padding 값이 커도 표 기본 padding을 사용하도록 수정했다.
- `src/renderer/layout/table_layout.rs`
  - `resolve_cell_padding`을 한컴 기준으로 단순화했다.
  - `apply_inner_margin=true`이면 cell padding을 사용하고, 0mm도 명시값으로 존중한다.
  - `apply_inner_margin=false`이면 cell padding 원값은 보존하지만 렌더링에는 table padding을 사용한다.
- `src/renderer/height_measurer.rs`, `src/renderer/layout/shape_layout.rs`
  - 행 높이/분할 측정 경로도 같은 padding 해석을 사용하도록 맞췄다.

## 8. 검증 결과

- `cargo test --test issue_493_cell_attrs -- --nocapture`: 통과, 8 passed
- `cargo test -p rhwp --lib renderer::layout::table_layout::row_cut_tests::test_shrink_cell_padding_preserves_explicit_cell_margin -- --nocapture`: 통과, 1 passed
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
  - `wasm-bindgen` prebuilt 미지원으로 `cargo install` fallback 후 완료
- off 샘플 PDF/PNG 비교:
  - 한컴 bbox: `117,452..693,684`
  - rhwp bbox: `117,452..693,685`
  - 확대 crop에서 마지막 행 첫 번째 셀의 `12345`가 한 줄로 표시됨
  - 생성 산출물: `output/poc/task_m100_1443_stage16_off_after/hancom_vs_rhwp_off_after_side_by_side.png`
  - 시각 검증 완료 후 `samples/셀보호2-안여백지정off.hwp`, `samples/셀보호2-안여백지정off.hwpx`, `pdf/셀보호2-안여백지정off-2024.pdf`는 커밋 대상에서 제외
