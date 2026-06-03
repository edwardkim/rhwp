# Task #157 단계 1 완료보고서: layout.rs vpos 기준점 리셋 예외 처리

> 단계 1 완료보고서 | 2026-04-24  
> Issue: #157  
> Branch: `local/task157`

---

## 수정 내용

**파일**: `src/renderer/layout.rs` lines 1445–1454

Para-relative float 표(vert=Para, TopAndBottom, non-TAC)는 앵커 문단에 attach되므로
후속 문단의 vpos 교정 기준점(`vpos_page_base`, `vpos_lazy_base`)을 초기화하지 않도록 예외 처리.

### 변경 전

```rust
let is_table_or_shape = matches!(item,
    PageItem::Table { .. } | PageItem::PartialTable { .. } | PageItem::Shape { .. });
if was_tac || is_table_or_shape {
    vpos_page_base = None;
    vpos_lazy_base = None;
}
```

### 변경 후

```rust
let is_table_or_shape = matches!(item,
    PageItem::Table { .. } | PageItem::PartialTable { .. } | PageItem::Shape { .. });
let is_para_float_table = if let PageItem::Table { para_index, control_index } = item {
    paragraphs
        .get(*para_index)
        .and_then(|p| p.controls.get(*control_index))
        .map(|c| {
            matches!(
                c,
                Control::Table(t)
                if !t.common.treat_as_char
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                    && matches!(t.common.vert_rel_to, VertRelTo::Para)
            )
        })
        .unwrap_or(false)
} else {
    false
};
if was_tac || (is_table_or_shape && !is_para_float_table) {
    vpos_page_base = None;
    vpos_lazy_base = None;
}
```

---

## 결과 (단계 2·3 포함)

**단계 2** — `engine.rs` effective_table_height 방어 코드도 함께 적용:
- Para-relative float 표가 body 범위 내에 완전히 들어오면 `effective_table_height = 0.0`

**단계 3** — 검증:

| 항목 | 결과 |
|------|------|
| `cargo test` | ✅ 941+4 = 945 passed, 0 failed |
| `dump-pages issue_157.hwpx -p 1` | ✅ Table pi=25 LAYOUT_OVERFLOW 없음 |
| Table pi=25 SVG 위치 | ✅ y=819.2px (이전: 894.7px 오클리핑) |
| Golden SVG 등록 | ✅ `tests/golden_svg/issue-157/page-1.svg` |
| 기존 테스트 regression | ✅ 없음 |

**남은 pi=28 overflow (9.6px)**:
- 수정 전과 동일한 y=1102.9, overflow=9.6px
- 단, 대상이 Table pi=25 → FullParagraph pi=28으로 변경됨
- 문서 자체 내용이 페이지 하단에 밀착된 기존 상태 (issue #157 버그와 무관)
- table pi=25가 y=819.2에 올바르게 배치된 후 pi=26~28이 표 하단 아래에 이어지며 마지막 줄이 body_bottom을 9.6px 초과

---

## 수정 파일 목록

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | is_para_float_table 예외 처리 (+18줄) |
| `src/renderer/pagination/engine.rs` | effective_table_height = 0.0 방어 코드 (+4줄) |
| `tests/svg_snapshot.rs` | issue_157_page_1 테스트 추가 |
| `tests/golden_svg/issue-157/page-1.svg` | golden SVG 신규 등록 |
