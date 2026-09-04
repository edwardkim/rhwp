# 구현계획서 — #2063 초대형 표 성능 + 과분할

브랜치 `fix/2063-cellunits-quadratic-scan` (devel base). 승인: 성능+과분할 한 타스크, 독립, 자동승인.

## 단계

### Stage 1 — 성능(O(n²) 제거) [산출 불변]
- `cell_units_uncached`(table_layout.rs:4627)의 표-불변량 `has_visible_text_with_nested_table`를
  표 단위 1회 계산으로 hoist. 방식: 표 포인터 키 메모이즈 필드(`table_flag_cache`) or 호출부
  `cell_units`에서 1회 계산해 인자 전달. 침습 최소안 채택.
- 2차 후보(`row_has_prior_rowspan_cover` L6207, `table_partial.rs:208`) 프로파일 후 유의 시 동일 hoist.
- 검증: 21914299 dump-pages/export-pdf 벽시계 before/after, **페이지 213 불변**, lib 테스트·표통합·
  golden_svg 스냅샷·renderer_baseline 무회귀. 커밋(성능 전용).

### Stage 2 — 과분할(+51) 진단
- 성능 픽스로 빠른 렌더 확보 후: `RHWP_TABLE_DRIFT=1` dump-pages로 rhwp cut_row_h 추출 +
  `tools/hangul_row_heights.py`로 한글 per-row 높이 추출 → 행높이 드리프트 정량화.
- rhwp row_h > 한글 row_h(초과분 = 페이지당 행수 25 vs 32)의 원인 특정
  (content+pad 과대측정 / cell.height 무시 / padding / line height).

### Stage 3 — 과분할 수정 [correctness]
- 진단 결과에 따라 CellBreak 행높이 측정 교정(한글 정합). #1937/#1842/#1658 계열과 정합 유지.
- 검증: 21914299 213→162 근접, 오라클 pi-page 재검(정렬 확인), 표통합·스냅샷·랜덤250 무회귀.
- 성능 커밋과 분리된 별도 커밋(기능 변경).

### Stage 4 — 최종 검증 + 보고
- 전체 재빌드, 게이트(rt/rd) 21914299 타임아웃 해소 확인, clippy, fmt(수정파일).
- `_stage{N}` 보고 + `_report`. Fork PR → devel.

## 무회귀 게이트
lib `cargo test`, 표 통합(1488/1749/2015 등), golden_svg 스냅샷, renderer_baseline, 랜덤250 render-diff.
