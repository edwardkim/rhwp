---
kind: review
status: accepted_with_maintainer_correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6314 검토 - body frame float exclusion

- PR: [#6314](https://github.com/edwardkim/rhwp/pull/6314)
- 이슈: [#6175](https://github.com/edwardkim/rhwp/issues/6175)
- 작성자: `@planet6897`
- 원 source head: `cede21be468ed0143c4d162f41960a238a07be77`
- 누적 검토 적용: 원 PR `10e1d9228` + 메인터너 보정 `bf5141aa6`

## 변경 검토

이 PR은 body frame float exclusion을 계산하고 #6175 HWPX fixture, 회귀 테스트, before/after report를
추가한다. `#6309`보다 실제 float width를 활용하려는 방향은 타당하지만, 현재 evidence 범위가
너무 넓다.

## 발견 사항

### P1 - 섹션 전체 float 폭이 관련 없는 문단의 stored row를 보존함

`src/renderer/float_placement.rs`의 `column_float_carve_widths`는 섹션의 모든 paragraph를 순회해
square/tight/through float의 폭만 수집한다. 그 결과에는 page, column, 세로 band, float alignment,
그리고 대상 paragraph와의 겹침 관계가 없다. 이 전역 폭 목록이 `TypesetEngine`의
`stored_rows_require_external_geometry`로 전달되므로, 같은 폭 차이를 가진 일반 문단 또는 table inset도
섹션의 무관한 float 때문에 외부 carve로 판단될 수 있다.

float footprint를 대상 paragraph의 page/column과 세로 overlap까지 연결하거나, 해당 위치에서 계산된
placement evidence만 전달하도록 보정해야 한다. 폭 일치만으로는 회귀 방지 근거가 되지 않는다.

## 통합 및 검증 상태

- 원 source head의 GitHub required CI는 성공했다.
- 원 PR의 폭 전용 판정은 P1로 보류했으며, 이 위험은 contributor 변경과 분리한
  메인터너 보정으로 해결했다.
- `regression_suite_022`의 #6175 focused 회귀 3건이 모두 성공했다.

## 2026-08-28 메인터너 보정 및 검증 증적

## 보정 범위

- 기존 보류 사유였던 폭 전용 float carve 판단을, Paper/Page `Square` float의 세로 영향 구간과
  실제 문단 행이 겹치는 경우에만 유지하도록 메인터너 보정했다.
- 최상위 Paper/Page float도 한글 권위 PDF와 대조할 수 있도록
  `tools/object_visual_regression.py`에 `--reference-pdf` 및 `Image` 노드 수집을 추가했다.

## 실행 결과

- `CARGO_TARGET_DIR=target/pr-review-planet6897-6304-6305-6309-6314-6316 cargo build`:
  성공.
- HWP2024 MCP `--engine 2020` 산출 PDF를 `--reference-pdf`로 분석했다.
- issue #6175 대상 Paper float는 rhwp `p1 (431.5, 386.4) 282.8x205.3`, 한글 PDF
  `p1 (431.1, 386.0) 282.7x205.1`로 매칭됐다. 위치 차이는 `x/y +0.4px`, 크기 차이는
  `+0.1/+0.2px`다.
- 증적: `output/pr_6314_issue6175_object_visual_regression_mcp_top_level_20260828/objects.tsv`,
  `gallery.html`, `hwp_ref.pdf`.

## 제한

- 글꼴 외형은 macOS 대체 글꼴 영향이 있으므로 이 대조로 동일성을 판정하지 않았다.
- `native-skia` 미빌드로 gallery의 rhwp PNG crop은 생성하지 못했다. SVG/PDF 페이지와
  geometry 대조만 수행했다.
- focused Rust: `regression_suite_022`의 다음 세 회귀가 모두 성공했다.
  - `stored_rows_are_not_dropped_by_reflow`
  - `square_float_outside_stored_row_band_does_not_preserve_narrow_rows`
  - `paragraph_beside_square_float_keeps_its_stored_narrow_width`

## 2026-08-28 native-skia gallery 완료

- `cargo build --release --bin rhwp --features native-skia` 성공 후
  `object_visual_regression.py --reference-pdf ... --rhwp-png`를 재실행했다.
- rhwp와 HWP2024 MCP PDF의 페이지 PNG 및 매칭 image crop을 모두 생성했다.
- issue #6175 대상 float crop(`rhwp_image_2.png`, `hwp_image_2.png`)은 앞선 bbox 대조와
  일치하게 본문 대비 위치·크기가 동일하다.
- 최종 gallery: `output/pr_6314_issue6175_object_visual_regression_mcp_native_skia_crops_20260828/gallery.html`.

## 최종 판정 - 메인터너 보정 후 수용

기존 P1의 폭 전용 오판정은 `FloatCarveEvidence`의 폭·세로 band 일치 조건으로 제한했다.
Paper/Page float의 geometry 대조, native-skia image crop, 그리고 focused Rust 회귀 3건이 모두
성공했으므로 `#6314`는 이 통합 묶음에서 **메인터너 보정 후 수용**한다.
