# Task #467 최종 보고서 — 보류 (Defer)

**이슈**: #467 — 확장 바탕쪽: 다른 apply_to 조합 (active=Odd + ext=Both 등) 동작 미검증
**브랜치**: `local/task467`
**상태**: **보류 (Stage 1 조사 후 결정)**

## 1. 결과

Stage 1 조사 완료, 후속 단계 보류.

상세 조사 내용은 `mydocs/working/task_m100_467_stage1.md` 참고.

## 2. 핵심 발견

| 항목 | 내용 |
|------|------|
| 다른 apply_to 조합 샘플 | `samples/exam_science.hwp` (master[3] Even ext + master[4] Both ext overlap) |
| 페이지 4 결함 | PDF 좌측 상단 "4(화1)" 표시, SVG 미표시 |
| 결함 원인 식별 | 어려움 (다른 apply_to 조합 처리 / master BehindText 표 렌더링 / 또 다른 결함 — 식별 불가) |

## 3. 보류 사유

1. 본 task 는 본질적으로 **조사 task** — 명확한 코드 수정 방향 없음
2. master 처리 변경은 회귀 위험 큼 (메모리 `feedback_essential_fix_regression_risk.md`)
3. 식별된 결함 ("4(화1)" 누락) 이 본 task 범위 (다른 apply_to 조합 처리) 와 직접 연관 모호
4. PDF/SVG 직접 비교 신뢰도 낮음 (메모리 `feedback_pdf_not_authoritative.md`)
5. 한컴 2010/2020 환경 직접 검증 어려움

## 4. 처리

- 코드 변경: 없음
- 이슈 close (보류, 향후 종합 해결)

## 5. 후속 가능성

향후 layout 리팩터링 또는 한컴 환경 직접 검증 가능 시점에 종합 처리. 페이지 4 좌측 상단 "4(화1)" 누락은 별도 결함으로 식별 시 새 issue 생성 권고 (master 컨트롤 BehindText 표 렌더링 또는 다른 결함).
