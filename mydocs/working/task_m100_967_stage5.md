# Task #967 Stage 5 — 시각 검증 + 최종 작업 정리

## 1. 시각 검증

- sample18.hwp 페이지 수: 69 → 67 ✓ 한컴 viewer 정합
- 다른 sample 페이지 수 회귀 0
- hwp-multi-001 회귀 차단 보존

## 2. 최종 변경 요약

### 2.1 코드 변경
`src/renderer/typeset.rs:584-604` (next_will_vpos_reset 가드 직후 별도 분기):
- 빈 paragraph + 다음 force_page_break (쪽나누기) case 추가 catch
- 기존 next_will_vpos_reset 의 next_force_break 제외 (hwp-multi-001 회귀 차단) 보존

### 2.2 문서 추가
- `mydocs/plans/task_m100_967.md`
- `mydocs/plans/task_m100_967_impl.md`
- `mydocs/plans/task_m100_967_impl_v2.md`
- `mydocs/working/task_m100_967_stage1.md`
- `mydocs/working/task_m100_967_stage4.md`
- `mydocs/working/task_m100_967_stage5.md`
- `mydocs/report/task_m100_967_report.md`
- `mydocs/orders/20260518.md` 갱신

## 3. PR 구성

- base = upstream/devel
- head = jangster77:local/task967
- 변경 단일 파일 (typeset.rs)
