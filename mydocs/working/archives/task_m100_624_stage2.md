# Task #624 Stage 2 보고서

## 목적

Task #520 부분 회귀 정정 — Picture 산식 + Shape `shape_area.y` + Shape `layout_cell_shape` para_y 인자 3 라인 복원.

## 적용 변경

`src/renderer/layout/table_layout.rs` 3 영역 정정.

### 1. Picture 분기 `tac_img_y` 산식 복원 (line 1605~1607)

**Before**:
```rust
if let Some(seg) = para.line_segs.get(target_line) {
    tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
}
```

**After**:
```rust
if let Some(seg) = para.line_segs.get(target_line) {
    // [Task #520 / #624 복원] LineSeg.vertical_pos 는 셀 origin 기준 절대값.
    // para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있어
    // 상대 오프셋(seg.vpos - ls[0].vpos)만 더해야 이중 합산을 피한다.
    let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
    tac_img_y = para_y_before_compose
        + hwpunit_to_px(seg.vertical_pos - first_vpos, self.dpi);
}
```

### 2. Shape 분기 `shape_area.y` + `layout_cell_shape` para_y 인자 (line 1814~1820)

**Before**:
```rust
let shape_area = LayoutRect {
    x: inline_x,
    y: para_y_before_compose,
    width: shape_w,
    height: inner_area.height,
};
self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, para_y_before_compose, Alignment::Left, styles, bin_data_content);
```

**After**:
```rust
// [Task #520 / #624 복원] target_line 기반 tac_img_y 사용 (Picture 분기와 동일).
// para_y_before_compose 사용 시 multi-line paragraph 의 ls[1]+ inline TAC Shape 가
// 항상 line 0 좌표에 떨어져 본문 텍스트와 겹친다 (exam_science p2 7번 글상자 ㉠).
let shape_area = LayoutRect {
    x: inline_x,
    y: tac_img_y,
    width: shape_w,
    height: inner_area.height,
};
self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, tac_img_y, Alignment::Left, styles, bin_data_content);
```

## 검증

### 단위 / 통합 테스트

| 테스트 | 결과 |
|---|---|
| `cargo test --lib test_624_textbox_inline_shape_y_on_line2_p2_q7` | **GREEN** (RED → GREEN) |
| `cargo test --lib` (전체) | **1135 passed / 0 failed / 2 ignored** |
| `cargo test --test svg_snapshot` | **6/6 passed** |
| `cargo clippy --lib -- -D warnings` | **clean** |

### 광범위 회귀 sweep (12 fixture, 102 페이지)

| 샘플 | 페이지 | 변경 | 비고 |
|---|---|---|---|
| `exam_science.hwp` | 4 | **1** | 의도된 정정 (page 2 ㉠) |
| `exam_kor.hwp` | 20 | 0 | 무회귀 |
| `exam_math.hwp` | 20 | 0 | 무회귀 |
| `exam_eng.hwp` | 8 | 0 | 무회귀 |
| `synam-001.hwp` | 35 | 0 | 무회귀 (multi-line + tac rect 3 case 무영향) |
| `21_언어_기출_편집가능본.hwp` | 15 | 0 | 무회귀 |
| **합계** | **102** | **1 (의도)** | **회귀 0** |

추가 sweep 권고 (158 sample 전체 fixture) 는 Stage 3 에서 수행.

### 정정 효과 (exam_science p2 7번 글상자 ㉠ 사각형)

```diff
-<rect x="117.066" y="213.946" width="62.986" height="22.880" fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
-<text x="141.56"  y="229.986" ...>㉠</text>
+<rect x="117.066" y="235.413" width="62.986" height="22.880" fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
+<text x="141.56"  y="251.453" ...>㉠</text>
```

- `<rect>` y: 213.95 → 235.41 (Δ +21.47 px = ls[1].vpos − ls[0].vpos / 75 정확)
- `<text>㉠` y: 229.99 → 251.45 (Δ +21.47 px)

→ ㉠ 사각형이 Line 1 영역 (본문 "분자당 구성" 위 겹침) 에서 Line 2 영역 (" 이다." 앞) 으로 정확히 이동.

## 코드 영향

| 파일 | 변경 LOC |
|---|---|
| `src/renderer/layout/table_layout.rs` | +9 / -2 (Picture 산식 + Shape 위치) |
| `src/renderer/layout/integration_tests.rs` | +59 (Stage 1 RED 테스트 — 이미 commit) |

## 잔존

- Stage 3: 158 sample 전체 fixture sweep (광범위 회귀 검증) + 최종 보고서 작성

## 다음 단계

Stage 3 진입 — 광범위 fixture sweep + orders 갱신 + 최종 보고서 작성.
