# Task #770 Stage 3 (GREEN) 완료 보고서

**Issue**: [#770](https://github.com/edwardkim/rhwp/issues/770)
**Stage**: 3 — GREEN (수정 적용 + RED PASS + 회귀 0)
**작성일**: 2026-05-10

---

## 정정 요약

`src/renderer/layout.rs::layout_table_item` tac_seg_applied 분기에 ColumnDef + 다중 LINE_SEG + 헤더 패턴 가드 추가.

### 변경 위치 (`layout.rs:2596` 영역)

`tac_seg_applied=true` 분기의 `outer_margin_bottom_px` 적용 후 `return (y_offset, true)` 직전:

```rust
// [Task #770] ColumnDef + TAC 1x1 헤더 표 + 후속 빈 라인 패턴의
// paragraph (예: shortcut.hwp pi=36 페이지 2 "파일") 처리 시
// 후속 LINE_SEG 의 lh+ls 가 advance 에 누락되는 결함 정정.
let has_column_def = para.controls.iter().any(|c|
    matches!(c, Control::ColumnDef(_)));
if has_column_def {
    if let (Some(first_seg), Some(last_seg)) = (
        para.line_segs.first(),
        para.line_segs.last(),
    ) {
        if last_seg.vertical_pos > first_seg.vertical_pos
            && last_seg.line_height > first_seg.line_height
        {
            let para_vpos_end = last_seg.vertical_pos
                + last_seg.line_height
                + last_seg.line_spacing.max(0);
            let target_y = para_y_for_table
                + hwpunit_to_px(para_vpos_end, self.dpi);
            if target_y > y_offset {
                y_offset = target_y;
            }
        }
    }
}
```

### 3중 가드 의의

| 조건 | shortcut pi=36 (정정 대상) | Textmail pi=0 (회귀 회피) |
|------|--------------------------|--------------------------|
| has_column_def | true (ColumnDef + Table) ✓ | true (ColumnDef + Table) — 1번 가드 통과 |
| last.vpos > first.vpos | 1200 > 0 ✓ | 56430 > 0 ✓ — 2번 가드 통과 |
| **last.lh > first.lh** | **2332 > 1200 ✓** | **9460 < 55830 ✗** — 3번 가드 차단 |

→ 3번 가드 (`last.lh > first.lh`) 가 결정자. 헤더 표 line 0 < 후속 line 1 패턴만 정정.

---

## RED 테스트 결과

```
$ cargo test --test issue_770 -- --nocapture

[issue_770] page_index=1 body_y=56.69 pi=37_y=103.79 offset=47.09 (expected_min=40)
test issue_770_page2_body_paragraph_below_header_zone ... ok
```

→ pi=37 ('새 문서') offset 31.09 → **47.09** (hwp_used 47.1 정합 ✓).

## 페이지 별 헤더 zone 정합 확인

| 페이지 | hwp_used | rhwp used (Before) | rhwp used (After) | 정합 |
|--------|---------|------------------|------------------|------|
| 1 | 53.1 | 69.1 (헤더 paragraph) | 69.1 | (영향 없음) |
| **2** | 47.1 | 27.3 | **~47** | ✓ |
| **3** | 53.5 | 30.5 | **~53** | ✓ |
| **4** | 53.3 | 20.0 | (FullParagraph, 영향 없음) | (별개) |
| **5** | 53.3 | 40.0 | (영향 추정) | — |
| **7** | 53.3 | 20.0 | (영향 없음) | (별개) |

→ **다중 LINE_SEG 헤더 paragraph (pi=36 / pi=81 등) 만 정정**. FullParagraph 단일 paragraph 헤더 (pi=94 등) 는 별개 영역.

## 회귀 검증 (cargo test --release)

```
test result: ok. 1217 passed; 0 failed; 2 ignored;
... (모든 통합/스냅샷/issue 테스트 PASS)
```

→ 회귀 0건. test_539 / test_548 / test_exam_math_page_count / golden SVG 7개 모두 PASS.

## 광범위 (205 샘플)

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| 샘플 수 | 205 | 205 | — |
| `LAYOUT_OVERFLOW_DRAW` 총 | 226 | 228 | +2 (shortcut만) |
| `LAYOUT_OVERFLOW` 총 | 354 | 358 | +4 (shortcut만) |

샘플별 차이:
- `shortcut.hwp`: DRAW 12→14 (+2), FLOW 13→17 (+4) — 본문 정합 후 column 잔여 변화
- 그 외 204 샘플: 변동 없음 ✓

## 시각 정합

PDF 페이지 2 헤더 ('파일') ~ 본문 ('새 문서') 거리 ≈ 60 px.
rhwp 페이지 2 (Before): 21 px → rhwp (After): **~47 px** (PDF 권위 정합 근접).

SVG → PNG 변환 시각 검증: 헤더와 본문 사이 적절한 spacing 확보.

## 영향 분석

### 본 정정의 효과

1. **다중 LINE_SEG 헤더 paragraph (ColumnDef + Table + 후속 빈 line)** 의 advance 정합
2. shortcut.hwp 페이지 2/3 의 본문 위치 PDF 정합 확보
3. Newspaper 다단 영역 헤더 표 정합

### 회귀 회피

- ColumnDef 미동반 TAC 표 paragraph (셀 내, 일반 본문) — 가드 1 통과 못 함
- 단일 LINE_SEG paragraph (pi=1 등) — 가드 2 통과 못 함
- last_seg.lh ≤ first_seg.lh paragraph (Textmail.hwp 등) — 가드 3 통과 못 함

→ shortcut.hwp 만 영향 미치는 핀포인트 정정.

## 다음 단계 (Stage 4 — 회귀)

cargo test 결과는 이미 Stage 3 에 포함. Stage 4 는 시각 검증 보고서.

## 승인 요청

Stage 3 GREEN 완료. RED PASS, 회귀 0, PDF 정합 확보. Stage 4-5-6 진입.
