# Task #624 구현 계획서

**제목**: exam_science p2 7번 글상자 ㉠ 사각형 y-위치 회귀 정정 — 3 line fix
**브랜치**: `local/task624`
**이슈**: https://github.com/edwardkim/rhwp/issues/624

---

## 1. 구현 대상 LOC 확정

`src/renderer/layout/table_layout.rs` (3 라인 정정).

### 1.1 Picture 분기 — `tac_img_y` 산식 (Task #520 복원)

위치: `Control::Picture(pic)` arm 안 `if pic.common.treat_as_char` 분기 — `target_line > current_tac_line` 블록 안.

현재 (HEAD `e9f3562`):
```rust
if let Some(seg) = para.line_segs.get(target_line) {
    tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
}
```

정정:
```rust
if let Some(seg) = para.line_segs.get(target_line) {
    // [Task #520 복원] LineSeg.vertical_pos 는 셀 origin 기준 절대값.
    // para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있어
    // 상대 오프셋만 더해야 한다 (Shape 분기와 동일).
    let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
    tac_img_y = para_y_before_compose
        + hwpunit_to_px(seg.vertical_pos - first_vpos, self.dpi);
}
```

### 1.2 Shape 분기 — `shape_area.y` (Task #520 복원)

위치: `Control::Shape(shape)` arm 안 `if shape.common().treat_as_char` 분기 — text_before 발행 직후, layout_cell_shape 호출 전.

현재:
```rust
let shape_area = LayoutRect {
    x: inline_x,
    y: para_y_before_compose,
    width: shape_w,
    height: inner_area.height,
};
self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, para_y_before_compose, Alignment::Left, styles, bin_data_content);
```

정정:
```rust
let shape_area = LayoutRect {
    x: inline_x,
    y: tac_img_y,
    width: shape_w,
    height: inner_area.height,
};
self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, tac_img_y, Alignment::Left, styles, bin_data_content);
```

## 2. TDD — Stage 1 RED 테스트

### 2.1 신규 통합 테스트 추가

`src/renderer/layout/integration_tests.rs` 끝에 `test_624_textbox_inline_shape_y_on_line2_p2_q7` 추가 — exam_science.hwp p2 7번 글상자의 ㉠ 사각형 y 좌표 검증.

```rust
#[test]
fn test_624_textbox_inline_shape_y_on_line2_p2_q7() {
    // exam_science.hwp p2 7번 글상자 (pi=33 ci=0) 안 p[1] 단락의
    // ㉠ 사각형 (treat_as_char + ls[1] 위치) y 좌표가 Line 2 영역에 와야 한다.
    // 회귀 (Task #520 부분 회귀) 시 사각형이 Line 1 (y≈213) 에 떨어져
    // 본문 텍스트 "분자당 구성" 위에 겹친다.

    let path = std::path::Path::new("samples/exam_science.hwp");
    let doc = crate::Document::load(path).expect("load");
    let tree = render_page_to_tree(&doc, 1 /* page index 0-based */);

    // ㉠ 사각형: Shape 노드 중 p[1] (pi=33 ci=0 cell paragraph[1])
    // 의 Rectangle ctrl 인 것을 식별
    let rect_y = find_inline_shape_y_in_textbox(&tree, /* section */ 0, /* paragraph */ 33);

    // Line 2 baseline ≈ 247.68, line top ≈ 235.65, sheet 22.88
    // 정상 범위: y ∈ [230, 240] (line 2 영역)
    // 회귀 범위: y ∈ [212, 218] (line 1 영역)
    assert!(rect_y >= 230.0 && rect_y <= 240.0,
        "rect y={} expected ~235.65 (Line 2), got Line 1 area (회귀)", rect_y);
}
```

> 테스트 헬퍼는 기존 패턴 (`render_page_to_tree`, `find_inline_shape_y_*`) 재사용. 헬퍼 부재 시 Stage 2 에서 SVG 텍스트 grep 으로 대체 측정.

대안: SVG snapshot 비교 방식 (`output/svg/exam_science_002.svg` 의 `<rect ... y="..."` 패턴 grep).

### 2.2 RED 확인

본 정정 적용 전 cargo test 실행 → 신규 테스트 RED.

## 3. 단계별 진행

