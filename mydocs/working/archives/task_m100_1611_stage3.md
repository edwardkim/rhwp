# Stage 3 완료보고서 — Task #1611 (수정 GREEN)

**단계**: footer Page+Bottom page-fit 정합 · **브랜치**: `local/task1611`

## 수정 (`src/renderer/typeset.rs`)

generic fit 체크 직전(Paper/Para 특수처리 다음)에 Page+Bottom 블록 처리 추가:

```rust
let is_page_bottom_topbottom_block = !table.common.treat_as_char
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Page)
    && matches!(table.common.vert_align, VertAlign::Bottom);
if is_page_bottom_topbottom_block && st.current_column == 0 {
    let target_y    = vpos(host para first line_seg);     // 640.7px
    let declared_px = px(table.common.height);            // 351.4px (선언)
    let block_height= table_total.max(declared_px);       // 측정 302.3 대신 선언 351.4
    let sync_h      = current_height.max(target_y);        // vpos 동기화
    if sync_h + block_height <= available { current_height = sync_h; }
    else if !current_items.is_empty() { advance_column_or_new_page(); }
    place_table_with_text(..., block_height, ...); return;
}
```

## 진단으로 확정한 2요소 (디버그 RHWP_DBG_1611)

```
DBG1611 pi=11 vpos=48053 target_y=640.7 cur_h=627.5 sync_h=640.7 table_total=302.3 avail=990.2
```
- ① vpos 미동기화: cur_h 627.5 < vpos 640.7 (Paper 만 동기화, Page 누락).
- ② 선언≠측정: `table_total=302.3`(셀 내용 측정) vs 선언 351.4 → fit 과소.
- 두 보정 동시 적용 시: 640.7 + 351.4 = **992.1 > 990.2 → 분할(2쪽)**.

## GREEN
```
test issue_1611_footer_page_bottom_splits_to_second_page ... ok
```
SVG 시각 확인: page1 본문 / page2 발신명의 블록(서울특별시장·수신자·협조자·시행·전화) 단독.

> Stage 2(RED)+Stage 3(fix)는 broken commit 방지를 위해 하나의 구현 커밋으로 GREEN 기록.
