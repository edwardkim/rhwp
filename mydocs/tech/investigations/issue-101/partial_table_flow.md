---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-101/README.md
last_verified: 2026-07-17
---

# PartialTable 분할 처리 흐름도

> 작성일: 2026-04-10
> 관련 이슈: #101

---

## 전체 처리 흐름

```
┌─────────────────────────────────────────────────────────────────────┐
│  페이지네이션 단계 (engine.rs)                                        │
│                                                                     │
│  paginate_table_control()                                           │
│  ├─ ① host_spacing 계산  (라인 989-1032)                            │
│  │     host_spacing = before + sa + outer_bottom + host_line_spacing│
│  │     before = spacing_before + outer_top  (조건부)                 │
│  │     spacing_before_px = before - outer_top  ← 레이아웃 전진량     │
│  │     is_column_top = st.current_height < 1.0                     │
│  │     ※ is_column_top이면 spacing_before = 0                      │
│  │                                                                  │
│  ├─ ② table_total_height 계산  (라인 1038-1072)                     │
│  │     비-TAC: effective_height + host_spacing                      │
│  │                                                                  │
│  ├─ ③ table_available_height 계산  (라인 984)                       │
│  │     = base_available_height - footnote - zone_y_offset          │
│  │                                                                  │
│  ├─ ④ 조기 새 페이지 이동 체크  (라인 1329-1343)                     │
│  │     remaining_on_page = table_avail - current_height            │
│  │     if remaining < first_row_h → advance_column_or_new_page()   │
│  │     ※ 이동 시 current_height 리셋 → is_column_top 변경될 수 있음  │
│  │                                                                  │
│  └─ ⑤ 피트 판정  (라인 1097)                                        │
│        current_height + effective_table_height <= table_available_h │
│                    │                         │                      │
│            ┌───────┘                         └──────┐              │
│            ▼ 전체 배치                             ▼ 분할 배치    │
│     place_table_fits()                  split_table_rows()         │
│     └─ current_height +=                (아래 상세 참조)            │
│           table_total_height                                        │
│        (= effective_h + spacing_before                             │
│           + host_line_spacing)                                      │
│        ← 레이아웃과 일치: layout_table_item도                        │
│          spacing_before + host_line_spacing 추가함                  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                    split_table_rows() 상세
                              │
┌─────────────────────────────────────────────────────────────────────┐
│  split_table_rows() — 행 단위 분할 루프  (라인 1379~)               │
│                                                                     │
│  [첫 분할 / cursor_row=0 / is_continuation=false]                   │
│                                                                     │
│  ⑥ page_avail 계산  (라인 1406-1413)                                │
│     = table_available_height - current_height                       │
│       - caption_extra - host_extra - v_extra                        │
│                                                                     │
│  ⑦ avail_for_rows 계산  (라인 1421-1429)                            │
│     = page_avail - header_overhead - spacing_before_px             │
│     ※ spacing_before_px: 레이아웃이 표 배치 전 y_offset 전진량       │
│                                                                     │
│  ⑧ find_break_row(avail_for_rows, cursor_row, effective_first_row_h)│
│     (height_measurer.rs 라인 1165)                                  │
│     → approx_end (배치할 행의 끝 인덱스, exclusive)                   │
│     ★ MeasuredTable.row_heights[r] 에 음수 HU 셀 높이가              │
│       포함되면 cumulative_heights가 실제보다 작아져                    │
│       더 많은 행을 선택하게 됨  ← pi=181 문제의 직접 원인             │
│                                                                     │
│  ⑨ end_row 결정 + 행 내부 분할 판정                                   │
│                                                                     │
│  ⑩ partial_height 계산  (range_height 사용)                         │
│     = mt.range_height(cursor_row, end_row)  ← O(1) prefix sum      │
│     ★ 음수 행 높이가 포함되면 partial_height도 실제보다 작게 계산됨   │
│                                                                     │
│  ⑪ PageItem::PartialTable 생성                                      │
│     { start_row, end_row, is_continuation, ... }                   │
│                                                                     │
│  ⑫ current_height 업데이트                                          │
│     중간 파트: advance_column_or_new_page() → 다음 페이지 리셋       │
│     마지막 파트: current_height += partial_height + sa               │
│                                                                     │
│  [연속 분할 / is_continuation=true]                                  │
│     page_avail = base_available_height (새 페이지 전체 가용)          │
│     spacing_before_px 미적용 (sb_extra = 0)                         │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│  레이아웃 단계 (layout.rs → table_partial.rs)                         │
│                                                                     │
│  PageItem::Table { } → layout_table_item()  (라인 1758-1762)        │
│  └─ 내부에서 spacing_before, host_line_spacing 모두 처리             │
│     (라인 1844-1850, 2040-2052)                                      │
│     반환 후 즉시 return → 라인 2040 이후 코드 미실행                  │
│                                                                     │
│  PageItem::PartialTable { } → layout_partial_table_item()           │
│                              (라인 1764-1772)                        │
│  ├─ ⑬ 표 배치 전 spacing_before 적용  (layout.rs 라인 1820-1850)    │
│  │     비-TAC TopAndBottom:                                          │
│  │       if !is_column_top: y_offset += spacing_before             │
│  │     ※ 실제 레이아웃에서 y_offset 전진 발생                         │
│  │                                                                  │
│  ├─ ⑭ layout_partial_table() 호출  (table_partial.rs)              │
│  │     첫 배치(is_continuation=false): table_y = y_start           │
│  │     분할 행 높이 오버라이드:                                        │
│  │       split_start_content_offset: start_row 높이 재계산           │
│  │       split_end_content_limit: end_row-1 높이 제한               │
│  │     ★ 음수 HU 셀 높이는 px 변환 후 0.0으로 처리되거나              │
│  │       레이아웃에서 실제 콘텐츠 기반으로 재계산됨                     │
│  │     반환값: table_bottom (실제 렌더 높이 기반)                      │
│  │                                                                  │
│  └─ ⑮ 배치 후 y_offset 전진  (layout.rs 라인 2206-2229)            │
│        + spacing_after (있으면)                                      │
│        ※ host_line_spacing 미추가  (Table과 다름!)                   │
│          → 단, PartialTable 뒤에 실제로 host_line_spacing가          │
│            필요한지는 문맥에 따라 다름                                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 확인된 불일치 포인트

### 포인트 1: spacing_before (v2 fix로 해결)

```
pagination page_avail = table_avail - current_height
                        ← current_height에 spacing_before 미포함
