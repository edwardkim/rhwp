# Task #1594 — Stage 3 완료보고서 (게이트 + 통제 비교)

**단계**: diff_documents 게이트 추가 + 통제 비교 · **브랜치**: `local/task1594`

## 1. 게이트 추가
`ObjectHoldAnchor` IrDifference + `diff_hold_anchor` 헬퍼. Table/Picture/Equation 비교에
`prevent_page_break`(holdAnchorAndSO) 검사 추가 → IR-invisible 였던 드롭을 게이트가 봉인.

## 2. IR 통제 비교 (fidelity15, 11855 파일)
- IR_DIFF **4** (fidelity14와 동일) → **수정+게이트의 IR 회귀 0**.
- holdAnchorAndSO 게이트가 0 mismatch 검출 = 직렬화 수정 정상(보존됨).

## 3. 한글 오라클 (붕괴 해소 + 악화 검증)
- **36383351: 2쪽→2쪽 해소**(수정 전 1쪽 붕괴). holdAnchorAndSO 보존(1×"1") 확인.
- 이전 OK 표본 30: **30/30 OK 유지(악화 0)** — 보존 수정이라 OK 파일 붕괴 불가.
- 단, 이전 붕괴 표본 30: 22건이 holdAnchorAndSO 보존됐으나 **여전히 붕괴** →
  **군집 이질적**: holdAnchorAndSO 는 36383351 의 deciding 요인이나 대다수는 다른 systematic
  드롭(outlineShapeIDRef·noteSpacing·noteLine·curSz)이 deciding.

## 4. 판정 — 채택
holdAnchorAndSO 수정은 **정확한 IR-충실 버그 수정**: 드롭된 속성 보존, 36383351 해소,
IR 회귀 0, 악화 0, 게이트 개선. **채택**. 단 #1589 군집의 대다수는 별 원인(후속).

## 5. 후속
군집 잔여 붕괴 = 다른 IR-invisible 직렬화 드롭(outlineShapeIDRef/noteSpacing/noteLine/curSz)
추정. 각 별 조사·수정 필요(holdAnchorAndSO 와 동형 패턴 가능성).
