# Task #547 Stage 1 완료 보고서

**제목**: TDD 통합 테스트 (RED) + 광범위 사전 평가 + fix 위치 정밀 진단
**브랜치**: `local/task547`
**이슈**: https://github.com/edwardkim/edwardkim/rhwp/issues/547

---

## 1. TDD 통합 테스트 추가 (RED 확인)

`integration_tests.rs` 에 `test_547_passage_text_inset_match_pdf_p4` 추가.

페이지 4 col 0 [7~9] passage 박스 안 본문 텍스트 좌측 inset 검증:
- pi=82 (passage 본문) ParaShape: margin_left=1704 HU, indent=1984 HU, border_fill_id=4
- 두 번째+ 줄 (line_indent=0) 텍스트 x 좌표 측정
- PDF 기대값 128.5 px (±2 px)

```
test test_547_passage_text_inset_match_pdf_p4 ... FAILED
[7~9] 박스 안 본문 텍스트 최소 x=139.89 가 PDF 기대값 128.50 (±2 px) 와
일치해야 함. 버그(수정 전): min_x=139.89 (+11.4 px, inner_pad_left=margin_left
중복 적용).
```

측정값 **139.89 px** = col_area.x (117.17) + box_margin_left (11.36) +
inner_pad_left (11.36). Stage 0 진단 정확히 일치.

`#[ignore]` attribute. 1120 단위 테스트 모두 통과.

## 2. fix 위치 정밀 진단

### 2.1 본 코드 위치 (`paragraph_layout.rs:709-716`)

```rust
let bs_left_px = para_style.map(|s| s.border_spacing[0]).unwrap_or(0.0);
let bs_right_px = para_style.map(|s| s.border_spacing[1]).unwrap_or(0.0);
let (inner_pad_left, inner_pad_right) = if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
    (box_margin_left, box_margin_right)  // ← 이 분기가 부작용 원인
} else {
    (0.0, 0.0)
};
let margin_left = box_margin_left + inner_pad_left;
let margin_right = box_margin_right + inner_pad_right;
```

### 2.2 기존 분기의 의도 (Task #544 이전)

주석: "한컴은 paragraph margin 값을 inner padding으로도 사용하여 텍스트가
테두리에 붙지 않도록 한다."

Task #544 이전 box outline 산식:
- Box outline x = col_area.x + box_margin_left
- Text x = col_area.x + 2 * box_margin_left
- 박스 안 좌측 여백 = box_margin_left

Task #544 후 box outline 산식 (정정):
- Box outline x = col_area.x
- Text x = col_area.x + 2 * box_margin_left (변경 없음 → **부작용**)
- 박스 안 좌측 여백 = 2 * box_margin_left

### 2.3 fix 방향 (Stage 2 적용 예정)

```rust
// Task #547: Task #544 후 box outline = col_area, 텍스트 inset = box_margin_left
// 한 번만. 기존 inner_pad 분기 제거.
let margin_left = box_margin_left;
let margin_right = box_margin_right;
```

→ Text x = col_area.x + 11.36 = **128.53 px** ≈ PDF 128.5 ✓

## 3. 광범위 사전 평가

### 3.1 paragraph border + margin_left > 0 분포

| 샘플 | 영향받는 ps_id (margin_left HU) | 영향 범위 |
|------|---------------------------------|-----------|
| 21_언어_기출 | ps_id=11 (1704), ps_id=25 (1700) | passage 박스 본문 (pi=82, 144, 145+) |
| exam_kor | ps_id (1700) 다수 | passage box 본문 |
| exam_math | ps_id (2200) | math box 본문 |
| exam_science | ps_id (2260) 일부 | science box 본문 |
| exam_eng | margin_left=0 만 | **영향 없음** |
| synam-001 | (Stage 3 측정) | 미확인 |

### 3.2 영향 예상

paragraph border + margin_left > 0 인 모든 본문이 box_margin_left 만큼
좌측 시프트 (음의 시프트). PDF 와 정합되는 의도된 변경.

| 샘플 | 예상 본문 텍스트 시프트 |
|------|----------------------|
| 21_언어_기출 (ps_id=11) | -11.36 px |
| 21_언어_기출 (ps_id=25) | -11.33 px |
| exam_kor (1700) | -11.33 px |
| exam_math (2200) | -14.67 px |
| exam_science (2260) | -15.07 px |
| exam_eng | 0 px (변경 없음) |

광범위 회귀 검증 (Stage 3) 에서 `텍스트 -시프트` 카운트가 paragraph border
본문 라인 수와 일치하는지 확인 — 의도된 변경 검증.

## 4. 셀 내부 / wrap=Square 호스트 영향

### 4.1 셀 내부 paragraph border

`paragraph_layout.rs:716` 의 `margin_left` 변수는 셀 내부 / 본문 모두 동일
경로 사용. 셀 내부 paragraph border 가진 문단도 동일 부작용 발생 가능.

Stage 3 광범위 회귀 검증 시 셀 내부 paragraph border 케이스 확인.

### 4.2 wrap=Square 호스트 (`border_box_override`)

`paragraph_layout.rs:2697` 분기는 box outline x/w 만 override. 텍스트 inset
계산은 동일 logic 사용. wrap=Square 호스트 케이스도 동일 부작용 발생 가능.

이 케이스 fix 도 동일 적용 (margin_left = box_margin_left).

## 5. 회귀 위험 재평가

| 케이스 | 영향 | 완화 |
|--------|------|------|
| margin_left=0 paragraph (exam_eng 등) | 변경 없음 | - |
| margin_left>0 paragraph border 본문 | PDF 일치 (개선) | - |
| paragraph border 없음 | has_visible_stroke=false → 변경 없음 | - |
| 셀 내부 paragraph border | 동일 fix 적용 → PDF 일치 (개선 예상) | Stage 3 검증 |
| wrap=Square 호스트 | 동일 fix 적용 → PDF 일치 (개선 예상) | Stage 3 검증 |
| Task #544 fix 충돌 | Task #544 후속 fix → 충돌 없음 | - |

## 6. fix 방향 정리 (Stage 2 적용)

### 6.1 본질 정정

`paragraph_layout.rs:709-717` 의 inner_pad 분기 **완전 제거**:

```rust
// 제거 후
let margin_left = box_margin_left;
let margin_right = box_margin_right;
```

paragraph border + border_spacing 가드, has_visible_stroke 검사 모두 제거.
변수 `bs_left_px`, `bs_right_px`, `has_visible_stroke`, `inner_pad_left`,
`inner_pad_right`, `para_border_fill_id_pre` 도 사용처가 본 분기 한정이면
함께 제거.

### 6.2 적용 범위

본 fix 는 단일 위치 (`paragraph_layout.rs` 의 margin 산출) 변경. 박스 outline
산식 (Task #544) 은 그대로 보존.

## 7. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | TDD 테스트 1건 (RED, +75 LOC) |
| `mydocs/working/task_m100_547_stage1.md` | 본 보고서 |

## 8. 다음 단계 (Stage 2)

1. `paragraph_layout.rs:709-717` inner_pad 분기 제거
2. 변경된 단위 테스트 (RED → GREEN) 확인
3. 1120 기존 단위 테스트 무회귀 확인
4. Stage 2 보고서 + 커밋

## 9. 승인 요청

Stage 1 완료. 본질 정정 (inner_pad_left 분기 제거) 진행 OK?

승인 후 Stage 2 (fix 적용) 진행합니다.
