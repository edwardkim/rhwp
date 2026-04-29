# Task #452 Stage 1: Baseline 측정 보고서

## 1. exam_kor 1페이지 좌측 단 baseline 측정

| 측정 | 값 | 비고 |
|------|------|------|
| pi=1 단락 내 step (line 0 → line 9) | 24.51 px | 균일 |
| **pi=1.line9 ↔ pi=2.line0 step** | **15.34 px** | **버그: 9.17 px 부족 (= 1 ls = 688 HU)** |
| pi=2 단락 내 step | 24.51 px | 균일 |
| pi=2.line4 ↔ pi=3.line0 step | 24.51 px | 우연 정상 (`composed.lines.len() ≠ LineSeg 수` 경로) |

baseline SVG 좌표:
- pi=1.line0 baseline y = 358.89 px
- pi=1.line9 baseline y = 579.45 px
- pi=2.line0 baseline y = 594.79 px
- pi=2.line4 baseline y = 692.81 px
- pi=3.line0 baseline y = 717.32 px

PDF (samples/exam_kor.pdf) 측정값: 모든 줄 step 12.96 pt 균일 (= 1838 HU).

## 2. 페이지 수 baseline (10 종 샘플)

| 샘플 | 페이지 수 |
|------|-----------|
| exam_kor.hwp | 20 |
| aift.hwp | 77 |
| biz_plan.hwp | 6 |
| 2022년 국립국어원 업무계획.hwp | 40 |
| exam_eng.hwp | 8 |
| exam_math_8.hwp | 1 |
| k-water-rfp.hwp | 28 |
| kps-ai.hwp | 80 |
| synam-001.hwp | 35 |
| 21_언어_기출_편집가능본.hwp | 15 |

캡처 위치: `/tmp/task_452_baseline/{base}/` (각 샘플 SVG 묶음).

## 3. Task #332 회귀 후보 식별

**21_언어_기출_편집가능본.hwp page 1 col 1** dump 결과:

```
단 1 (items=20, used=987.3px, hwp_used≈1213.1px, diff=-225.8px)
  ...
  FullParagraph  pi=26  h=24.9  vpos=78090  "2.-'프로세스 마이닝'에 대해 추론한 것으로 적절하지 않은 것은?"
  FullParagraph  pi=27  h=29.3  vpos=80802..82618  "①-프로세스 마이닝을 도입하면..."
  FullParagraph  pi=28  h=29.3  vpos=84434..86250  "②-프로세스 마이닝을 통해..."
  FullParagraph  pi=29  h=29.3  vpos=88066..89882  "③-프로세스 마이닝은 판에 박힌..."
```

**Task #332 회귀 검증 기준**: pi=26 + 보기 ①②③ (pi=27, pi=28, pi=29) 가 page 1 col 1 에 모두 fit (현재 ✓).

회귀 형태 예측: trailing ls 가산으로 누적 height 가 늘어 pi=29 가 page 2 로 밀릴 가능성. col 1 하단 여유 = 1226.4 - 987.3 = 239.1 px. 매 단락 +9.17 px (1 ls) × 4 단락 = 36.7 px 추가 → 여전히 fit (239.1 - 36.7 = 202.4 px 여유). **회귀 가능성 낮음**.

## 4. 이론 예측 (Stage 2 효과)

수정 전후 변화 예측:
- **exam_kor pi=1.line9 ↔ pi=2.line0 step**: 15.34 px → **24.51 px** (정상)
- **다른 단락 경계도 동일하게 +9.17 px**: 페이지당 단락 경계 수만큼 누적 → 페이지 끝 단락이 col_bottom 직전에 있을 경우 다음 페이지로 이동 가능성. 그러나 pagination engine 의 `effective_trailing` fit 판정 로직이 본 수정으로 변하지 않으므로 페이지 분배 자체는 유지될 것으로 예상.

## 5. 산출물

- `/tmp/task_452_baseline/{base}/` × 10 샘플 SVG
- `/tmp/task_452_baseline/page_counts.txt` 페이지 수 표
- `/tmp/task_452_baseline/21eum_p1.txt` 21_언어 page 1 dump

다음 단계: Stage 2 (코드 수정 + 단위 검증).
