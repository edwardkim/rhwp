# Task #1765 Stage 1 완료보고서 — 재현 동결 + 초기 가설 확인

- `samples/task1765/merged_cell_trailing_ls.hwp` 동결 (17931383) + README.
- 실패 재현: p2 12×4 표 render-tree h=**1006.8px** vs 한글 find_tables 1001.6px (+5.2).
- 초기 가설: `height_measurer.rs` 병합 셀 경로(2-b) — rs=1 경로와 동일한
  `include_trailing_ls = !is_cell_last_line || cell_para_count > 1` 포함 측정 +
  `required > combined` 시 마지막 스팬 행 deficit 확장 (가드 부재).

## 상태
완료. Stage 2 에서 행높이 대조로 가설을 검증한다.
