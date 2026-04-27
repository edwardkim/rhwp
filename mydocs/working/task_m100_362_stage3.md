# Task #362 Stage 3 — 코드 수정 + 자동 회귀

## 수정 내용 (8 항목 누적)

본 task 의 진단 + 수정이 누적되며 옵션 A → 옵션 B 로 확장 (TypesetEngine 의 PartialTable + Square wrap 처리 깊은 결함 정정).

### 1. 외부 셀 vpos 가드 (`src/renderer/layout/table_layout.rs:1287-`)

nested table 이 있는 셀에서 LineSeg.vertical_pos 적용 제외:

```rust
let has_nested_table = cell.paragraphs.iter()
    .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
let text_y_start = if !has_nested_table && first_line_vpos.filter(|&v| v > 0.0).is_some() {
    cell_y + pad_top + first_line_vpos.unwrap()
} else {
    match effective_valign { ... }
};
```

**효과**: kps-ai p56 외부 표 안의 콘텐츠 클립 차단.

### 2. PartialTable nested table 분할 허용 (`src/renderer/layout/table_layout.rs:2113-`)

한 페이지보다 큰 nested table 의 atomic 미루기 대신 분할 표시:

```rust
let bigger_than_page = has_limit && para_h > content_limit;
let exceeds_limit = has_limit && para_end_pos > content_limit && !bigger_than_page;
```

**효과**: kps-ai p67 빈 페이지 (PartialTable 셀 안의 큰 nested table 표시 누락) 차단.

### 3. PartialTable 잔여 height 정확 계산

`calc_visible_content_height_from_ranges_with_offset` 신설 — split_start 시 nested table 의 잔여 높이를 content_offset 기반으로 정확히 계산.

**효과**: kps-ai p68 외곽 표 height 정상화.

### 4. nested table 셀 capping (`src/renderer/height_measurer.rs:1090-`)

`remaining_content_for_row` 의 nested table 분기에서 외부 행 높이로 cap:

```rust
let effective_total = if c.has_nested_table {
    capped.min(max_content.max(line_sum))
} else { capped };
return (effective_total - content_offset).max(0.0);
```

**효과**: PartialTable 의 typeset partial_height 계산 정확화 (84.56 px ≈ row 잔여).

### 5. hide_empty_line TypesetEngine 추가

`section.section_def.hide_empty_line` 을 typeset_section 에 전달 + Paginator 와 동일 로직 (페이지 시작 빈 줄 최대 2개 height=0):

```rust
if st.hide_empty_line && is_empty_para && st.current_height + fmt.height_for_fit > available
    && st.hidden_empty_lines < 2
{
    st.hidden_empty_lines += 1;
    st.hidden_empty_paras.insert(para_idx);
    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
    return;
}
```

### 6. wrap-around 메커니즘 (Square wrap) 이식 ★ 핵심

Paginator (`engine.rs:288-372`) 의 wrap-around 시멘틱 그대로 이식:

- **TypesetState 에 wrap_around_cs/sw/table_para 필드 + current_column_wrap_around_paras 추가**
- **Square wrap 표 직후 wrap zone 활성화** — `wrap_around_cs/sw = 표 host paragraph 의 LineSeg.column_start/segment_width`
- **후속 paragraph 가 동일 cs/sw 또는 sw=0 어울림 매칭이면 흡수** — `current_column_wrap_around_paras` 에 push, height 소비 없이 `continue`
- **flush_column 에서 wrap_around_paras 를 ColumnContent 로 전달**

**효과**: 외부 Square wrap 표 옆 paragraph (= HWP 어울림 영역) 가 height 소비 없이 흡수. kps-ai pi=675~748 의 74개 빈 paragraph 가 외부 표 옆에 흡수됨.

### 7. vpos-reset 가드 wrap zone 안 무시

```rust
if para_idx > 0 && !st.current_items.is_empty() && st.wrap_around_cs < 0 {
    // ... vpos-reset 가드 발동
}
```

**효과**: wrap zone 활성 중 vpos=0 paragraph 가 가드를 잘못 발동시키는 경우 차단.

### 8. Task #359 빈 paragraph skip 가드 강화

빈 텍스트지만 표/그림/도형 컨트롤 보유한 paragraph 는 skip 안 함:

```rust
let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
if is_empty_no_ctrl {
    continue;
} else {
    st.skip_safety_margin_once = true;
}
```

**효과**: pi=778 (빈 텍스트 + 3x3 wrap=Square 표) 누락 차단 — 표가 정상 표시.

## 자동 회귀 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1008 passed, 0 failed** |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

## 7 핵심 샘플 + 추가 회귀

| 샘플 | 페이지 (수정 전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-----|-----|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 77 → 77 | 3 → 3 |
| KTX | 27 → 27 | 1 → 1 |
| **k-water-rfp** | 28 → **27** | **0 → 0** |
| exam_eng | 11 → 11 | 0 → 0 |
| **kps-ai** | **88 → 79** | 60 → 5 |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

kps-ai 88 → 79 (한컴 의도 / Paginator 78 페이지에 거의 일치). LAYOUT_OVERFLOW 60→5 (대폭 개선).

## 시각 판정 (작업지시자 통과)

- **kps-ai p56**: 외부 표 안 콘텐츠 클립 차단 ✅
- **kps-ai p67**: PartialTable nested 표 정상 표시 (빈 페이지 차단) ✅
- **kps-ai p68**: 외곽 표 height 정상화 ✅
- **kps-ai p68-70**: 빈 페이지 2개 차단 → 정상 흐름 ✅
- **kps-ai p72-73**: pi=778 (3x3 Square wrap 표) 정상 표시 ✅

## 다음 단계 (Stage 4)

1. 최종 보고서 (`mydocs/report/task_m100_362_report.md`)
2. 트러블슈팅 (`mydocs/troubleshootings/typeset_partial_table_wrap_around.md`)
3. orders 갱신
4. 타스크 브랜치 커밋 + local/devel merge (작업지시자 승인 후)
