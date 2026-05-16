# Task #928 구현 계획서

## 1. ROOT CAUSE 요약

`src/renderer/layout/table_layout.rs::layout_table_cells` 의 Shape 분기 (1812-1963) 와 트레일링 텍스트 블록 (2158-2231) 에 **`will_render_inline` 가드 누락**. Picture 분기 (1693-1769) 는 가드가 있어 회귀 없음.

결과: `layout_composed_paragraph` 의 inline TAC split (run_tacs) 가 한 baseline 에 발행한 paragraph 텍스트를, Shape 분기와 트레일링 블록이 다른 baseline 에 재발행 → 시각적 2회 중복.

## 2. Fix 적용 영역

**HWP3 전용 분기 추가 금지** (CLAUDE.md). 본 회귀는 공통 렌더러 (`src/renderer/layout/table_layout.rs`) 의 가드 누락이며, HWP3 한정 회귀가 아니라 **셀 내부 inline Shape (tac=true) 일반 케이스의 가드 누락**. 공통 모듈에서 수정한다.

> Stage 1 측정상 회귀는 HWP3 샘플 (`exam_kor.hwp`) 에서만 확인되었지만, table_layout 은 HWP5/HWPX/HWP3 공용 경로다. HWP5/HWPX 에서 동일 케이스 (셀 내 사각형 tac=true) 가 발생하면 같은 회귀가 있을 수 있으나 미확인. 본 수정은 모든 포맷에 동일 적용된다.

## 3. 구현 단계 (Stage 3~5, 총 3단계)

### Stage 3: Fix 구현 + 단위 회귀 차단

**3.1 Shape 분기 가드 추가** — `table_layout.rs:1812-1963`

```rust
Control::Shape(shape) => {
    if shape.common().treat_as_char {
        let will_render_inline = composed.tac_controls.iter().any(|&(abs_pos, _, ci)| {
            ci == ctrl_idx && composed.lines.iter().any(|line| {
                let line_chars: usize = line.runs.iter().map(|r| r.text.chars().count()).sum();
                abs_pos >= line.char_start && abs_pos < line.char_start + line_chars
            })
        });
        
        if will_render_inline {
            // layout_composed_paragraph 가 텍스트와 inline_shape_position 등록을 완료함.
            // 도형 자체 렌더링만 수행하고 텍스트/prev_tac_text_pos 갱신은 스킵.
            let shape_w = hwpunit_to_px(shape.common().width as i32, self.dpi);
            // get_inline_shape_position 로 좌표 조회
            let inline_pos = tree.get_inline_shape_position(section_index, cp_idx, ctrl_idx, cell_context.as_ref());
            if let Some((shape_x, shape_y)) = inline_pos {
                let shape_area = LayoutRect {
                    x: shape_x,
                    y: shape_y,
                    width: shape_w,
                    height: hwpunit_to_px(shape.common().height as i32, self.dpi),
                };
                self.layout_cell_shape(tree, &mut cell_node, shape, &shape_area, shape_y, Alignment::Left, styles, bin_data_content);
            } else {
                // fallback: 기존 경로 유지 (안전망)
                /* 기존 1814-1963 로직 */
            }
            // 텍스트/prev_tac_text_pos 갱신 없음
        } else {
            // 기존 경로: text_before 추출 + 발행 + layout_cell_shape + prev_tac_text_pos 갱신
            /* 기존 1814-1963 로직 그대로 */
        }
    }
}
```

**3.2 트레일링 텍스트 블록 가드 추가** — `table_layout.rs:2158-2231`

`prev_tac_text_pos > 0` 조건만 으로 진입하면, will_render_inline 으로 prev_tac_text_pos 가 갱신되지 않은 경우 진입 안 함. 단 다음 보정이 필요:
- 만약 한 paragraph 에 will_render_inline=true Shape 와 will_render_inline=false Shape 가 혼재하면, 후자 때문에 prev_tac_text_pos 가 갱신되어 트레일링 블록이 발행될 수 있음. 이 경우 잘못된 trailing 발행이 회귀가 될 수 있는지 검토.
- 본 케이스 (단일 Shape) 에서는 prev_tac_text_pos = 0 유지 → 트레일링 블록 미진입.

