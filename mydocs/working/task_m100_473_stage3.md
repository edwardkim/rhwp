# Task #473 Stage 3 완료 보고서

## 단계 목표

광범위 샘플 회귀 시각 검증.

## 검증 결과

### 전체 cargo test

총 **1078 PASS / 0 FAIL** (#473 통합 테스트 + 기존 svg::tests 4건 갱신 포함).

### 광범위 샘플 nested-svg-image viewBox 검사

7개 샘플 (415개 페이지) 의 모든 nested `<svg viewBox><image></svg>` 에 대해 `viewBox / image` 비율이 1.1 초과 (그림 축소 버그) 인지 검사:

| 샘플 | 페이지 수 | nested-svg-image | viewBox > image (ratio > 1.1) |
|------|---|---|---|
| 21_언어_기출_편집가능본 | 15 | 0 | **0** ✓ (수정 후 nested SVG 미생성) |
| exam_kor | 20 | 20 | **0** ✓ (모두 partial crop, ratio < 1) |
| exam_eng | 8 | 11 | **0** ✓ |
| hwpspec | 177 | 1 | **0** ✓ |
| aift | 77 | 0 | 0 |
| k-water-rfp | 28 | 0 | 0 |
| kps-ai | 80 | 0 | 0 |

**총 0건 회귀**. 모든 partial crop 케이스(주로 exam_kor 헤더 로고) 정상 보존.

### 1차 검증 거짓 양성 (학습)

처음 회귀 검사 시 `abs(ratio - 1.0) > 0.1` 로 양방향 검출 → exam_kor 20/20 false alarm. 분석 결과: partial crop 은 viewBox < image 가 정상 (예: viewBox.w=1364 / image.w=2320 → 좌측 58.8% 표시). 검사 기준을 `ratio > 1.1` (그림이 viewBox 보다 작을 때) 만으로 좁힘.

메모리 `feedback_essential_fix_regression_risk` 적용: **광범위 샘플 검증 + 거짓 양성 재해석** 으로 본질 보존 확인.

## 21_언어_기출 12p 시각 확인

- 변경 전: `<svg viewBox="0 0 13875 10333"><image width="2220" height="1654"/></svg>` (16% 축소)
- 변경 후: `<image width="355.2" height="264.8"/>` (full crop 인식 → nested SVG 불필요, plain image 정상 렌더)

## 다음 단계

최종 보고서 작성 + orders 갱신 + commit + merge + PR 갱신.
