# 구현 계획서 V2 — Task #959 Stage 2 — Fix C 적용 계획

- 이슈: [#959](https://github.com/edwardkim/rhwp/issues/959)
- Stage 1 결과: pi=69 picture (horz_rel_to=Column, Center align + h_offset 300px) 이 col_area 밖 (x=767, col_right=759) 에 emit, 그러나 cursor 는 column 안에서 +274px advance
- 선택: **Fix C** — picture emit x 가 column 밖 시 cursor advance skip

## 1. 변경 위치

`src/renderer/layout.rs:3500-3554` (non-TAC picture column 처리 분기).

## 2. 변경 내용

### Before
```rust
} else {
    let comp = composed.get(para_index);
    let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
    let alignment = styles.para_styles.get(para_style_id)
        .map(|s| s.alignment)
        .unwrap_or(Alignment::Left);
    let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
    let pic_container = LayoutRect {
        x: col_area.x, y: pic_y,
        width: col_area.width,
        height: col_area.height - (pic_y - col_area.y),
    };
    result_y = self.layout_body_picture(
        tree, col_node, pic,
        &pic_container, col_area, &layout.body_area,
        &LayoutRect { x: 0.0, y: 0.0, width: layout.page_width, height: layout.page_height },
        bin_data_content, styles, alignment, pic_y,
        page_content.section_index, para_index, control_index,
    );
    ...
}
```

### After
```rust
} else {
    let comp = composed.get(para_index);
    let para_style_id = comp.map(|c| c.para_style_id as usize).unwrap_or(para.para_shape_id as usize);
    let alignment = styles.para_styles.get(para_style_id)
        .map(|s| s.alignment)
        .unwrap_or(Alignment::Left);
    let pic_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
    let pic_container = LayoutRect {
        x: col_area.x, y: pic_y,
        width: col_area.width,
        height: col_area.height - (pic_y - col_area.y),
    };
    let saved_y_offset = y_offset;
    result_y = self.layout_body_picture(
        tree, col_node, pic,
        &pic_container, col_area, &layout.body_area,
        &LayoutRect { x: 0.0, y: 0.0, width: layout.page_width, height: layout.page_height },
        bin_data_content, styles, alignment, pic_y,
        page_content.section_index, para_index, control_index,
    );
    // [Task #959] horz_rel_to=Column 의 picture 가 col_area 우측을 초과하는
    // 위치 (예: h_offset + pic_width > col_area.width) 에 emit 되면 한컴 viewer
    // 는 해당 picture 를 column flow 에 reservation 하지 않음. rhwp 는 cursor
    // 를 picture height 만큼 advance → 후속 paragraph 처짐.
    // (sample 3-11월_실전_통합_2022 page 1 우측 단 pi=69 picture +274px advance
    // → 문9 +250px 처짐).
    // Picture 의 좌측 edge (x) 가 col_area 우측 (col_area.x + col_area.width)
    // 을 초과하면 column flow advance skip.
    if matches!(pic.common.horz_rel_to, HorzRelTo::Column) {
        let pic_width_px = hwpunit_to_px(pic.common.width as i32, self.dpi);
        let h_offset_px = hwpunit_to_px(pic.common.horizontal_offset as i32, self.dpi);
        let pic_emit_x = match pic.common.horz_align {
            crate::model::shape::HorzAlign::Left | crate::model::shape::HorzAlign::Inside =>
                col_area.x + h_offset_px,
            crate::model::shape::HorzAlign::Center =>
                col_area.x + (col_area.width - pic_width_px) / 2.0 + h_offset_px,
            crate::model::shape::HorzAlign::Right | crate::model::shape::HorzAlign::Outside =>
                col_area.x + col_area.width - pic_width_px - h_offset_px,
        };
        if pic_emit_x >= col_area.x + col_area.width {
            // Picture 의 좌측 edge 가 column 우측 외부 → column flow advance skip
            result_y = saved_y_offset;
        }
    }
    // 기존 Task #683, Square wrap fallback 등 후속 처리 유지
    ...
}
```

## 3. 영향 분석

### 3.1 변경 직접 영향
- pi=69 picture (horz_rel_to=Column + h_offset 300px + Center align + pic_width 224px) — pic_emit_x=767, col_area.right=759 → advance skip → 274px advance 제거 → 문9 정상 위치
- horz_rel_to=Column 인 picture 가 column 내부에 위치하는 경우: pic_emit_x < col_area.right → advance 유지 (기존 동작)

### 3.2 다른 sample 영향 (예상)
- 일반 column 내부 picture: 영향 없음 (advance 유지)
- horz_rel_to=Paper/Page picture: 영향 없음 (is_paper_based 분기)
- horz_rel_to=Para picture: 영향 없음 (Column 검사 false)

## 4. 위험 평가

| 위험 | 평가 | 완화 |
|------|------|------|
| Column 외부 emit picture 가 사실 column 안에 의도된 case | **낮음** (pic_emit_x >= col_right 면 명확히 외부) | Stage 4 다중 sample 검증 |
| 한컴 의도와 다른 케이스 | **중** | 한컴 PDF 정합 확인 |
| 회귀 가능성 | **낮음** (column 내부 picture 영향 없음) | cargo test 1288 + 다중 sample |

## 5. 검증 계획 (Stage 3-4)

### Stage 3 단위 검증
1. cargo build --release
2. 시험지 3-11월 page 1 SVG render:
   - pi=69 picture cursor advance 0 (TAC_CURSOR)
   - 문9 y 위치 ~810 (한컴 정합)
3. PNG render → 한컴 PDF 정합

### Stage 4 회귀 검증
1. `cargo test --release --lib` 전체 (1288 tests)
2. 추가 sample:
   - 시험지 4종 (3-09월/3-10월/3-11월) page 1
   - sample16 (HWP3)
   - exam_kor/math/eng (column-based picture 가능성)
   - hwp3-sample10/11/13/14
3. golden SVG diff 회귀 0

## 6. Stage 5 (시각 검증 + 최종 보고서 + PR)

- 한컴 PDF 정합 비교
- rhwp-studio UI 시각 확인
- commit + PR

## 7. 진행 규칙

- 자동진행 안함
- 각 stage 종료 시 보고서 + 명시 승인
