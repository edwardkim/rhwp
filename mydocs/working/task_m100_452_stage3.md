# Task #452 Stage 3: 광범위 회귀 검증 보고서

## 1. 페이지 수 비교 (10 종 샘플)

| 샘플 | Stage 1 (baseline) | Stage 3 (after fix) | 변동 |
|------|-------|-------|------|
| exam_kor.hwp | 20 | 20 | 0 ✓ |
| aift.hwp | 77 | 77 | 0 ✓ |
| biz_plan.hwp | 6 | 6 | 0 ✓ |
| 2022년 국립국어원 업무계획.hwp | 40 | 40 | 0 ✓ |
| exam_eng.hwp | 8 | 8 | 0 ✓ |
| exam_math_8.hwp | 1 | 1 | 0 ✓ |
| k-water-rfp.hwp | 28 | 28 | 0 ✓ |
| kps-ai.hwp | 80 | 80 | 0 ✓ |
| synam-001.hwp | 35 | 35 | 0 ✓ |
| 21_언어_기출_편집가능본.hwp | 15 | 15 | 0 ✓ |

**결과**: 10/10 샘플 페이지 수 동일. 페이지네이션 회귀 0.

## 2. byte 차이 통계 (대표 6 종)

| 샘플 | 변경 페이지 / 전체 |
|------|-------------------|
| exam_kor | 20/20 |
| aift | 70/77 |
| biz_plan | 6/6 |
| exam_eng | 8/8 |
| synam-001 | 22/35 |
| 21_언어_기출_편집가능본 | 15/15 |

본문 단락이 있는 페이지는 거의 모두 변경됨 (모든 텍스트 y 좌표가 trailing ls 누적분만큼 시프트). 표지/이미지 전용 페이지(aift 7건, synam-001 13건)는 변경 없음. **변경 패턴: 일관된 y 시프트, 콘텐츠/구조 변화 없음** — 의도된 정합 효과.

## 3. Task #332 회귀 점검 (21_언어 page 1 col 1)

`./target/release/rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 0`:

```
단 1 (items=20, used=987.3px, hwp_used≈1213.1px, diff=-225.8px)
  ...
  FullParagraph  pi=26  h=24.9  vpos=78090  "2.-'프로세스 마이닝'에 대해 추론한 것..."
  FullParagraph  pi=27  h=29.3  vpos=80802..82618  "①-..."
  FullParagraph  pi=28  h=29.3  vpos=84434..86250  "②-..."
  FullParagraph  pi=29  h=29.3  vpos=88066..89882  "③-..."
```

**결과**: pi=26 + 보기 ①②③ (pi=27, pi=28, pi=29) 모두 page 1 col 1 에 fit. **#332 회귀 0**.

이론 검증: pagination engine 의 fit 판정은 `effective_trailing` (마지막 단락의 trailing ls 만 fit 시 제외) 을 사용 → 본 수정으로 fit 판정 로직 자체는 변하지 않음. layout 의 y 시프트만 정합되어, 페이지 분배 결과는 동일하게 유지됨.

## 4. golden SVG snapshot 영향

Stage 2 에서 갱신:
- `tests/golden_svg/issue-147/aift-page3.svg` (TOC 페이지)
- `tests/golden_svg/issue-157/page-1.svg` (등기 양식 페이지)

PNG 시각 검토: 두 페이지 모두 콘텐츠 정상 표시, 잘림/겹침/누락 없음. Task #332 stage 5 에서 갱신된 baseline → 본 정합 baseline 으로 재갱신.

## 5. 시각 검토 (exam_kor page 1, page 20)

- **page 1**: pi=1↔pi=2 간격 정상화 + 다른 단락 경계도 모두 단락내 step 과 동일. PDF 와 시각 정합 향상.
- **page 20**: 마지막 페이지, 콘텐츠 잘림 없음. 페이지 배분 유지 확인.

## 6. LAYOUT_OVERFLOW 빈도

snapshot 테스트 시 issue-157 에서 1건의 LAYOUT_OVERFLOW (pi=28, 10.9 px) 발생 — 페이지 마지막 단락의 trailing ls 가 col_bottom 을 살짝 넘는 cosmetic 효과. 빈 공간이므로 시각 무영향. 이 경고는 본 수정의 알려진 부수 효과이며 실제 콘텐츠 잘림은 없음.

## 7. 결론

- 페이지네이션 회귀 0 (10/10 샘플)
- Task #332 회귀 0 (21_언어 fit 유지)
- golden SVG 2건 baseline 갱신 (의도된 정합)
- byte 차이는 모두 의도된 y 시프트, 구조 변화 0
- 시각 검토 통과

**Stage 4 진행 가능**.
