# Task #959 Stage 4 — 다중 sample 회귀 검증

## 1. cargo test --release --lib

**결과**: **1288 passed, 0 failed, 2 ignored**

→ golden SVG diff 회귀 0.

## 2. 시험지 page 1 (Fix C 검증)

### Before fix
```
Shape pi=69 ci=0 y_in=709.4 y_out=983.4 dy=274.0 ⚠️
FullPara pi=73 y_in=1043.5 ...  (문9)
```
문9 가 y≈1061 처짐 (한컴 예상 ~810).

### After fix
```
Shape pi=69 ci=0 y_in=709.4 y_out=727.4 dy=18.0 ✓ (line advance only)
FullPara pi=73 y_in=787.5 y_out=839.2 ...  (문9)
```
문9 정상 위치 (y≈805) ✓ 한컴 PDF 정합.

## 3. 다중 sample 회귀 점검

| Sample | Page | 결과 |
|--------|------|------|
| 3-11월_실전_통합_2022 | 1 | 문9 정상 ✓ |
| 3-09월_교육_통합_2022 | 1 | 정상 (그림 정상 emit) |
| 3-09월_교육_통합_2023 | 1 | 정상 |
| 3-10월_교육_통합_2022 | 1 | 정상 (그래프 + 하트 그림 정상) |
| exam_kor | 18 | 정상 (Square wrap picture 영역, Task #722 영역) |
| exam_math | 1 | 정상 |
| exam_eng | 1 | 정상 |
| hwp3-sample11 | 1 | 정상 |

## 4. 평가

- 단위 검증 (시험지 page 1): ✓
- cargo test 전체: ✓ (1288/0/2)
- 다중 sample 시각 회귀: ✓ 0
- column 외부 picture 만 advance skip — column 내부 picture 영향 없음 확인

→ Stage 5 진행 (한컴 PDF 정합 + 최종 보고서 + PR).
