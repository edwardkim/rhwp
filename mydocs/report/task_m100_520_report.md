# Task #520 최종 결과보고서 — exam_science.hwp 인라인 TAC 도형 위치 정정

GitHub Issue: [#520](https://github.com/edwardkim/rhwp/issues/520)
브랜치: `local/task520`

## 1. 결함 요약

`samples/exam_science.hwp` 페이지 3, 문제 7번의 1×1 표(pi=33) 셀 내부 두 번째 문단(p[1])에 들어 있는 inline 도형(㉠ 사각형, `tac=true`, `wrap=TopAndBottom`)이 자기가 속한 라인(ls[1], "...모두 ㉠ 이다.")이 아니라 **다음 문단 [탐구 과정 및 결과] 라인** 위에 그려져 텍스트와 시각적으로 겹쳤다.

사용자가 인지한 "[탐구 과정 및 결과] 위치가 위로 올라가 있다" 는 실제로는 ㉠ 사각형이 한 줄 내려와 그 라인을 침범한 결과.

## 2. 원인

`src/renderer/layout/table_layout.rs` 의 인라인 TAC 도형/그림 렌더 패스(line 1547-1549, 1631-1633):

```rust
// Before
tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
```

HWP `LineSeg.vertical_pos` 는 **셀 origin 기준** 절대값이며, `para_y_before_compose` 에는 이미 `ls[0].vpos` 가 누적되어 있다. 둘을 그대로 더하면 `ls[0].vpos` 가 이중 합산된다.

본 케이스 수치:
- 셀 origin y = 185.28 px
- p[1] ls[0].vpos = 1610 HU = 21.47 px → `para_y_before_compose` = 206.75
- p[1] ls[1].vpos = 3220 HU = 42.93 px
- 잘못된 산출: 206.75 + 42.93 = **249.68** (1줄 아래)
- 올바른 산출: 185.28 + 42.93 = **228.21** (paragraph_layout 가 등록한 inline_pos.y 와 일치)

paragraph_layout.rs 가 `tree.set_inline_shape_position` 으로 정확한 좌표(228.21) 를 등록함에도 본 패스가 이 값을 조회하지 않고 자체 공식으로 재계산한 것이 본질.

## 3. 수정

`src/renderer/layout/table_layout.rs` 두 위치 (Picture / Shape 인라인 분기):

```rust
let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
tac_img_y = para_y_before_compose
    + hwpunit_to_px(seg.vertical_pos - first_vpos, self.dpi);
```

`ls[0].vpos` 를 빼서 `target_line` 의 paragraph 내 상대 오프셋만 더한다.

## 4. 영향 범위

- 셀 내부의 두 번째+ paragraph 에 `line_segs.len() ≥ 2` 인 인라인 TAC shape/picture 가 있는 경우에만 결과 좌표가 변경됨.
- 셀 첫 paragraph(`ls[0].vpos = 0`) 또는 단일 줄 paragraph 에서는 두 공식이 동일 → 회귀 없음.

## 5. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ warnings 0 |
| `cargo test --release` | ✅ 1111 passed, 0 failed, 1 ignored |
| `cargo clippy --release -- -D warnings` | ✅ |
| `samples/exam_science.hwp` p3 시각 검증 | ✅ PDF 일치 (㉠ 박스 "이다." 줄, [탐구 과정 및 결과] 깨끗) |
| `samples/tac-img-02.hwpx` (73p) | ✅ 정상 출력 |
| `samples/table-vpos-01.hwpx` (5p) | ✅ 정상 출력 |
| `exam_science` p4/p5 인접 페이지 | ✅ 박스 안 다단 콘텐츠 정상 |

## 6. 산출물

- 수정 코드: `src/renderer/layout/table_layout.rs`
- 문서:
  - `mydocs/plans/task_m100_520.md` — 수행계획서
  - `mydocs/plans/task_m100_520_impl.md` — 구현계획서
  - `mydocs/working/task_m100_520_stage1.md` — 진단 + 원인 확정
  - `mydocs/working/task_m100_520_stage3.md` — 검증 + 회귀 확인
  - 본 보고서

## 7. 향후 과제

본 수정은 좌표 산출 공식 정정에 한정된다. 더 본질적으로는 `table_layout.rs` 의 인라인 TAC 패스가 `paragraph_layout.rs` 가 등록하는 `inline_shape_position` 을 그대로 재사용하도록 통합하는 것이 바람직하다 (중복 좌표 산출 제거). 별도 리팩터링 task 로 분리 권장.
