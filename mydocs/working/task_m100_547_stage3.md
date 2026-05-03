# Task #547 Stage 3 완료 보고서

**제목**: 광범위 회귀 검증 + paragraph border 텍스트 inset 정정 검증
**브랜치**: `local/task547`
**이슈**: https://github.com/edwardkim/rhwp/issues/547

---

## 1. 단위 테스트 전체 통과

```
test result: ok. 1121 passed; 0 failed; 2 ignored
test_547_passage_text_inset_match_pdf_p4 ... ok
test_544_passage_box_coords_match_pdf_p4 ... ok (--ignored)
```

기존 1120 단위 테스트 + Task #547 GREEN 1건. Task #534v2/#537/#539/#540/#544
무회귀.

## 2. 광범위 회귀 검증 (6 샘플)

### 2.1 텍스트 x 시프트 분포 (Stage 1 baseline 대비)

| 샘플 | +shift | -shift | unchanged | max +shift | max -shift |
|------|--------|--------|-----------|-----------|-----------|
| 21_언어_기출 | 7010 | 7871 | 10094 | +12.64 | -11.36 |
| exam_kor | 5675 | 9442 | 18102 | +12.70 | **-36.67** |
| exam_math | **0** | **0** | 5286 | 0 | 0 |
| exam_eng | 1417 | 1532 | 25118 | +12.26 | -11.33 |
| exam_science | **0** | **0** | 5411 | 0 | 0 |
| synam-001 | **0** | **0** | 37559 | 0 | 0 |

**텍스트 element 개수 무변경** (line wrap 무회귀): 6 샘플 모두 before/after
text element count 동일.

### 2.2 시프트 해석

#### 2.2.1 0 shift 샘플 (exam_math, exam_science, synam-001)

paragraph border + visible-stroke + border_spacing[0]=[1]=0 + margin_left>0
조합 케이스가 없음. inner_pad logic 발동되지 않았으므로 fix 영향 없음.

→ **무회귀 검증 ✓**

#### 2.2.2 -shift = box_margin_left (좌측 정렬 텍스트)

좌측 정렬 텍스트는 fix 후 좌측 시프트 (`text_x = col_area.x + box_margin_left`,
이전엔 `+ 2*box_margin_left`).

| 샘플 | 예상 -shift 절대값 | 측정 max -shift |
|------|-------------------|---------------|
| 21_언어_기출 (margin_left=1700/1704 HU) | 11.33/11.36 px | -11.36 ✓ |
| exam_kor (margin_left=1700~5500 HU) | 11.33~36.67 px | **-36.67** ✓ |
| exam_eng (margin_left=1700 HU 일부) | 11.33 px | -11.33 ✓ |

exam_kor 의 -36.67 px = ParaShape margin_left=5500 HU / 2 / 7200 * 96. 큰
margin paragraph 의 정상 시프트 — **회귀 아님** (의도된 PDF 정합).

#### 2.2.3 +shift ≈ box_margin_left (우측 정렬 텍스트)

우측 정렬 텍스트는 fix 후 우측 시프트 (`text_x = col_x + (avail_w - text_w)`,
avail_w 가 fix 후 2*box_margin_left 만큼 커짐 → 텍스트 우측 끝 close to 박스 우측).

이는 paragraph border 박스 안 우측 정렬 텍스트의 **PDF 정합** 효과 (인용문 끝
화살표 같은 우측 정렬 케이스).

### 2.3 [13~15] passage 박스 직접 검증 (사용자 보고 케이스)

페이지 8 col 1 [13~15] 박스 안 본문 텍스트:
- 시프트 건수: 556 건 (박스 본문 모든 글자)
- 시프트 양: -11.36 px (= ps_id=11 box_margin_left)
- 박스 안 좌측 여백: **22.66 px → 11.36 px** (정상화)

작업지시자 보고 ("박스 안에 여백이 조금 넓은 것 같음") 정정 완료.

## 3. 박스 outline 무회귀 (Task #544 검증)

```
test_544_passage_box_coords_match_pdf_p4 ... ok
```

페이지 4 [7~9] 박스 outline 좌표 (top_y=233.97 / left_x=117.17 / width=425.17)
모두 PDF (±2 px) 일치 유지. Task #544 fix 보존.

## 4. 셀 내부 / wrap=Square 호스트 케이스

### 4.1 셀 내부 paragraph border

`paragraph_layout.rs:716` 의 `margin_left` 변수는 본문/셀 내부 동일 경로.
fix 자동 적용. 광범위 회귀 검증 6 샘플 (셀 포함) 텍스트 element 개수 무변경
→ 셀 내부 케이스 무회귀.

### 4.2 wrap=Square 호스트 (`border_box_override`)

`paragraph_layout.rs:2697` 분기는 box_x/box_w 만 override. 텍스트 inset 산출
은 본 fix 영향 받음 (box_margin_left 만 적용). 광범위 회귀 검증에서 이상
없음.

## 5. 메모리 룰 적용

### 5.1 [feedback_pdf_not_authoritative]

본 fix 정합 기준: 한컴 2010 PDF (보조 ref). 한컴 2020 / 한컴독스 환경 검증
권고. 작업지시자 입력 가능 시 추가 검증.

핵심 측정 (페이지 4 [7~9] passage 박스 안 본문 첫 글자 x):
- PDF (한컴 2010): 128.5 px
- rhwp 수정 후: 128.53 px
- 차이: +0.03 px ✓

### 5.2 [feedback_essential_fix_regression_risk]

paragraph border 본문 텍스트 inset 본질 정정. 광범위 샘플 검증으로 회귀 위험
완화:
- 텍스트 element 개수 무변경 (line wrap 무회귀)
- 영향 없는 샘플 (exam_math/science/synam-001) 0 shift 확인
- 영향 있는 샘플의 시프트 양이 box_margin_left 와 일치 (예측 가능)

### 5.3 [feedback_rule_not_heuristic]

paragraph margin_left 는 텍스트 좌측 inset (한 번만, 단일 룰). paragraph
border 유무 / border_spacing 값 / has_visible_stroke 검사 분기 모두 제거.

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/working/task_m100_547_stage3.md` | 본 보고서 |
| `mydocs/report/task_m100_547_report.md` | 최종 결과 보고서 (별도) |

## 7. 다음 단계

1. 최종 결과 보고서 작성 (`task_m100_547_report.md`)
2. orders 갱신
3. local/task547 → local/devel → devel merge + push
4. PR #538 업데이트 (Task #547 추가) 또는 새 PR 검토

`closes #547` 는 최종 commit 메시지에 포함.

## 8. 승인 요청

Stage 3 완료. 광범위 회귀 검증 결과:
- 의도된 시프트 (paragraph border 본문 텍스트 좌측/우측 정렬 inset 정정)
- 무회귀 샘플 (exam_math/science/synam-001) 0 shift
- 박스 outline 무회귀 (Task #544 검증 유지)
- 1121 단위 테스트 통과

최종 결과 보고서 작성 + 머지 + 푸시 진행 승인 요청.
