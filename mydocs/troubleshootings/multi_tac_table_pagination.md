---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 한 문단 내 다중 TAC 표 페이지네이션 오류

## 발견 경위

- **파일**: `samples/p222.hwp` 68페이지 (`output/p222_068.svg`)
- **증상**: 한컴에서는 5개 표가 모두 한 페이지에 렌더링되지만, 우리 렌더러는 5번째 표를 다음 페이지로 분할(PartialTable)
- **원인 문단**: pi=103 — 한 문단에 TAC 표 컨트롤 2개 (ctrl[0]: 6×5, ctrl[1]: 14×5)

## 근본 원인

### 1. 페이지네이션: `para_height_for_fit` 합산 문제

`pagination/engine.rs`의 `paginate_table_control()`에서 TAC 표의 높이 판단:

```rust
let table_total_height = if is_tac_table && para_height > 0.0 {
    para_height_for_fit  // ← 문단 전체 높이 (두 표 합산)
} else {
    effective_height + host_spacing
};
```

- `para_height_for_fit`은 **문단 전체 높이** (ctrl[0] + ctrl[1] + 캡션 + 간격 합산)
- `process_controls()`에서 ctrl[0], ctrl[1]을 순회하며 각각 `paginate_table_control()`을 호출하지만, 둘 다 동일한 `para_height_for_fit`을 전달받음
- ctrl[1] 처리 시: `st.current_height + (두 표 합산 높이) > available_height` → 초과 판정 → 불필요한 분할 발생

**수정**: 문단 내 TAC 표가 2개 이상이면 `para_height_for_fit` 대신 개별 `effective_height + host_spacing` 사용

```rust
let tac_table_count = para.controls.iter()
    .filter(|c| matches!(c, Control::Table(t) if t.attr & 0x01 != 0))
    .count();
let table_total_height = if is_tac_table && para_height > 0.0 && tac_table_count <= 1 {
    para_height_for_fit
} else {
    effective_height + host_spacing
};
```

### 2. 레이아웃: 같은 문단 내 표 간격 불일치

`layout.rs`의 TAC line_seg 줄간격 처리에서:

- 별도 문단 표 간격: `line_spacing / 2` = 4px
- 같은 문단 표 간격: `line_seg[1].vertical_pos`로 점프 → full `line_spacing` = 8px (2배)

원인: `line_seg[1].vertical_pos = line_seg[0].vertical_pos + line_seg[0].line_height + line_spacing`으로 계산되어 전체 line_spacing이 포함됨

**수정**: `next_seg.vertical_pos` 점프 제거, `control_index` 기반 line_seg 선택 후 일관된 `line_spacing / 2` 적용

```rust
if is_tac {
    let seg_idx = *control_index;
    if let Some(seg) = para.line_segs.get(seg_idx) {
        let line_end = col_area.y
            + hwpunit_to_px(seg.vertical_pos + seg.line_height, self.dpi);
        if line_end > y_offset {
            y_offset = line_end;
        }
    }
    if let Some(seg) = para.line_segs.get(seg_idx) {
        y_offset += hwpunit_to_px(seg.line_spacing, self.dpi) / 2.0;
    }
    tac_seg_applied = true;
}
```

## 핵심 교훈

1. **한 문단에 여러 TAC 표 컨트롤이 존재할 수 있다** — pi=103처럼 개행 없이 연속된 표
2. 페이지네이션에서 **문단 높이를 개별 컨트롤의 적합성 판단에 사용하면 안 된다** (다중 컨트롤 문단에서 합산 오류)
3. line_seg의 `vertical_pos`에는 **line_spacing이 포함**되어 있으므로, 표 간격 계산 시 직접 사용하면 간격이 2배가 될 수 있다

## 관련 파일

| 파일 | 수정 내용 |
|---|---|
| `src/renderer/layout.rs` | TAC 표 line_seg 줄간격: control_index 기반 seg 선택 + line_spacing/2 일관 적용 |
| `src/renderer/pagination/engine.rs` | 다중 TAC 표 문단: 개별 effective_height 사용 |

## 검증

- `cargo test`: 608개 전체 통과
- `p222_068.svg`: 5개 표 + 캡션 모두 한 페이지에 렌더링
- 총 페이지: 123 → 122 (불필요한 분할 페이지 제거)
- 한컴 출력과 동일한 페이지 구분 확인
