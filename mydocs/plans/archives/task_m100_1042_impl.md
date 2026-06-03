# Task #1042 구현 계획서 — multi-fixture alignment 정합 stage 별 진행

**Issue**: [#1042 HWP3→HWP5 multi-fixture paragraph alignment 정합](https://github.com/edwardkim/rhwp/issues/1042)
**Branch**: `local/task1042`
**Scope**: 본 task 내에서 4 sub-task 를 stage 별 진행 (새 issue 등록 없음)

---

## 1. Stage 구성 — 우선순위 별

### Stage 1 — 진단 자료 commit (현재 상태)

- ✅ 모든 fix code revert (parser/mod.rs, paragraph_layout.rs)
- ✅ 진단 test 정리 (tests/diag_1042_*)
- ✅ 이슈 #1042 title/body 정정
- ✅ 수행/구현 계획서 정정
- ⏳ 진단 보고서 작성 (`mydocs/working/task_m100_1042_stage1.md`)
- ⏳ Stage 1 commit

### Stage 2 — paragraph 드래그 선택 정합 (sub-task 1, 우선순위 높음)

- 근본 원인: composer fallback `segment_width=0` → paragraph_layout hit-test 영역 부정확
- Fix 영역: composer fallback 의 ComposedLine.segment_width 보정 (paragraph 의 column_width 활용) 또는 paragraph_layout 의 fallback 영역 보정
- 회귀 sweep: 5 fixture + 변환본 9 + 일반 fixture 페이지 수 무변동
- 시각 검증: paragraph 드래그 선택 정확도

### Stage 3 — k-water-rfp +2 over-split 해소 (sub-task 2, 우선순위 중상)

- 근본 원인: paragraph 별 +0.5~+1 px 누적 차이 (corrected_line_height / baseline_distance 미세)
- Fix 영역: paragraph_layout 의 height 측정 정밀화 (PR #1036 style narrow guard 적용 가능성)
- 회귀 sweep: k-water-rfp 27 정합 + sample16-hwp5 64 유지

### Stage 4 — sample16-hwp5-2022 의 +1 baseline 회귀 해소 (sub-task 3, 우선순위 중)

- 근본 원인: paragraph data 동일이지만 section_def/header/version flag 차이 추정
- Fix 영역: 2022 binary diff 진단 + 정합화

### Stage 5 — p23 외곽선 overflow 해소 (sub-task 4, 우선순위 중)

- 근본 원인: paginator + paragraph_layout 의 PartialParagraph 처리
- Fix 영역: PartialParagraph 분할 + paragraph_layout 동기화

### Stage 6 — 최종 보고서 + PR

- 4 stage 결과 종합 보고서 (`mydocs/report/task_m100_1042_report.md`)
- 회귀 가드 test 추가 (각 stage 별)
- PR 생성 (stacked on PR #1036/#1040, base devel, closes #1042)

---

## 2. Stage 별 위험성 + 완화

| Stage | 위험 | 완화 |
|-------|------|------|
| 2 | 드래그 선택 hit-test 정합 시 다른 paragraph 영향 | composer fallback 한정 가드 + 5 fixture 시각 검증 |
| 3 | paragraph height 정밀화 시 다른 fixture 회귀 | narrow guard (variant 한정 또는 fixture pattern) + 회귀 sweep |
| 4 | 2022 의 root cause 식별 어려움 | binary diff + 단독 진단 |
| 5 | PartialParagraph 처리 광범위 영향 | paginator path 정밀 추적 |

---

## 3. Stage 진행 결정 (각 stage 별 작업지시자 승인)

각 Stage 완료 시 작업지시자에게 진행 결정 요청:
- 진단 결과 + 회귀 sweep 결과 + 시각 검증 단계
- 승인 시 다음 stage 진행
- 미달 시 본 stage 폐기 + 다음 stage 또는 보류

---

## 4. 본 task 의 fix 범위

본 task 의 시간 누적 매우 큼. 모든 stage 완료 보장 어려움. **각 stage 진행 후 작업지시자 결정으로 종료 가능**:
- Stage 1 commit 후 종료 (진단 자료만, 가장 안전)
- Stage 2 후 종료 (paragraph 선택 fix)
- Stage 3 후 종료 (paragraph height 정밀화 추가)
- Stage 4~5 후 종료 (모든 sub-task 완료)

---

## 5. 결정 요청

본 구현 계획서 승인 시:
1. Stage 1 진행 (진단 자료 commit, 즉시)
2. Stage 2~5 는 각 stage 완료 후 작업지시자 결정으로 진행

승인 또는 scope 조정 권고.