레이아웃 실제 표 시작 y = current_y + spacing_before
                        ← 레이아웃이 y_offset을 spacing_before만큼 전진

해결: avail_for_rows -= spacing_before_px  (v2 fix)
```

### 포인트 2: 음수 셀 높이 HU 값 (미해결 — pi=181의 실제 원인)

```
HWP 파일의 일부 셀 h 필드가 u32 오버플로우 값 (= 음수 i32):
  pi=181, r=5, c=2: h=4294966114 HU (= -182 as i32)
  pi=181, r=13, c=2: h=4294961386 HU (= -5398 as i32)

hwpunit_to_px(h as i32, dpi)으로 변환 시:
  -182 / 75 = -2.4px
  -5398 / 75 = -71.97px

MeasuredTable.row_heights[5] ← 이 셀을 포함한 행의 높이
row_heights[r] = max(셀 높이들) 이어야 하나,
같은 행의 다른 셀(c=3, h=1182)이 정상값이면 max로 보정될 수 있음.

그런데 r=5의 c=0,1 셀이 r=3의 병합 셀(rs=3)에 속해있어
r=5 자체의 셀은 c=2, c=3만 존재.
  c=2: h=-182px → 음수
  c=3: h=1182 HU = 15.8px → 정상

row_heights[5] = max(-2.4, 15.8) = 15.8px 이어야 정상.
그러나 병합 셀 처리나 MeasuredTable 계산에서 오류가 있을 수 있음.
```

### 포인트 3: PartialTable 배치 후 host_line_spacing 미추가

```
PageItem::Table → layout_table_item → host_line_spacing 추가 (라인 2048)
PageItem::PartialTable → layout_partial_table_item → host_line_spacing 미추가

그런데 pagination split_table_rows에서도:
  중간 파트: advance_column_or_new_page() 후 current_height 리셋
  마지막 파트: current_height += partial_height + sa  (host_line_spacing 미포함)

→ PartialTable 마지막 파트에서는 pagination과 레이아웃 모두 host_line_spacing 미포함
   → 일치하므로 문제 없음
```

---

## 결론: pi=181 LAYOUT_OVERFLOW의 실제 원인

**음수 HU 셀 높이로 인한 `MeasuredTable.row_heights` 오차 가능성**이 의심되나,
max 처리로 자동 보정될 경우 다른 원인일 수 있음.

**정확한 원인 확정을 위해 MeasuredTable 계산 로직 추가 검토 필요:**
- `height_measurer.rs`에서 row_heights 계산 시 음수 HU 처리 방식
- r=3 (rs=3 병합 셀)이 r=3, r=4, r=5에 걸칠 때 row_heights 분배 방식

---

## 관련 파일 및 라인

| 처리 | 파일 | 주요 라인 |
|------|------|----------|
| host_spacing 계산 | engine.rs | 989-1032 |
| table_total_height 계산 | engine.rs | 1038-1072 |
| 조기 새 페이지 이동 체크 | engine.rs | 1329-1343 |
| 피트 판정 | engine.rs | 1097 |
| place_table_fits current_height | engine.rs | 1224-1232, 1278 |
| split_table_rows page_avail | engine.rs | 1406-1417 |
| split_table_rows avail_for_rows | engine.rs | 1421-1429 |
| find_break_row | height_measurer.rs | 1165-1177 |
| range_height | height_measurer.rs | 1182-1188 |
| row_heights 계산 (MeasuredTable) | height_measurer.rs | 측정 로직 |
| PageItem::Table 분기 (return) | layout.rs | 1758-1762 |
| PageItem::PartialTable 분기 | layout.rs | 1764-1772 |
| 표 배치 전 spacing_before | layout.rs | 1820-1850 |
| layout_table_item 배치 후 host_ls | layout.rs | 2040-2052 |
| layout_partial_table_item | layout.rs | 2120-2254 |
| layout_partial_table | table_partial.rs | 28-600+ |
