# Task #1763 구현계획서 — 셀 선언높이 권위 가드 (3단계)

수행계획서: `mydocs/plans/task_m100_1763.md`

## Stage 1 — 재현 샘플 동결 + 실패 재현
- `samples/task1763/cell_trailing_ls_expand.hwp` (2501937) + README.
- render-tree row0 149.1px / 선언 142.2px 기록.

## Stage 2 — 수정 + 테스트
- `height_measurer.rs` 측정 closure 에서 셀 마지막 줄 trailing ls 포함분
  (`cell_trailing_ls`) 추적 → required 판정에 가드:
  `content - cell_trailing_ls + pad ≤ 선언높이` 이면 required = 선언높이 (확장 억제).
- 단위테스트(측정) + 통합테스트(render-tree row0 높이).

## Stage 3 — 회귀 검증 + 최종보고
- lib/통합/페이지 게이트 9종, #1759 하니스 대상 표 재측정, 한글 OLE 대조
  (mismatch 잔여 + MATCH 150) → 보고서 → squash → PR.
