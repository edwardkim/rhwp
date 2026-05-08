# Task #634 Stage 3 — 광범위 회귀 검증 + 최종 보고서

**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/rhwp/issues/634
**Stage 2 보고서**: `mydocs/working/task_m100_634_stage2.md`
**진행 시점**: 2026-05-06

---

## 1. 검증 매트릭스 (20 샘플)

`./target/release/rhwp export-svg` 로 모든 페이지 출력 후 footer 영역 (y >= 1040) 텍스트
op 카운트.

| 문서 | 페이지 | with_footer | without_footer | 분류 |
|------|--------|-------------|----------------|------|
| **aift** | 77 | 71 | 6 (1~6) | NewNumber 발화 전 미표시 ✓ |
| **2022년 국립국어원** | 40 | 38 | 2 (1~2) | NewNumber 발화 전 미표시 ✓ |
| **biz_plan** | 6 | 4 | 2 (1~2) | NewNumber 발화 전 미표시 (PDF 없음) |
| **KTX** | 27 | 26 | 1 (1) | PageHide page_num=true 적용 ✓ |
| **hwp3-sample** | 16 | 16 | 0 | A안 (관대) 회귀 0 ✓ |
| **issue_265** | 16 | 16 | 0 | A안 회귀 0 ✓ |
| **pic-in-head-02** | 7 | 7 | 0 | A안 회귀 0 ✓ |
| **endnote-01** | 5 | 5 | 0 | A안 회귀 0 ✓ |
| **footnote-01** | 6 | 6 | 0 | A안 회귀 0 ✓ |
| **hwp-multi-001** | 10 | 10 | 0 | A안 회귀 0 ✓ |
| **table-vpos-01** | 5 | 5 | 0 | A안 회귀 0 ✓ |
| 21_언어_기출 | 15 | 8 | 7 | pgnp/nn 모두 0 — footer 는 다른 컨텐츠 |
| exam_eng | 8 | 8 | 0 | pgnp/nn 모두 0 — 시험지 footer |
| exam_math | 20 | 20 | 0 | pgnp/nn 모두 0 |
| exam_science | 4 | 4 | 0 | pgnp/nn 모두 0 |
| exam_math_8 | 1 | 1 | 0 | 단일 페이지 |
| equation-lim | 1 | 0 | 1 | 단일 페이지 (footer 없음) |
| 복학원서 | 1 | 1 | 0 | 단일 페이지 |
| pua-test | 1 | 0 | 1 | 단일 페이지 (footer 없음) |
| text-align-2 | 1 | 0 | 1 | 단일 페이지 (footer 없음) |

## 2. 미표시 페이지 정확 인덱스

```
=== aift (NewNumber 위치: 구역 2 문단 79 → 페이지 7) ===
  aift_001.svg ~ aift_006.svg  (페이지 1~6 미표시)

=== 2022년 국립국어원 (NewNumber 위치: 구역 0 문단 37 → 페이지 3,
                    페이지 1 PageHide page_num=true) ===
  2022년 국립국어원 업무계획_001.svg
  2022년 국립국어원 업무계획_002.svg

=== biz_plan ===
  biz_plan_001.svg
  biz_plan_002.svg

=== KTX (PageHide page_num=true paragraph 0.11 → 페이지 1) ===
  KTX_001.svg
```

## 3. 한컴 PDF vs rhwp 정량 비교 (국립국어원)

`/tmp/pdf_inspect/target/release/footer_detect3` 로 한컴 PDF 의 페이지별 footer 영역
(y < 84, q/Q 스택 기반) 텍스트 op 카운트. rhwp SVG 의 y >= 1040 텍스트 op 카운트와 비교.

```
Page 1 | 한컴 footer ops=0 (미표시) | rhwp footer ops=0 (미표시) ✓
Page 2 | 한컴 footer ops=0 (미표시) | rhwp footer ops=0 (미표시) ✓
Page 3 | 한컴 footer ops=3 (표시)   | rhwp footer ops=3 (표시)   ✓
Page 4 | 한컴 footer ops=3 (표시)   | rhwp footer ops=3 (표시)   ✓
Page 5 | 한컴 footer ops=3 (표시)   | rhwp footer ops=3 (표시)   ✓
Page 6 | 한컴 footer ops=18 (표시)  | rhwp footer ops=3 (표시)   ✓
```

**6/6 표시여부 정확 일치**. (페이지 6 한컴 ops=18 은 footer 영역에 추가 표 캡션/각주
컨텐츠 — 핵심은 "표시 vs 미표시" 일치.)

## 4. 단위 테스트

```
test result: ok. 1124 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

기존 1119 + 신규 5 = 1124. **회귀 0**.

## 5. 최종 보고서

`mydocs/report/task_m100_634_report.md` 작성 완료.

오늘할일 갱신: `mydocs/orders/20260506.md` 신규 작성.

## 6. 결론

**가설 H1'' 검증 완료. 광범위 회귀 0. 한컴 PDF 정량 일치.**

`closes #634`.
