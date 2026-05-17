# Task #960 Stage 5 — 시각 검증 + 최종 작업 정리

## 1. 시각 검증

- 시험지 page 2 문14 cases formula:
  - Before: y=329.6 (line 0 영역, header overlap)
  - After: y=352.0 (line 1 정상 위치, "정의한다." 옆) ✓ 한컴 PDF 정합
- 다른 페이지/sample 시각 회귀 0

## 2. 최종 변경 요약

### 2.1 코드 변경
`src/renderer/layout/paragraph_layout.rs` 라인 1719-1736:
- **Fix A**: `allow_end_tac` 변수 추가 — `is_last_run || (has_line_break && is_last_run_of_line)` 조건
- filter 의 end-position 허용 조건을 has_line_break line 까지 확장

### 2.2 문서 추가
- `mydocs/plans/task_m100_960.md`
- `mydocs/plans/task_m100_960_impl.md`
- `mydocs/plans/task_m100_960_impl_v2.md`
- `mydocs/working/task_m100_960_stage1.md`
- `mydocs/working/task_m100_960_stage2.md`
- `mydocs/working/task_m100_960_stage4.md`
- `mydocs/working/task_m100_960_stage5.md`
- `mydocs/report/task_m100_960_report.md`
- `mydocs/orders/20260517.md` 갱신

## 3. PR 구성

samples/pdfs 는 PR #956 / #961 와 중복이므로 **본 PR 에 포함 안 함** (소스 fix + 문서만).

- base = upstream/devel
- head = jangster77:local/task960
- title: fix: line break 직전 inline TAC control 의 line 매핑 정정 — 시험지 page 2 cases formula off-by-one 해소 (closes #960)
