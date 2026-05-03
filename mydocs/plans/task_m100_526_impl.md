# Task #526 구현 계획서 — `layout_inline_table_paragraph` TAC 수식 처리 추가 (A안)

## 1. 개요

Stage 1 진단 (`mydocs/working/task_m100_526_stage1.md`) 결과에 따라 `paragraph_layout.rs:88 layout_inline_table_paragraph` 가 인라인 표 외 TAC 컨트롤(수식·Picture·Form)도 처리하도록 확장한다. 현재는 `inline_tables` 만 처리해 segments 루프에서 수식들이 누락 → `set_inline_shape_position` 등록 실패 → shape pass fallback → 한 점 stack.

본 task 의 영향 범위는 5개 문단(pi=61, 79, 110, 118, 120) 이며 모두 **표 + 수식** 조합. **Picture/Form 은 현 샘플에 케이스 없음** 이므로 본 구현은 **수식 처리에 집중**, Picture/Form 은 동일 패턴으로 확장 가능한 구조로만 남긴다 (실제 호출 케이스 발견 시 별도 task).

## 2. 코드 변경 위치 한정

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/renderer/layout/paragraph_layout.rs` | 117-124 | `inline_tables` → `inline_tac_controls` (TAC 컨트롤 통합 추출) |
| 동 | 186-204 | `table_widths` → `tac_widths` (모든 TAC 컨트롤 폭 통합 계산) |
| 동 | 232 | `total_width` 계산식: `seg_widths + tac_widths` |
| 동 | 280-563 | segments 루프: 표 분기 → TAC 컨트롤 종류별 분기 (표/수식). 수식 처리 시 `EquationNode` 생성 + `set_inline_shape_position` 등록 |
| 동 | 565-591 | 후행 표 처리 → 후행 TAC 컨트롤 처리 (수식 포함) |

다른 파일은 손대지 않는다 (`shape_layout.rs`, `composer.rs`, `layout.rs` 모두 무수정).

## 3. 단계 (3단계)

### Stage 2 — 정정 구현

**핵심 변경**: `inline_tables: Vec<(usize, &Table)>` 를 enum 기반 통합 컬렉션으로 대체.

```rust
enum InlineTac<'a> {
    Table(&'a Table),
    Equation(&'a Equation),
    // 향후 확장: Picture, Form
}

let inline_tac_controls: Vec<(usize, InlineTac)> = para.controls.iter().enumerate()
    .filter_map(|(i, c)| match c {
        Control::Table(t) if t.common.treat_as_char => Some((i, InlineTac::Table(t.as_ref()))),
        Control::Equation(eq) => Some((i, InlineTac::Equation(eq.as_ref()))),
        _ => None,
    })
    .collect();
```

(주의: `Equation` 의 `treat_as_char` 는 항상 true 로 가정 — composer.rs:130-133 도 위치만 보고 무조건 포함하므로 일관성 있음. 검증 필요시 `composer.rs::find_control_text_positions` 와 동일한 필터 사용.)

**폭 계산 통합**:
```rust
let tac_widths: Vec<f64> = inline_tac_controls.iter().map(|(_, kind)| {
    match kind {
        InlineTac::Table(t) => calc_table_width(t),  // 기존 184-204 로직 추출
        InlineTac::Equation(eq) => hwpunit_to_px(eq.common.width as i32, self.dpi),
    }
}).collect();
```

**segments 루프 변경**: 기존 `if table_idx < inline_tables.len()` 분기를 다음으로 교체:

```rust
if ctrl_idx < inline_tac_controls.len() {
    let (ci, kind) = &inline_tac_controls[ctrl_idx];
    let w = tac_widths[ctrl_idx];
    match kind {
        InlineTac::Table(tbl) => {
            // 기존 537-562 로직 그대로
            let mt = ...;
            let tbl_h = ...;
            let tbl_y = (current_y + baseline_dist + om_bottom - tbl_h).max(current_y);
            let table_bottom = self.layout_table(...);
            if table_bottom > max_table_bottom { max_table_bottom = table_bottom; }
        }
        InlineTac::Equation(eq) => {
            // 신규 — layout_composed_paragraph:1845-1900 패턴 이식
            let tokens = crate::renderer::equation::tokenizer::tokenize(&eq.script);
            let ast = crate::renderer::equation::parser::EqParser::new(tokens).parse();
            let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
            let layout_box = crate::renderer::equation::layout::EqLayout::new(font_size_px).layout(&ast);
            let color_str = crate::renderer::equation::svg_render::eq_color_to_svg(eq.color);
            let svg_content = crate::renderer::equation::svg_render::render_equation_svg(
                &layout_box, &color_str, font_size_px,
            );
            let hwp_eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
            let eq_h = if hwp_eq_h > 0.0 { hwp_eq_h } else { layout_box.height };
            // baseline 정렬: 텍스트 줄의 baseline 기준 (wrapped_below_table 분기 고려)
            let cur_baseline = if wrapped_below_table { text_line_baseline } else { baseline_dist };
            let eq_y = if hwp_eq_h > 0.0 && layout_box.height > 0.0 {
                let scale = hwp_eq_h / layout_box.height;
                (current_y + cur_baseline - layout_box.baseline * scale).max(current_y)
            } else {
                (current_y + cur_baseline - layout_box.baseline).max(current_y)
            };
            let eq_node = RenderNode::new(
                tree.next_id(),
                RenderNodeType::Equation(crate::renderer::render_tree::EquationNode {
                    svg_content, layout_box, color_str, color: eq.color,
                    font_size: font_size_px,
                    section_index: Some(section_index),
                    para_index: Some(para_index),
                    control_index: Some(*ci),
                    cell_index: None,
                    cell_para_index: None,
                }),
                BoundingBox::new(inline_x, eq_y, w, eq_h),
            );
            col_node.children.push(eq_node);
            // shape_layout 의 fallback 우회
            tree.set_inline_shape_position(section_index, para_index, *ci, inline_x, eq_y);
        }
    }
    inline_x += w;
    ctrl_idx += 1;
}
```

**표 height 영향 보존**: `max_table_bottom` 갱신은 표 분기에서만. 수식 분기는 줄 높이에 영향 주지 않음 (텍스트 줄 안에 들어가는 인라인 요소).

**후행 컨트롤 처리** (565-591): 표 전용 → TAC 컨트롤 전용으로 일반화. 수식은 줄 안에 그대로 배치.

**디버그 로깅**: `LAYOUT_INLINE_TBL[]` 옆에 `LAYOUT_INLINE_TAC[]` 추가하여 수식 위치/폭도 출력 (선택, 디버깅용).

**완료 기준**:
- `cargo build --release` 성공
- `cargo test --lib` 1111+ pass (기존 테스트 무회귀)
- `RHWP_LAYOUT_DEBUG=1 ... export-svg samples/exam_science.hwp -p 1` 출력 SVG 에서 pi=61 의 9개 수식이 각각 distinct (gx, gy) 로 ls[1]/ls[2] 줄에 인라인 배치 확인
- 빠른 시각 점검: 페이지 2 우측 단 12번 문제가 stack 없이 정상 렌더링

### Stage 3 — 회귀 검증

- `cargo test --lib` (전체 통과)
- `cargo clippy --release -- -D warnings` (warning 0)
- `scripts/svg_regression_diff.sh` (7 샘플 172 페이지 byte-identical 검증). 변경 발생 페이지는 다음 분류:
  - **의도된 정정**: exam_science p2/p3/p4 우측 단의 12번/유사 문제 (pi=61, 79, 110, 118, 120 영향)
  - **회귀**: 위 외 페이지 — 0 건 목표
- 의도된 정정 페이지는 `--debug-overlay` SVG + 시각 비교 (한컴 PDF 와 정합) 자료를 보고서에 첨부

**완료 기준**: 다른 샘플 byte-identical, exam_science 변경은 의도된 정정만 (0 회귀).

### Stage 4 — 최종 보고서 + close

- `mydocs/report/task_m100_526_report.md` 작성 (원인·수정 요약, before/after 좌표 비교, 회귀 검증 결과)
- `mydocs/orders/20260502.md` 의 #526 행 갱신 (상태 → 완료)
- merge: `local/task526` → `local/devel` (`--no-ff`)
- 작업지시자 승인 후: `local/devel` → `devel` (`--no-ff`) → `git push origin devel`
- `gh issue close 526`

## 4. 위험·대응

| 위험 | 영향 | 대응 |
|------|------|------|
| 수식 baseline 계산 차이 (`layout_composed_paragraph` 와 미세 다름) | 수식 y 가 1-2px 어긋남 | layout_composed_paragraph:1862-1867 식과 동일 적용. 시각 확인. |
| 수식이 줄바꿈된 텍스트 줄 (`wrapped_below_table=true`) 에 위치 시 baseline 차이 | 표 줄 baseline vs 텍스트 줄 baseline 혼동 | `cur_baseline = if wrapped_below_table { text_line_baseline } else { baseline_dist }` 분기로 명시. |
| 수식 폭이 텍스트 폭 계산에 포함되어 단 우측 마진 침범 | reflow 발생 또는 정렬 어긋남 | `total_width` 에 수식 폭 포함 → start_x 계산 시 자연스럽게 흡수. inline_x 누적도 표와 동일 처리. |
| `inline_tac_controls` 와 segments 갯수 불일치 | 표/수식 인덱스 mismatch | composer.rs::find_control_text_positions 와 동일한 필터 (Table tac=true + Equation 무조건) 사용. char_offsets 갭 분석 결과와 1:1 대응 가정. 미스매치 시 디버그 assert 추가. |
| 다른 샘플의 인라인 표 단락 회귀 | 의도치 않은 좌표 변경 | scripts/svg_regression_diff.sh 7 샘플 byte-identical 검증. 표만 있는 단락은 inline_tac_controls 에 표만 들어가므로 기존 경로와 동일 동작 보장. |

## 5. 검증 게이트

| 게이트 | 도구 | 기준 |
|--------|------|------|
| 빌드 | `cargo build --release` | 성공 |
| 단위 테스트 | `cargo test --lib` | 1111+ pass |
| Clippy | `cargo clippy --release -- -D warnings` | warning 0 |
| 회귀 검증 | `scripts/svg_regression_diff.sh` | 7 샘플 회귀 0 (exam_science 의도 정정만 OK) |
| 시각 정합 | export-svg + 한컴 PDF 비교 | exam_science p2 우측 단 12번 문제 9개 수식이 ls[1]/ls[2] 에 distinct 인라인 배치 |

## 6. 참고

- 본 구현은 `layout_composed_paragraph:1843-1922` 의 inline TAC 처리 패턴을 그대로 이식. 새 알고리즘 도입 없음.
- Picture·Form 은 현 샘플에 케이스 없으므로 본 task 범위에서 제외. enum 구조는 향후 확장 가능.
- 영향 문단 5건 모두 표 + 수식 조합 (Stage 1 §5).

---

승인 요청: 본 구현 계획서대로 Stage 2 정정 구현 진행 가능 여부.
