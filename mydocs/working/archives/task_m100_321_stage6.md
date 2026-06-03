# Task #321 v5 Stage 6 — pi=0 block-table drift 보정 (수정)

## Stage 6a — H1 시안 적용

**수정 위치**: `src/renderer/typeset.rs::typeset_block_table`

조건이 모두 참인 block-table 호스트 문단(=Paper-anchored TopAndBottom 절대 배치 표) 처리:
- `!treat_as_char`
- `wrap = TopAndBottom`
- `vert_rel_to = Paper`
- `current_column == 0`

이 경우 cur_h 를 `first_seg.vertical_pos` (HWP first_vpos) 로 jump, 표 effective_height 는
cur_h advance 에 포함하지 않는다 (`table_total_height = 0.0` 으로 `place_table_with_text` 호출).

표 자체는 layout 단의 Paper-anchored 절대 좌표 경로(`table_layout.rs:992` `ref_y=0.0`) 로
원래 위치에 그대로 그려지므로 **시각 무변경**.

핵심 diff:

```rust
let is_paper_topbottom_block =
    !table.common.treat_as_char
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Paper);
if is_paper_topbottom_block && st.current_column == 0 {
    if let Some(first_seg) = para.line_segs.first() {
        let target_y = hwpunit_to_px(first_seg.vertical_pos as i32, self.dpi);
        let pre_lines_h = fmt.line_advances_sum(0..fmt.line_heights.len());
        if target_y > st.current_height && target_y + pre_lines_h <= available {
            st.current_height = target_y;
            self.place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, 0.0);
            return;
        }
    }
}
```

## Stage 6b — 회귀 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **992 passed**, 0 failed, 1 ignored |
| `cargo test --test svg_snapshot` (golden 6개) | **6 passed**, 0 failed |
| `cargo clippy --release` | **clean** |
| 21_언어 page count | 16 (유지) |
| exam_math page count | 20 (유지) |
| exam_kor page count | 24 (유지) |
| exam_eng page count | 10 (유지) |
| exam_math_8 / exam_science / exam_social | 1 / 5 / 5 (유지) |

## 21_언어 page 1 측정 (전후)

| 항목 | 수정 전 (devel/v3-정밀화) | 수정 후 (v5) | 변화 |
|------|--------------------------|--------------|------|
| col 0 used | 1233.5 px | 1229.9 px | -3.6 |
| col 0 hwp_used | 1147.7 px | 1220.3 px | (LINE_SEG 측정 차이) |
| col 0 drift | **+85.8 px** | **+9.5 px** | **-76.3** |
| col 0 마지막 item | PartialParagraph pi=9 lines=0..11 | **FullParagraph pi=9 (전체)** | ✓ |
| col 1 첫 item | PartialParagraph pi=9 lines=11..14 ("도출될 경우...") | **FullParagraph pi=10 ("적합성 검증이란...")** | ✓ PDF 일치 |

drift 잔여 +9.5 px = 마지막 paragraph 의 `trailing_ls`. placement 자체는 정확하며,
dump-pages 의 측정 방식 차이일 뿐 페이지 분할에 영향 없음.

## 시각 검증

- 상단 4×5 폼 표 (성명/수험번호/홀수형): 위치 무변경 (Paper-anchored 절대 좌표)
- col 0: 불릿 4개 → "[1~3]" → "비즈니스 프로세스..." (border 다중 줄 정상)
- col 1 첫 문단: **"적합성 검증이란 기존의 프로세스 모델과 이벤트 로그 분석에서..."** ✓ PDF 일치

이미지: `/tmp/v5_half.png` (작업용)

## 영향 범위

- 진단 로그(Stage 5a) + 표 advance 보정(Stage 6a) 외 동작 변경 없음
- env-gated 진단 로그는 default 출력 무영향
- 다른 샘플(exam_*, hwpx, form-002 등) 페이지 수/golden SVG 무변경
- col 1+ 의 `pending_body_wide_top_reserve` 처리와 충돌 없음 (col 0 한정 가드)

## 잔여 사항

- col 0 drift +9.5 px (= trailing_ls) 은 측정 노이즈로 분류, 추가 수정 불필요
- col 1 drift +218.9 px (dump-pages 보고) 는 col 1 끝부분(4번 보기) 가 col 1 에 들어간 결과의
  부산물 — visual 검증에서 정상 (PDF 와 일치)
- Paper-anchored TopAndBottom 표가 다른 페이지의 col 1 (= `current_column != 0`) 에 등장하는
  케이스가 발견되면 가드 확장 필요 (현 단계에선 회귀 0)
