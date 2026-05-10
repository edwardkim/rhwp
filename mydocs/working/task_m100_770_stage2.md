# Task #770 Stage 2 (분석) 완료 보고서

**Issue**: [#770](https://github.com/edwardkim/rhwp/issues/770)
**Stage**: 2 — 분석 / instrument
**작성일**: 2026-05-10

---

## 산출물

`RHWP_TASK770_DEBUG=1` 환경변수 가드 instrument (3 위치):
- `layout.rs::layout_table_item` 진입/종료 — TASK770_TBL_ENTER / TASK770_TBL
- `layout.rs::PageItem::PartialParagraph` 처리 — TASK770_PP
- `paragraph_layout.rs::line advance` — TASK770_LINE

GREEN 후 instrument 모두 제거.

## Trace 결과

```
TASK770_TBL_ENTER: pi=1  ci=1 y_in=99.36
TASK770_TBL:       pi=1  y_in=99.36 y_out=130.45 advance=31.09
TASK770_TBL_ENTER: pi=29 ci=1 y_in=410.45
TASK770_TBL:       pi=29 y_in=410.45 y_out=441.55 advance=31.09
TASK770_TBL_ENTER: pi=36 ci=1 y_in=56.69
TASK770_TBL:       pi=36 y_in=56.69 y_out=87.79 advance=31.09  ← 결함
TASK770_TBL_ENTER: pi=81 ci=1 y_in=56.69
TASK770_TBL:       pi=81 y_in=56.69 y_out=94.19 advance=37.49
```

## 핵심 발견 — LINE_SEG 차이

| paragraph | LINE_SEG 개수 | first_seg | last_seg | vpos_end (HU) | vpos_end (px) | 실제 advance |
|-----------|--------------|----------|---------|--------------|---------------|------------|
| pi=1 (페이지 1 보기) | **1** | vpos=0 lh=2332 | (동일) | 2332 | **31.1** | 31.09 ✓ |
| pi=29 (페이지 1 입력) | **1** | vpos=0 lh=2332 | (동일) | 2332 | 31.1 | 31.09 ✓ |
| pi=36 (페이지 2 파일) | **2** | vpos=0 lh=1200 | vpos=1200 lh=2332 | **3532** | **47.1** | **31.09 ❌** |
| pi=81 (페이지 3 보기) | **2** | vpos=0 lh=1200 ls=480 | vpos=1680 lh=2332 ls=480 | 4012 | 53.5 | 37.49 ❌ |

**Root cause**: `tac_seg_applied` 분기 (`layout.rs:2585+`) 가 paragraph 의 `line_segs.get(control_index)` 만 처리. 다중 LINE_SEG paragraph 의 후속 line 의 lh+ls 가 advance 에 누락 → 본문 압축.

## 영향 범위 식별

dump 으로 다중 LINE_SEG TAC 표 paragraph 검색 → shortcut.hwp 외에도 광범위 (Textmail.hwp 등). 그러나 shortcut.hwp 만 결함, 다른 샘플은 paragraph 의 본질이 다름:

- **shortcut pi=36**: ColumnDef + Table + 후속 빈 line. ls[0].lh < ls[1].lh (16 < 31)
- **Textmail pi=0**: ColumnDef + Table + 또 다른 표. ls[0].lh > ls[1].lh (744 > 126)

→ 가드 조건: **ls[0].lh < ls[1].lh** (헤더 표 + 후속 line 패턴 단독 식별)

## 정정 방향 (Stage 3 적용)

`layout.rs::layout_table_item` tac_seg_applied 분기에 가드 추가:

```rust
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

가드 3 조건:
1. `has_column_def`: ColumnDef 동반 (다단 영역 진입 paragraph)
2. `last.vpos > first.vpos`: 다중 LINE_SEG
3. `last.lh > first.lh`: 헤더 표 line 0 < 후속 line 1 패턴

## 다음 단계 (Stage 3 — GREEN)

instrument 제거 + 가드 적용 코드 + RED PASS 확인 + 회귀 검증.

## 승인 요청

Stage 2 분석 완료. Stage 3 GREEN 진입.
