# Task #552 Stage 2 완료 보고서

**제목**: fix 적용 — paragraph border 시작 직전 trailing ls 보존
**브랜치**: `local/task552`
**이슈**: https://github.com/edwardkim/rhwp/issues/552

---

## 1. fix 적용 내용

### 1.1 LayoutEngine 에 Cell 추가 (`layout.rs:243-246`)

```rust
/// [Task #552] 다음 paragraph 가 visible border 시작이면 true.
/// 호출 직전 caller 가 set, 호출 직후 false 로 리셋.
next_para_starts_visible_border: std::cell::Cell<bool>,
```

### 1.2 헬퍼 메서드 추가 (`layout.rs:265-292`)

```rust
pub(crate) fn next_paragraph_starts_visible_border(
    &self, curr_pi: usize,
    paragraphs: &[Paragraph], styles: &ResolvedStyleSet,
) -> bool {
    let visible_bf = |ps_id: u16| -> bool {
        let ps = styles.para_styles.get(ps_id as usize)?;
        if ps.border_fill_id == 0 { return false; }
        let bf = styles.border_styles.get((ps.border_fill_id as usize) - 1)?;
        bf.borders.iter().any(|b|
            !matches!(b.line_type, BorderLineType::None) && b.width > 0)
    };
    let curr = paragraphs.get(curr_pi)?;
    let next = paragraphs.get(curr_pi + 1)?;
    visible_bf(next.para_shape_id) && !visible_bf(curr.para_shape_id)
}
```

### 1.3 paragraph_layout.rs trailing ls 분기 보강 (`paragraph_layout.rs:2647-2660`)

```rust
let next_starts_border = self.next_para_starts_visible_border.get();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if is_full_paragraph_end && cell_ctx.is_none() && !next_starts_border {
    // 셀 외부 paragraph 의 마지막 줄 (#479)
    y += line_height;
} else {
    // [Task #552] border-start 직전 마지막 줄: trailing ls 보존
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}
```

### 1.4 caller 보강 (`layout.rs` 3 위치)

`layout.rs:1987` FullParagraph (layout_paragraph 호출), `layout.rs:2042`
PartialParagraph (wrap host), `layout.rs:2156` PartialParagraph (text start
line) 직전:

```rust
self.next_para_starts_visible_border.set(
    self.next_paragraph_starts_visible_border(*para_index, paragraphs, styles));
y_offset = self.layout_partial_paragraph(...);
self.next_para_starts_visible_border.set(false);
```

PartialParagraph 의 경우 추가 가드: `end_line >= comp.lines.len()` (paragraph
완전 종료) 일 때만 적용.

## 2. 검증

### 2.1 RED → GREEN

```
test test_552_passage_box_top_gap_p2_4_6 ... ok
[4~6] 박스 top y=233.97 (PDF 정합 ±2 px tolerance, gap=9.54 px ≈ PDF 8.73 px)
```

수정 전: gap=0.00 (FAIL). 수정 후: gap=9.54 (PASS).

### 2.2 1119 단위 테스트 무회귀

```
test result: ok. 1119 passed; 0 failed; 2 ignored
```

### 2.3 페이지 카운트 무회귀 (6 샘플)

| 샘플 | pages | Task #479 본 효과 |
|------|-------|------------------|
| 21_언어_기출 | 15 | 보존 (페이지 12 200px drift fix) |
| exam_kor | 20 | 보존 |
| exam_math | 20 | 보존 |
| exam_eng | 8 | 보존 |
| exam_science | 6 | 보존 |
| synam-001 | 35 | 보존 |

→ 페이지 누적 본질 변경 없음 (no→border transition 만 ls 보존, 그 외 1999/2047
케이스 동일).

## 3. 핵심 측정값

| 항목 | pre-#479 | post-#479 (회귀) | post-#552 (fix) | PDF 한컴 2010 |
|------|----------|----------------|----------------|----------------|
| 박스 top y | 233.97 | 224.43 | **233.97** | 175.36 pt |
| gap (header→box) | 9.54 | 0.00 | **9.54** | 8.73 px |

→ pre-#479 baseline 정확 회복. PDF ±2 px tolerance 통과.

## 4. 영향 범위

### 4.1 변경 케이스 (6 샘플 48 cases)

| 샘플 | no→border 케이스 |
|------|-----------------|
| 21_언어_기출 | 10 (포함 페이지 2 [4~6] 등) |
| exam_kor | 14 |
| exam_math | 8 |
| exam_eng | 16 |
| exam_science | 0 |
| synam-001 | 0 |

각 case 의 박스 top y 가 ~9.54 px 하향 → PDF 정합 회복.

### 4.2 무영향 케이스 (1999 cases)

- in_border (border 그룹 내부) 290
- border→no (border 끝) 48
- no→no (본문 sequential) 1652

→ Task #479 의 본 효과 (페이지 12 200px drift fix) 보존.

## 5. 별도 사항

"글상자 우측 시프트" (box_left=591.49 vs PDF 580) 850 HU 차이는 pre-existing
결함. 본 task scope 외 (별도 이슈 등록 후보).

## 6. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | Cell + helper + 3 caller 보강 (+44 LOC) |
| `src/renderer/layout/paragraph_layout.rs` | 분기 보강 (+5 / -1 LOC) |
| `src/renderer/layout/integration_tests.rs` | RED 테스트 1건 (Stage 1 추가) |
| `examples/scan_border_starts.rs` | 광범위 평가 도구 (Stage 1 추가) |
| `mydocs/working/task_m100_552_stage2.md` | 본 보고서 |

## 7. 다음 단계 (Stage 3)

1. 광범위 회귀 검증 (6 샘플 박스 위치 시프트 분포 측정)
2. Task #544 [7~9] 페이지 4 박스 PDF 정합 시각 검증
3. Task #547/#548 무회귀 확인
4. 최종 결과 보고서

## 8. 승인 요청

Stage 2 완료. RED → GREEN, 1119 baseline 무회귀, pre-#479 정확 회복. Stage 3
(광범위 회귀 검증 + 최종 보고서) 진행 OK?
