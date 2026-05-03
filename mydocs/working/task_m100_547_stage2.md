# Task #547 Stage 2 완료 보고서

**제목**: paragraph border 텍스트 inset 산식 정정 (inner_pad_left 분기 제거)
**브랜치**: `local/task547`
**이슈**: https://github.com/edwardkim/rhwp/issues/547

---

## 1. fix 적용 내용

### 1.1 `paragraph_layout.rs:693-717` (변경 -25 / +8 LOC)

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

### 1.2 제거된 코드

- `para_border_fill_id_pre`, `has_visible_stroke` 변수 (border stroke 검사)
- `bs_left_px`, `bs_right_px` 변수 (border_spacing 검사)
- `inner_pad_left`, `inner_pad_right` 분기 (= `box_margin_left/right` 한번 더)

분기 자체가 본질적으로 잘못된 우회 (Task #544 이전 박스 margin 적용 시 의미
있던 보정). Task #544 가 박스를 col_area 로 옮긴 후 부작용 (이중 inset) 만
남음.

## 2. 검증

### 2.1 단위 테스트 — RED → GREEN

```
test test_547_passage_text_inset_match_pdf_p4 ... ok
test result: ok. 1121 passed; 0 failed; 2 ignored
```

페이지 4 [7~9] passage 박스 안 본문 텍스트 min x = **128.50 px** ≈ PDF 기대
128.5 px (Stage 1 RED 측정 139.89 → 128.50, -11.36 px 보정).

기존 1120 단위 테스트 모두 통과. Task #534v2/#537/#539/#540/#544 무회귀.
Task #547 통합 테스트 1건 추가 (총 1121).

### 2.2 핵심 측정값 (페이지 4 [7~9] passage 박스)

| 좌표 | Stage 0 측정 | Stage 2 fix 후 | PDF 기대 | 차이 |
|------|-------------|--------------|----------|------|
| Box outline x | 117.17 | 117.17 (변경 없음) | 117.0 | +0.17 ✓ |
| Box width | 425.17 | 425.17 (변경 없음) | 425.1 | +0.07 ✓ |
| Text min x (line 2+) | 139.89 | **128.53** | 128.5 | +0.03 ✓ |
| 박스 안 좌측 여백 | 22.66 px | **11.36 px** | ≈11.33 px | +0.03 ✓ |

## 3. 영향 범위

### 3.1 paragraph border + margin_left > 0 본문 (의도된 시프트)

| 샘플 | 영향 ps_id (margin_left HU) | 예상 -시프트 |
|------|---------------------------|-------------|
| 21_언어_기출 | ps_id=11 (1704), ps_id=25 (1700) | -11.36 / -11.33 px |
| exam_kor | ps_id (1700) 다수 | -11.33 px |
| exam_math | ps_id (2200) | -14.67 px |
| exam_science | ps_id (2260) 일부 | -15.07 px |
| exam_eng | margin_left=0 만 | 영향 없음 |

광범위 회귀 검증 (Stage 3) 시 -시프트 카운트가 paragraph border 본문 라인
수와 일치하는지 검증.

### 3.2 paragraph border 없음 — 영향 없음

`box_margin_left` 만 적용 (이전과 동일). Task #547 fix 는 paragraph border
유무와 관계없이 동일 산식 (margin 한 번만) 적용 — 단일 룰 [feedback_rule_not_heuristic].

paragraph border 없는 paragraph 는 이전에도 inner_pad_left=0 이었으므로 결과
변경 없음.

### 3.3 셀 내부 paragraph border / wrap=Square 호스트

본 fix 는 paragraph_layout 내 margin 산출에 적용 — 셀 내부 / wrap host 케이스
모두 동일 경로 사용. 동일 fix 자동 적용.

Stage 3 회귀 검증으로 케이스별 무회귀 확인.

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | inner_pad 분기 제거 (-25 / +8 LOC) |
| `src/renderer/layout/integration_tests.rs` | `#[ignore]` 제거 (RED→GREEN) |
| `mydocs/working/task_m100_547_stage2.md` | 본 보고서 |

## 5. 핵심 설계 결정

### 5.1 단일 룰 적용 [feedback_rule_not_heuristic]

paragraph margin_left = 텍스트 좌측 inset (한 번만). paragraph border 유무 /
border_spacing 값 / has_visible_stroke 등 분기 없이 단일 산식.

### 5.2 박스 outline 보존

Task #544 의 box outline = col_area 산식은 그대로 유지. 본 task 는 텍스트
inset 만 정정. 박스 outline 좌표 검증 (`test_544_passage_box_coords_match_pdf_p4`)
무회귀.

### 5.3 영향 범위 명확

본 fix 는 단일 위치 (`paragraph_layout.rs:693-717`) 변경. paragraph_layout
내부 함수만 영향, layout.rs 등 외부 모듈 변경 없음.

## 6. 다음 단계 (Stage 3)

1. 광범위 회귀 검증 — 6 샘플 vs Stage 1 baseline:
   - 21_언어_기출 (예상 -시프트: passage 박스 본문 라인 수)
   - exam_kor (예상 -시프트)
   - exam_math (예상 -시프트)
   - exam_science (예상 -시프트 일부)
   - exam_eng (무변경)
   - synam-001 (paragraph border 분포 측정)
2. 박스 outline 무회귀 (Task #544 검증 유지)
3. 셀 내부 / wrap=Square 호스트 케이스 검증
4. 한컴 2020 / 한컴독스 비교 (작업지시자 입력 시)
5. Stage 3 보고서 + 최종 보고서

## 7. 승인 요청

Stage 2 완료. RED → GREEN, 1121 단위 테스트 모두 통과. 박스 안 좌측 여백
PDF 정합 (128.50 px ✓).

Stage 3 (광범위 회귀 검증 + 최종 보고서) 진행 승인 요청.
