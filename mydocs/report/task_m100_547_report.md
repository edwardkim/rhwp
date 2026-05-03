# Task #547 최종 결과 보고서

**제목**: 21_언어_기출 passage 박스 안 본문 텍스트 좌측 inset 정정 (Task #544 후속)
**브랜치**: `local/task547`
**이슈**: https://github.com/edwardkim/rhwp/issues/547
**Milestone**: M100 (v1.0.0)

---

## 1. 요약

Task #544 에서 paragraph border 박스 outline 을 `col_area` 로 정정한 결과,
박스 안 본문 텍스트의 좌측 inset 이 PDF (한컴 2010) 보다 넓어진 부작용 발생.
원인: `paragraph_layout.rs` 의 `inner_pad_left` 분기 (paragraph border + visible
stroke + border_spacing=0 케이스에 box_margin_left 를 한 번 더 더하는 logic)
가 Task #544 이전 (박스도 margin 적용 시) 에 의미 있던 보정인데, Task #544 후
이중 inset 만 남음.

본 task 에서 inner_pad logic 제거 → 텍스트 inset = `box_margin_left` 한 번만
적용 (단일 룰).

**핵심 측정값** (페이지 4 [7~9] passage 박스):

| 항목 | PDF (한컴 2010) | 수정 전 | 수정 후 |
|------|-----------------|---------|---------|
| 박스 outline left x | 117.0 | 117.17 (Task #544 ✓) | 117.17 (변경 없음) |
| 본문 텍스트 min x (line 2+) | 128.5 | 139.89 (+11.4 px) | **128.53** (+0.03 px ✓) |
| 박스 안 좌측 여백 | ≈11.33 px | 22.66 px | **11.36 px** ✓ |

## 2. 변경 사항

### 2.1 `src/renderer/layout/paragraph_layout.rs` (-25 / +8 LOC)

```rust
// [Task #547] paragraph margin_left/right 는 텍스트 좌/우 inset 으로 한 번만
// 적용. Task #544 후 box outline = col_area (margin 미적용) 이므로 박스 안
// 좌측 여백 = box_margin_left (PDF 한컴 2010 정합).
// 이전 코드는 paragraph border + border_spacing=0 인 경우 inner_pad_left =
// box_margin_left 로 한 번 더 더해 이중 inset 부작용 발생 (Task #544 전 박스도
// margin 적용했을 때만 의미가 있던 분기).
let margin_left = box_margin_left;
let margin_right = box_margin_right;
```

제거된 변수: `para_border_fill_id_pre`, `has_visible_stroke`, `bs_left_px`,
`bs_right_px`, `inner_pad_left`, `inner_pad_right`.

### 2.2 `src/renderer/layout/integration_tests.rs` (+75 / -1 LOC)

`test_547_passage_text_inset_match_pdf_p4` 통합 테스트 추가:
- 페이지 4 [7~9] passage 박스 안 본문 텍스트 min x = 128.5 ±2 px (PDF 정합)

## 3. 핵심 설계 결정

### 3.1 단일 룰 적용 [feedback_rule_not_heuristic]

paragraph margin_left = 텍스트 좌측 inset (한 번만). paragraph border 유무,
border_spacing 값, visible stroke 검사 등 분기 모두 제거. 단일 산식.

### 3.2 박스 outline 보존

Task #544 의 box outline = `col_area` 산식은 그대로 유지. 본 task 는 텍스트
inset 만 정정. 박스 outline 좌표 검증 (`test_544_passage_box_coords_match_pdf_p4`)
무회귀 확인.

### 3.3 Task #544 와의 관계

Task #544 가 박스 outline 을 col_area 로 옮긴 후 inner_pad logic 의 의미가
상실됨 (Task #544 전: 박스 안 inset 보장 / 후: 이중 inset 부작용). 본 task
fix 는 Task #544 의 본질 정정에 따른 자연스러운 후속 조치.

## 4. 검증 결과

### 4.1 단위 테스트

```
test result: ok. 1121 passed; 0 failed; 2 ignored
test_547_passage_text_inset_match_pdf_p4 ... ok
test_544_passage_box_coords_match_pdf_p4 ... ok (--ignored)
```

기존 1120 단위 테스트 + Task #547 GREEN 1건. Task #534v2/#537/#539/#540/#544
무회귀.

### 4.2 광범위 회귀 검증 (6 샘플)

| 샘플 | +shift | -shift | unchanged | 평가 |
|------|--------|--------|-----------|------|
| 21_언어_기출 | 7010 | 7871 | 10094 | passage 박스 본문 의도 시프트 (max ±11.36) |
| exam_kor | 5675 | 9442 | 18102 | passage 박스 본문 의도 시프트 (max -36.67 = margin=5500 HU) |
| exam_math | **0** | **0** | 5286 | 무회귀 |
| exam_eng | 1417 | 1532 | 25118 | 일부 paragraph border 의도 시프트 |
| exam_science | **0** | **0** | 5411 | 무회귀 |
| synam-001 | **0** | **0** | 37559 | 무회귀 |

**텍스트 element 개수 무변경** — line wrap 변동 없음 (6 샘플 전체).

### 4.3 [13~15] passage 박스 직접 검증 (사용자 보고 케이스)

페이지 8 col 1 [13~15] 박스 안 본문:
- 시프트 건수: 556 건 (박스 본문 모든 글자)
- 시프트 양: -11.36 px = ps_id=11 box_margin_left
- 박스 안 좌측 여백: 22.66 px → **11.36 px** (정상화)

## 5. 위험 및 완화

| 위험 | 영향 | 완화 |
|------|------|------|
| paragraph border 본문 텍스트 위치 변경 | 매우 큼 | 광범위 회귀 검증 (6 샘플, line wrap 무변경) |
| Task #544 fix 와 충돌 | 큼 | Task #544 박스 outline 검증 무회귀 (`test_544`) |
| 셀 내부 / wrap=Square 호스트 케이스 회귀 | 큼 | paragraph_layout 단일 경로, 광범위 검증 무이상 |
| Task #534v2/#537/#539/#540 fix 와 충돌 | 큼 | 1120 단위 테스트 무회귀 |
| PDF 절대 기준 아님 [feedback_pdf_not_authoritative] | 중간 | 한컴 2010 PDF 일치 + 한컴 2020/한컴독스 검증 권고 |

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | inner_pad 분기 제거 (-25 / +8 LOC) |
| `src/renderer/layout/integration_tests.rs` | TDD 통합 테스트 (+75 / -1 LOC) |
| `mydocs/plans/task_m100_547.md` | 수행 계획서 |
| `mydocs/working/task_m100_547_stage{0,1,2,3}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_547_report.md` | 본 보고서 |

## 7. 커밋 이력

- `9f57d45f` Stage 0: 사전 분석 + paragraph border 텍스트 inset 산식 진단
- `9bec6d8a` Stage 1: TDD 통합 테스트 (RED) + 광범위 사전 평가
- `b3586723` Stage 2: paragraph border 텍스트 inset 산식 정정
- (Stage 3) — 본 보고서 + 광범위 회귀 검증

## 8. 회고

Stage 0 진단에서 본질 (Task #544 가 박스 outline 만 옮기고 inner_pad logic
그대로 둔 부작용) 을 정확히 식별. Stage 1 광범위 사전 평가로 영향 샘플 분포
파악 (paragraph border + visible stroke + bs=0 + margin>0 조합만 영향).
Stage 2 fix 적용 시 단일 룰 (margin 한 번만) 로 코드 단순화 + 분기 제거.
Stage 3 회귀 검증으로 영향 없는 샘플 (exam_math/science/synam-001) 0 shift
확인.

「본질 정정 회귀 위험」 [feedback_essential_fix_regression_risk] 적용: 광범위
샘플 검증 (6 샘플, line wrap 무변경 + 영향 없는 샘플 0 shift) 으로 회귀 위험
완화. 「룰과 휴리스틱 구분」 [feedback_rule_not_heuristic] 적용: paragraph
border 분기 / border_spacing 검사 등 휴리스틱 제거하고 단일 룰 (margin 한 번)
로 통합.

closes #547