**3.3 단위 회귀 차단**

- `cargo build --release` + `cargo test` 통과
- `samples/exam_kor.hwp` 5쪽 SVG 재출력 → 다이어그램 행 `(가) ⇨ [A 단계] ⇨ (나)` 3 요소만 출력 확인 (y baseline 1개)
- `cargo test` snapshot 회귀 없음 (변경된 snapshot 은 별도 승인 후 갱신)

→ 단계 보고서 `task_m100_928_stage3.md` 작성, 승인 요청

### Stage 4: 시각 회귀 검사 확장

**4.1 동일 케이스 회귀 가능 파일 스캔**

`samples/` 의 HWP3 / HWP5 / HWPX 샘플에서 "셀 내 inline Shape tac=true" 케이스 자동 탐지 → 변경 전후 SVG diff. 후보 대상:

- `samples/exam_kor.hwp` (회귀 대상)
- `samples/exam_science.hwp` (Task #500 등 비슷한 표 셀 inline shape 처리 이력)
- `samples/basic/` 의 HWP3 샘플들
- `samples/hwpx/` 의 HWPX 샘플들

**4.2 한컴 2022 PDF 정합 비교** (가능한 경우)

- `pdf/exam_kor-2022.pdf` 가 존재하면 다이어그램 행 시각 일치 확인
- 부재 시: 작업지시자 시각 판정 요청

**4.3 svg_snapshot 회귀 확인**

```bash
cargo test --test svg_snapshot 2>&1 | tail
```

snapshot 변경 발생 시 변경 paragraph 분석 + 의도된 변경 (회귀 해소) vs 의도치 않은 변경 (다른 회귀) 분리.

→ 단계 보고서 `task_m100_928_stage4.md`, 승인 요청

### Stage 5: 최종 정리 + 보고서

**5.1 코드 cleanup**

- 주석 정리 (Task #928 참조 추가, ROOT CAUSE 1줄 메모)
- 불필요한 디버그 출력 제거

**5.2 최종 보고서** `report/task_m100_928_report.md`

- 회귀 원인, fix, 검증 결과, 영향 범위
- 향후 회귀 차단 위한 가드 패턴 정착 노트

**5.3 오늘할일 갱신**

`mydocs/orders/20260516.md` 에 Task #928 완료 갱신

→ 최종 승인 요청, merge 준비

## 4. 위험 요소 및 완화

| 위험 | 가능성 | 영향 | 완화 |
|------|--------|------|------|
| `get_inline_shape_position` 미반환 시 도형 사라짐 | 중 | 큼 | fallback 으로 기존 경로 유지 (안전망) |
| paragraph_layout 의 run_tacs split 좌표가 부정확하면 새 baseline 에 발행되어 새 회귀 | 낮음 | 중 | Stage 4 시각 검사로 차단 |
| 트레일링 블록 가드 미적용으로 부분 회귀 | 낮음 | 중 | 단일 paragraph 에 will_render_inline 혼재 케이스 회귀 검사 (Stage 4) |
| 다른 샘플 (HWPX 셀 내 inline 사각형) 에서 의도치 않은 시각 변경 | 중 | 중 | svg_snapshot 회귀 0건 확인 (Stage 4) |

## 5. 비범위

- HWP3 파서 (`src/parser/hwp3/`) 수정 없음 — 회귀는 렌더러 가드 누락
- 본문 paragraph (셀 외부) 의 동일 케이스는 별도 코드 경로 (paragraph_layout 직접 호출) 이므로 영향 없음
- 사각형 외 다른 Shape 종류 (타원, 화살표, polyline 등) 의 인라인 처리는 동일 분기 사용 → 자연스럽게 fix 혜택

## 6. 승인 기준

- `samples/exam_kor.hwp` 5쪽 다이어그램 행 `(가) ⇨ [A 단계] ⇨ (나)` 3 요소 정확 출력 (단일 baseline)
- `cargo test` 전체 통과
- svg_snapshot 회귀 0건 (변경 시 작업지시자 승인 후 갱신)
- 다른 HWP3/HWP5/HWPX 샘플 시각 회귀 0건
