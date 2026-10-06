# Task #1763 Stage 1 완료보고서 — 재현 샘플 동결 + 실패 재현

## 수행 내용
- `samples/task1763/cell_trailing_ls_expand.hwp` 동결 (2501937, 32KB) + README.
- 실패 재현: render-tree Table(pi=1, 12×16) 전체 h=524.7px vs 한글 517.5px (+7.2),
  전액 row0 (rhwp 149.1 vs 한글=선언 142.2px).
- 원인 코드 특정: `height_measurer.rs` 측정 closure —
  `include_trailing_ls = !is_cell_last_line || cell_para_count > 1` 이 다문단 셀의
  셀 마지막 줄 trailing ls(600HU=8px)를 포함 → required 149.1 > 선언 142.2 → 확장.
  줄 간격 자체는 저장 LINE_SEG 와 전부 일치(21.3/56.2/21.4/21.3px) — 측정 오버헤드만 결함.

## 상태
완료. Stage 2 (선언높이 권위 가드 + 테스트) 진행.
