# #3128 Stage 2 — 구조 보정 구현

- **Issue**: #3128
- **기록일**: 2026-08-18 KST
- **성격**: 소급 완료 기록

> 이 Stage의 코드도 계획 승인 전에 작성됐다. 이 문서는 현재 로컬 diff를 설명하며, 승인 이력을
> 만들어내지 않는다.

## 1. Composer 보정

- 일반 `recompose_for_cell_width`의 기존 동작을 유지했다.
- 구조 gate를 통과한 child만 사용하는
  `recompose_for_cell_width_with_indented_tracking` opt-in 경로를 추가했다.
- 동일 font metric 안의 `CharShapeRef` 경계를 다시 run으로 나누고 구간별 tracking을 복원했다.
- literal ASCII 공백을 반각 advance로 재측정했다.
- HWPUNIT subpixel과 embedded-font integer advance의 누적 차이는 wrap edge에서 최대 1px만 허용했다.
- legacy bullet regenerated-space 경로와 새 tracking 복원을 상호 배타적으로 유지했다.

## 2. Table layout 보정

- `long_indented_tracking_uses_table_content_box`가 #3128 구조 조건을 한 곳에서 판정한다.
- 일반 render, mixed fragment 측정, partial/lazy compose가 모두 같은 gate를 사용한다.
- gate가 참인 child만 저장 small margin 호환 경로에서 제외해 table content box 전체를 사용한다.
- native terminal RowBreak child가 source cursor를 소유하면 generic first-unit reservation을 생략한다.

## 3. 회귀 계약 정정

기존 #2308 테스트의 p34 426.9px 높이와 saved-margin 보존 설명은 HWP 2024 PDF를 다시 대조한 결과
stale expectation이었다. continuation child 높이를 370.9px로, 우측 viewport를 table content-box
edge로 정정했다. p81→p82 short RowBreak child의 `… 사고` / `를 예방…` owner 계약은 유지했다.

## 4. 변경 범위

- `src/renderer/composer.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`
- `tests/issue_2308_render_normalized_derived_state.rs`
- `tests/cases/issue_3128_terminal_nested_table_geometry.rs`

## 5. Stage 결과

최종 로컬 render-tree에서 p34 outer continuation은 y=75.6px, h=389.1px, 내부 child는
y=77.1px, h=370.9px, 후속 직접편익 표는 y=508.6px이다. PDF raster 좌표와 stroke/clip 외곽 차이를
고려한 수용 오차 안에 들며 약 60px의 후속 흐름 하강이 제거됐다.
