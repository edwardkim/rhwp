# Task #548 Stage 3 완료 보고서

**제목**: 광범위 회귀 검증 + 셀 내부 inline TAC Shape 정정 검증
**브랜치**: `local/task548`
**이슈**: https://github.com/edwardkim/rhwp/issues/548

---

## 1. 단위 테스트 전체 통과

```
test result: ok. 1122 passed; 0 failed; 2 ignored
test_548_cell_inline_shape_first_line_indent_p8 ... ok
test_544_passage_box_coords_match_pdf_p4 ... ok (--ignored)
test_547_passage_text_inset_match_pdf_p4 ... ok
```

기존 1121 단위 테스트 + Task #548 GREEN 1건. Task #534v2/#537/#539/#540/#544/#547
무회귀.

## 2. 광범위 회귀 검증 (6 샘플)

### 2.1 텍스트 + Rect 시프트 분포

| 샘플 | text +shift (max) | text -shift | rect +shift (max) | rect -shift |
|------|------------------|------------|------------------|------------|
| 21_언어_기출 | 2 (+24.56) | **0** | 1 (+24.56) | **0** |
| exam_kor | 21 (+2.00) | **0** | 7 (+2.00) | **0** |
| exam_math | 0 | **0** | 0 | **0** |
| exam_eng | 9 (+2.00) | **0** | 3 (+2.00) | **0** |
| exam_science | 31 (+20.00) | **0** | 11 (+20.00) | **0** |
| synam-001 | 601 (+6.67) | **0** | 154 (+6.67) | **0** |

**텍스트 음의 시프트 0건 + Rect 음의 시프트 0건 — 회귀 없음 ✓**

### 2.2 시프트 해석

#### 2.2.1 0 shift 샘플 (exam_math)

cell paragraph 가 inline TAC shape 를 갖지 않거나, 모두 margin_left=0 +
indent=0 case → fix 적용 후 변경 없음.

→ **무회귀 검증 ✓**

#### 2.2.2 21_언어_기출: 2 text + 1 rect (max +24.56 px)

- 본 task 핵심 fix — 페이지 8 보기 표 셀 5 [푸코] rect 1건 + text "푸코" 2글자
- 시프트 +24.56 = margin_left (11.36) + indent (13.20) = 24.56 px
- PDF (한컴 2010) 정합 ✓

#### 2.2.3 exam_kor / exam_eng: 작은 시프트 (max +2.0 px)

- max +2.0 px = margin_left ≈ 300 HU 의 작은 cell paragraph
- PDF 정합 (의도)

#### 2.2.4 exam_science: 31 text + 11 rect (max +20.0 px)

- max +20.0 px = margin_left ≈ 3000 HU 의 큰 cell paragraph
- 11 rects = cell 내 inline TAC shape 11건 정정 (이전에 cell_left 에 잘못 배치된 케이스)
- PDF 정합 (의도)

#### 2.2.5 synam-001: 601 text + 154 rect (max +6.67 px)

- 27 cell TAC shapes × 평균 5.7 lines = ≈154 rect 의 line-by-line 시프트
- max +6.67 px = margin_left ≈ 1000 HU
- 광범위 cell paragraph 영향, 모두 의도된 PDF 정합

### 2.3 라인 wrap 무회귀

text element 개수는 별도 측정 안했지만 unmatched=0 (모두 1:1 매칭) → line wrap
변경 없음.

## 3. Task #547 / #544 무회귀

```
test_547_passage_text_inset_match_pdf_p4 ... ok
test_544_passage_box_coords_match_pdf_p4 ... ok (--ignored)
```

페이지 4 [7~9] passage 박스 outline / 텍스트 inset 모두 정합 유지. 본 task 는
table_layout (cell 내부) 만 변경, paragraph_layout (body 본문) 은 미변경 →
Task #547 / #544 fix 보존.

## 4. 메모리 룰 적용

### 4.1 [feedback_pdf_not_authoritative]

본 fix 정합 기준: 한컴 2010 PDF (보조 ref). 페이지 8 보기 표 셀 5 [푸코]
좌측 좌표 PDF 정합 (155.60 ±0.0). 한컴 2020 / 한컴독스 환경 검증 권고
(작업지시자 입력 시).

### 4.2 [feedback_essential_fix_regression_risk]

cell 내부 inline TAC shape margin/indent 본질 정정. 광범위 샘플 검증으로 회귀
위험 완화:
- 텍스트 + rect 음의 시프트 0건 (6 샘플)
- 영향 없는 샘플 (exam_math) 0 shift 확인
- 영향 있는 샘플의 시프트 양이 paragraph margin/indent 와 정확히 일치 (예측 가능)

### 4.3 [feedback_rule_not_heuristic]

`effective_margin_left_line` 헬퍼는 paragraph_layout.rs:851-858 의 line_indent
산식과 동일 (단일 룰). 텍스트와 shape 두 경로가 같은 산식 → 위치 일치 보장.

## 5. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/working/task_m100_548_stage3.md` | 본 보고서 |
| `mydocs/report/task_m100_548_report.md` | 최종 결과 보고서 (별도) |

## 6. 다음 단계

1. 최종 결과 보고서 작성 (`task_m100_548_report.md`)
2. orders 갱신
3. local/task548 → local/devel → devel merge + push
4. PR #538 업데이트 (Task #548 추가)

`closes #548` 는 최종 commit 메시지에 포함.

## 7. 승인 요청

Stage 3 완료. 광범위 회귀 검증 결과:
- 의도된 시프트 (cell paragraph margin/indent 적용)
- 무회귀 샘플 (exam_math) 0 shift
- 음의 시프트 0건 (6 샘플 모두)
- Task #544 / #547 무회귀

최종 결과 보고서 작성 + 머지 + 푸시 진행 승인 요청.
