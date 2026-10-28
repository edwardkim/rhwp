# Task M100 #1443 Stage 14 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `34f8ea91 task 1443: 셀 안 여백 속성 표시와 렌더 보정`

## 1. 목표

`samples/셀보호2.hwp` 렌더링을 `pdf/셀보호2-2024.pdf` 한컴 PDF 기준과 전체 표 구조까지 맞춘다.

Stage 13에서는 셀 안 여백과 텍스트 줄바꿈을 보정했지만, 전체 표 grid 기준으로는 아직 다르다.

## 2. 발견된 차이

96dpi PNG 기준:

- 한컴 PDF 표 bbox: `x=117..693`, `y=452..691`, 크기 `577x240`.
- rhwp 표 bbox: `x=113..672`, `y=448..691`, 크기 `560x244`.

차이:

- rhwp가 표 외곽을 약 1mm만큼 왼쪽/위로 그린다.
- rhwp가 표 전체 폭을 한컴보다 약 17px 좁게 그린다.
- rhwp가 마지막 행 일부 셀 높이를 독립 적용해 아래선이 들쭉날쭉하다.

## 3. 원인 가설

- `Table.common.width=43190HU`, `Table.common.height=17932HU`가 한컴 PDF의 전체 표 외곽과 맞는다.
- 현재 렌더러는 열 폭을 셀 폭 제약으로만 풀어 `41950HU` 폭만 사용한다.
- `outer_margin_left/top=283HU`가 표 위치에 반영되지 않아 약 1mm만큼 왼쪽/위로 밀린다.
- 행 높이에서도 `Table.common.height` 기준의 전체 높이 보존이 빠져 마지막 행 아래선이 한컴과 다르다.

## 4. 수정 방향

- 표 렌더링의 외곽 위치는 한컴처럼 바깥 여백을 반영한다.
- 열 폭 해석은 병합 셀 제약을 만족하도록 보정한다.
- 최종 열 폭 합계는 가능하면 `Table.common.width`와 일치시킨다.
- 행 높이 합계도 가능하면 `Table.common.height`와 일치시킨다.
- 단, Stage 12의 Shift+개별 segment 리사이즈 동작은 깨지지 않도록 행/열별 독립 segment 처리를 보존한다.

## 5. 검증 계획

- `pdf/셀보호2-2024.pdf`를 `pdftoppm -png -r 96`으로 렌더링한다.
- `samples/셀보호2.hwp`를 rhwp SVG로 렌더링 후 `rsvg-convert`로 PNG 변환한다.
- 한컴 PDF/rhwp PNG의 표 bbox와 주요 수평/수직 선 좌표를 비교한다.
- 기존 회귀 테스트:
  - `cargo test --test issue_493_cell_attrs -- --nocapture`
  - `cargo test -p rhwp --lib renderer::layout::table_layout::row_cut_tests::test_shrink_cell_padding_preserves_explicit_cell_margin -- --nocapture`
  - `cd rhwp-studio && npx tsc --noEmit`

## 6. 구현 내용

- depth 0 표 위치 계산에서 바깥 여백을 포함한 외곽 박스를 기준으로 맞추고, 실제 border 좌표는 `outer_margin_left/top`만큼 안쪽으로 이동하도록 보정했다.
- 열 폭 계산에서 병합 셀 제약을 뒤쪽 열에 반영하고, 최종 열 폭 합계가 `Table.common.width`와 맞도록 마지막 열을 보정했다.
- 행 높이는 셀/컨텐츠 측정값이 `Table.common.height`와 어긋날 경우 아래 행부터 보정해 표 외곽 높이를 한컴과 맞췄다.
- `MeasuredTable` 경로도 동일한 행 높이 보정을 타도록 수정했다.
- 행/열별 독립 segment 렌더링은 전체 폭/높이가 표 외곽과 맞는 경우에만 유지해, 저장 파일의 보조 `cell.width/height` 때문에 표가 깨지는 상황을 피했다.

## 7. 시각 비교 결과

96dpi PNG 기준:

- 한컴 PDF 표 bbox: `x=117..693`, `y=452..691`, 크기 `577x240`.
- rhwp 표 bbox: `x=117..693`, `y=452..691`, 크기 `577x240`.
- 긴 수평선 좌표:
  - 한컴: `[452, 497, 543, 589, 635, 691]`
  - rhwp: `[452, 498, 543, 589, 635, 691]`
- 긴 수직선 좌표:
  - 한컴: `[117, 196, 229, 357, 469, 588, 693]`
  - rhwp: `[117, 196, 228, 357, 469, 588, 692]`

외곽 bbox는 완전 일치하고, 내부 선은 래스터 반올림 기준 0~1px 차이만 남는다.

생성 산출물:

- `output/poc/task_m100_1443_stage14/hancom_vs_rhwp_stage14_side_by_side.png`
- `output/poc/task_m100_1443_stage14/hancom_rhwp_stage14_diff_crop.png`

## 8. 검증 결과

- `cargo fmt`: 통과
- `git diff --check`: 통과
- `cargo test --test issue_493_cell_attrs -- --nocapture`: 통과, 7 passed
- `cargo test -p rhwp --lib renderer::layout::table_layout::row_cut_tests::test_shrink_cell_padding_preserves_explicit_cell_margin -- --nocapture`: 통과, 1 passed
- `cd rhwp-studio && npx tsc --noEmit`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
  - `wasm-bindgen` prebuilt 미지원으로 `cargo install` fallback 후 완료