### Stage 1 (현재): 구현 계획서 + RED TDD
1. 본 문서 작성 (완료)
2. TDD 통합 테스트 추가 → cargo test --lib RED 확인
3. Stage 1 보고서 (`task_m100_624_stage1.md`) 작성

### Stage 2: 정정 적용 (3 라인)
1. `src/renderer/layout/table_layout.rs` 3 라인 정정
2. cargo test --lib → RED → GREEN 전환 확인
3. cargo test --test svg_snapshot 6/6 통과
4. cargo clippy --lib -- -D warnings clean
5. 시각 판정 — `output/svg/exam_science_002.svg` 의 ㉠ 사각형 y 좌표 측정 (≈235.65)
6. Stage 2 보고서 (`task_m100_624_stage2.md`) 작성

### Stage 3: 광범위 회귀 검증 + 최종 보고
1. 광범위 fixture sweep — 164 fixture × ~10 페이지 SVG 생성 + 기존 baseline 대비 diff
2. 의도된 변경 (exam_science 7번 + 인접) vs 회귀 분리
3. exam_math/exam_eng/exam_kor/synam-001/21_언어 무회귀 확인
4. 최종 보고서 (`mydocs/report/task_m100_624_report.md`) 작성
5. orders 갱신

### Stage 4 (옵션): merge
- local/task624 → local/devel merge
- 작업지시자 승인 후 진행

## 4. 단위 테스트 매핑

기존 테스트 영향 없음으로 추정 (`tac_img_y` 산식은 `seg.vpos - first_vpos` 로 첫 paragraph (first_vpos=0) 케이스에서는 동일). 단:

- `test_544_passage_box_coords_match_pdf_p4`: 무관 (paragraph border 좌표)
- `test_547_passage_text_inset_match_pdf_p4`: 무관 (텍스트 inset)
- `test_548_cell_inline_shape_first_line_indent_p8`: **잠재 영향** — 셀 안 inline TAC Shape 위치
  - first-line 케이스이므로 first_vpos = ls[0].vpos = ls[0].vpos → `seg.vpos - first_vpos = 0` (line 0 그대로) → 영향 없을 것
- `test_552_passage_box_top_gap_p2_4_6`: 무관 (border push)

## 5. 회귀 위험 정밀 평가

### 5.1 Picture 분기 산식 변경

영향 케이스: `target_line > 0` 인 inline TAC Picture (multi-line paragraph 안 ls[1]+ 위치 picture).

- **변경 전**: `tac_img_y = para_y_before_compose + seg.vpos` (절대값 더함)
- **변경 후**: `tac_img_y = para_y_before_compose + (seg.vpos - first_vpos)` (상대 오프셋만)
- **첫 paragraph (first_vpos = 0)**: 동일 동작
- **두 번째+ paragraph (first_vpos > 0)**: ls[0].vpos 만큼 위로 이동 (이중 합산 해소)

회귀 가능 케이스: 셀 안 첫 paragraph 의 multi-line 인라인 picture — 거의 없음 (대부분 `target_line=0`).

### 5.2 Shape 분기 `shape_area.y` 변경

영향 케이스: 셀 안 multi-line paragraph + inline TAC Shape (사각형/원 등) — exam_science 7번이 대표.

- **변경 전**: `y = para_y_before_compose` (Line 0 위치)
- **변경 후**: `y = tac_img_y` (target_line 기반 위치)
- **target_line = 0** 케이스: `tac_img_y = para_y_before_compose` 로 동일 동작
- **target_line > 0** 케이스: 정상 line 위치로 이동 (회귀 해소)

## 6. 검증 체크리스트

- [ ] Stage 1: TDD 테스트 RED 확인
- [ ] Stage 2: 정정 적용 후 RED → GREEN
- [ ] Stage 2: cargo test --lib 회귀 0
- [ ] Stage 2: svg_snapshot 6/6 GREEN
- [ ] Stage 2: clippy clean
- [ ] Stage 2: exam_science p2 7번 ㉠ 사각형 y 좌표 시각 측정
- [ ] Stage 3: 164 fixture sweep — 의도된 변경 외 회귀 0
- [ ] Stage 3: exam_math/exam_eng/exam_kor/synam-001/21_언어 무회귀
- [ ] Stage 3: 최종 보고서 + orders 갱신

---

승인 시 Stage 1 (TDD RED 테스트 추가) 진행.
