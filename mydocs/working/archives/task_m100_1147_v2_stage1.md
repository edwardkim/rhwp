# Task #1147 v2 Stage 1 보고서 — LayoutEngine is_hwpx_source 신설 + 산식 적용

- **수행계획서**: [task_m100_1147_v2.md](../plans/task_m100_1147_v2.md)
- **구현계획서**: [task_m100_1147_v2_impl.md](../plans/task_m100_1147_v2_impl.md)

## 1. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | `LayoutEngine.is_hwpx_source: Cell<bool>` 필드 신설 + `LayoutEngine::new` 초기화 + `set_hwpx_source()` 메서드 추가 + `layout_table_item` 의 "표 아래 간격" 분기에 HWPX 한정 `gap=0` 트리거 추가 |
| `src/document_core/queries/rendering.rs` | `find_page` 호출부에서 `set_hwpx_source(matches!(source_format, FileFormat::Hwpx))` 1 줄 추가 |
| `src/renderer/typeset.rs` | ad-hoc LayoutEngine 인스턴스 (advance_row_cut 측정용) 도 `set_hwpx_source(st.is_hwpx_source)` 동기화 |

## 2. 주요 변경 코드

### `layout.rs` — 산식 보정 (라인 4082-4093 부근)

```rust
// [Task #1147 v2] HWPX 원본의 빈 앵커 TopAndBottom 비-TAC 표는 typeset
// 측 is_topbottom_empty_anchor_hwpx 보정으로 host_line_spacing=0 처리되므로,
// 렌더러도 앵커 line_spacing 을 표 아래 갭으로 가산하지 않는다. 가산 시
// typeset 의 cur_h 와 layout 의 y_offset 가 18 px 어긋나 표 직후 문단이
// 시각상 아래로 밀려난다 (작업지시자 시각 검수, 권위 PDF 정합).
let is_topbottom_empty_anchor_hwpx =
    self.is_hwpx_source.get() && is_current_empty_para_float;
if let Some(seg) = para.line_segs.last() {
    let gap = if is_topbottom_empty_anchor_hwpx {
        0
    } else if is_current_empty_para_float {
        seg.line_spacing.max(0)
    } else if seg.line_spacing > 0 {
        seg.line_spacing
    } else {
        seg.line_height
    };
    if gap > 0 {
        y_offset += hwpunit_to_px(gap, self.dpi);
    }
}
```

## 3. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --bin rhwp` | ✓ pass (warning 없음) |

## 4. 다음 단계

Stage 2: 본 페이지 SVG 시각 검증 + `dump-pages` items 유지 확인 + `cargo test` 전수 통과 + golden SVG 회귀.
