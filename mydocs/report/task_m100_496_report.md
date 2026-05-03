# Task #496 최종 보고서 — 보류 결정

**이슈**: [#496](https://github.com/edwardkim/rhwp/issues/496) — exam_science.hwp 2페이지 12번 본문 줄간격 압축
**브랜치**: `local/task496`
**상태**: **보류** (단계 1 완료 후 작업지시자 결정으로 보류)
**결정일**: 2026-04-30

---

## 1. 결과

본 task 는 **단계 1 (원인 정확 진단) 완료 후 보류** 결정. 코드 변경 없음.

## 2. 결함 본질 (단계 1 진단 결과)

### 분기 결정 — `layout::layout_column_item:1944`

pi=61 (12번 후반부 paragraph) 의 ctrl[0] 표 (2x1, tac=true, wrap=TopAndBottom) 가 `is_tac_table_inline=true` 로 판정되어 일반 `layout_paragraph` 가 아닌 `layout_inline_table_paragraph` 분기로 진입.

### `layout_inline_table_paragraph` 의 한계

- multi-row 표 + multi-line 텍스트 처리를 못함
- `wrapped_below_table` 플래그가 한 번만 줄바꿈 허용
- 표가 2 행이면 표 행 1 하단(=행 2 시작) 으로 텍스트 이동 → 표 행 2 와 시각적 겹침
- paragraph 의 line_segs[2] 무시

### 시각적 겹침 메커니즘

- 표 행 0 baseline = 1191.68
- paragraph 본문 baseline = 1195.85
- 4px 안에 두 텍스트 발행 → "줄간격 압축" 으로 보임

PDF 와 비교: 한컴은 표를 paragraph 본문과 **별도 블록** 으로 처리한 것으로 보임 (인라인 처리 X).

상세: [`mydocs/working/task_m100_496_stage1.md`](../working/task_m100_496_stage1.md)

## 3. 보류 사유

본 결함은 **inline tac 컨트롤 처리의 본질적 한계**. 단독 수정으로 안전 해결 어려움.

### 검토된 수정 방향 4가지

| 후보 | 내용 | 위험 |
|---|---|---|
| (A) | `layout_inline_table_paragraph` multi-row 표 보강 | 변경 범위 큼, 인라인 표 케이스 회귀 위험 |
| (B) | `is_tac_table_inline` 에서 multi-row 표 제외 → block 처리 | 룰 변경, 광범위 회귀 (PDF 와 가장 정합) |
| (C) | ls[i].vpos 직접 사용 + paragraph_layout 통합 | 코드 통합 필요, 중간 위험 |
| **(D)** | **보류 — 향후 layout 리팩터링 시 종합 해결** | 결함 잔존, 가장 안전 |

### 보류 결정 근거

1. **메모리 `feedback_essential_fix_regression_risk.md`**: layout 본질 정정은 회귀 위험 큼. 광범위 샘플 + 한컴 2010/2020 검증 필요.
2. **#495 단계 3 v1 회귀 학습**: 단일 케이스 가설 일반화는 회귀 발생.
3. **#500 (사각형 위치 결함) 과 본질 동일**: 둘 다 inline tac 컨트롤 + multi-line/multi-row 처리 한계. 별개 task 로 단편 수정하기보다 **layout 리팩터링 시 종합 해결** 이 안전.
4. **시각적 영향 한정**: 본 결함은 12번 본문 마지막 부분의 한정된 영역에 발생. 다른 페이지/문서로의 직접 확산 없음.

## 4. 향후 처리

- 본 issue (#496) 는 **open 유지** — close 하지 않음
- 향후 layout 리팩터링 (특히 inline tac 컨트롤 처리 통합) 시 함께 검토
- 동일 패턴 발견 시 본 issue 에 추가 케이스 등록

## 5. 산출물

- 수행계획서: `mydocs/plans/task_m100_496.md`
- 단계 1 보고서: `mydocs/working/task_m100_496_stage1.md` (원인 진단 상세)
- 본 보고서 (보류 결정): `mydocs/report/task_m100_496_report.md`
- 코드 변경: **없음** (진단용 임시 코드 모두 제거)

## 6. 관련 이슈

- **#495** (closed): exam_science p2 7번 박스 텍스트 중복 — 같은 페이지의 다른 결함, 이미 처리 완료
- **#500** (open): exam_science p2 사각형 위치 결함 — #495 처리 중 발견된 별개 결함, 본 task 와 동일 본질 (inline tac 처리 한계). 함께 layout 리팩터링 대상
