# Task #568 Stage 1 진단 보고서 — 본질 결함 식별

- **이슈**: [#568](https://github.com/edwardkim/rhwp/issues/568)
- **브랜치**: `local/task568`
- **단계**: Stage 1 — 정밀 진단 (코드 무수정)
- **작성일**: 2026-05-04

## 1. 결론 (요약)

본질 결함은 **`layout_composed_paragraph` 의 `available_width` 가 `col_area.width` 만 사용하고, 줄별 `comp_line.segment_width` 를 무시**하는 것이다.

인라인 TAC 표(treat_as_char + wrap=TopAndBottom)가 있는 줄에서 HWP 는 `LINE_SEG.segment_width` 를 표 폭 + 작은 여유로 좁게 인코딩한다. 그런데 layout 은 컬럼 전체 폭(407.5 px) 으로 `available_width` 를 잡고, **Justify slack 을 과대 산출** 해 `extra_word_spacing` (per-space) 으로 분배한다. 결과적으로 줄 시작의 공백 글자가 80 px 씩 늘어나 그 다음에 오는 인라인 표를 약 +167 px 우측으로 밀어버린다.

같은 패턴이 셀 내부 paragraph 에도 동일하게 발생 (cell_ctx 가 있어도 코드 경로는 같음).

## 2. 핵심 단락 — pi=61 (page 2 12번 응답)

### 2.1 IR 측 데이터 (rhwp dump + 임시 example diag_task568)

```
--- 문단 0.61 --- cc=137, text_len=56, controls=10
  텍스트: "  는? (단, 는 임의의 원소 기호이고, , , , 의 원자량은 각각 , , , 이다.) [3점] "
  [PS] ps_id=102 align=Justify spacing: before=0 after=1000 line=140/Percent
       margins: left=2260 right=0 indent=0
  ls[0]: vpos=74118 lh=2864 bl=1432 ls=460 cs=1130 sw=18939 text_start=0
  ls[1]: vpos=77442 lh=1150 bl=575 ls=460 cs=1130 sw=18939 text_start=13
  ls[2]: vpos=79052 lh=1150 bl=575 ls=460 cs=1130 sw=30562 text_start=60
  ctrl[0]: Table 행=2 tac=true wrap=TopAndBottom w=14745 h=2864    ← 인라인 분수
  ctrl[1..9]: Equation tac=true (rmX, rmA..D, m±2, m±4)
  composed.lines = 3
    line[0]: char_start=0 segment_width=18939 line_height=2864 runs=1
      run[0]: 5 chars text="  는? "
    line[1]: char_start=5 segment_width=18939 line_height=1150 runs=1
      run[0]: 23 chars text="(단, 는 임의의 원소 기호이고, , , "
    line[2]: char_start=28 segment_width=30562 line_height=1150 runs=4
  tac_controls (pos, width_HU, ci):
    (2, 14745 HU = 196.60 px, ci=0)   ← 인라인 표 (분수) — line[0] 내부, 2 spaces 뒤
    (9, 675 HU = 9 px, ci=1)
    (24, 11 px, ci=2) ... (46, 34.11 px, ci=9)
```

**핵심 관찰**:
- `ls[0]` 와 `ls[1]` 는 sw=18939 HU = **252.5 px** (좁은 segment), `ls[2]` 만 sw=30562 HU = **407.5 px** (full column).
- 컬럼 전체 폭 = 30562 HU = 407.5 px (다단 한 단).
- `tac_controls` pos=2 (line[0] 내부) — 2 spaces 뒤에 인라인 분수.

### 2.2 SVG 측 실측 (output/svg/exam_science_dbg/exam_science_002.svg)

```
column 1 좌측 edge :  x=549.87 (또는 536.8 — column 박스 내 안쪽 텍스트 영역)
pi=61 단락 박스   :  x=549.87, y=1137.5, w=407.5
pi=61 ci=0 2x1   :  x=739.87, y=1137.5, w=196.6   ← 인라인 분수 debug-overlay
첫 cell 글리프    :  translate(747.25, 1140.44)
```

**좌표 차이**:
- 컬럼 시작: x=549.87
- 단락 left margin: 2260 HU / 7200 inch / 25.4 mm × 96dpi 변환 후 ParaStyle resolver 가 `/2` 적용 → **15.07 px**
- 기대 표 시작 x : 549.87 + 15.07 ≈ **564.94 px** (Justify 의 default branch)
- 실제 표 시작 x : **739.87 px**
- **편위: +174.93 px**

## 3. 원인 추적 — 코드 경로

### 3.1 라우팅 (Task #565 fix 후)

`src/renderer/layout.rs` L2056-2120:

```rust
let has_inline_tables = … treat_as_char + is_tac_table_inline …;
let has_other_inline_ctrls = matches Equation / Picture(tac) / Shape(tac);

if has_inline_tables && !has_other_inline_ctrls {
    layout_inline_table_paragraph(…)        // 인라인 표 전용
} else {
    layout_paragraph(…)                     // 일반 경로
}
```

pi=61 는 `has_inline_tables=true && has_other_inline_ctrls=true` → **else 분기 (일반 경로)**.

### 3.2 일반 경로 — `layout_composed_paragraph` (paragraph_layout.rs)

#### 3.2.1 `effective_col_x / effective_col_w` 계산 (L857-866)

```rust
let (effective_col_x, effective_col_w) = if has_picture_shape_square_wrap
    && comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu - 200
{
    let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
    let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)        ← pi=61 line[0] 진입 — Picture/Shape 없음
};
```

`has_picture_shape_square_wrap` 은 비-TAC Picture 또는 Shape 의 wrap=Square 만 검출. pi=61 의 인라인 TAC 표 (treat_as_char=true) 는 이 조건에 해당하지 않으므로 **else 분기로 빠져 `col_area.width` 를 사용**한다 → `effective_col_w = 407.5 px` (실제 줄 폭 252.5 px 가 아닌 컬럼 전체).

#### 3.2.2 `available_width` 산출 (L907)

```rust
let available_width = effective_col_w - effective_margin_left - margin_right
                       - inline_offset - num_offset;
                    = 407.5 - 15.07 - 0 - 0 - 0
                    = 392.43 px       ← 실제로는 252.5 - 15 = 237 px 가 맞음
```

#### 3.2.3 Justify slack 분배 (L1087-1121)

line[0] 의 run = "  는? " (5 chars, 2 leading spaces + "는?" + trailing space).

est_x 계산 (L984-1004):
```
est_x_start = effective_margin_left + 0 = 15.07
seg "  " (2 spaces, extra_word_sp 미적용 시점) → est_x += ~10 px
TAC 표 → est_x += 196.6 px
remaining "는? " → est_x += ~30 px
est_x ≈ 251.67 px
total_text_width = est_x - est_x_start ≈ 236.6 px
```

`needs_justify = true` (line[0] 은 마지막 줄 아니고 forced break 없음) →
```
all_chars = "  는? "  →  trailing_spaces=1, visible_count=4, interior_spaces=2 (선두 2개)
trailing_width = ~5 px (한 공백 폭)
effective_used = 236.6 - 5 = 231.6
slack = available_width - effective_used = 392.43 - 231.6 = 160.83 px
extra_word_sp = slack / interior_spaces = 160.83 / 2 = 80.41 px / space    ← 본질 결함
```

#### 3.2.4 렌더링 — 표가 우측으로 밀려나는 순간 (L1734-1774, L1888-1939)

```rust
let mut x = x_start;       // x_start = effective_col_x + effective_margin_left = 564.94

for &(tac_rel, tac_w, tac_ci) in &run_tacs {
    if seg_start < tac_rel {                    // seg_start=0 < tac_rel=2
        let seg_text = "  ";                    // run_chars[0..2] = 2 spaces
        let seg_w = estimate_text_width(seg_text, &seg_style);
        // estimate_text_width 가 공백마다 extra_word_spacing 추가 (text_measurement.rs:220):
        //    if c == ' ' { w += style.extra_word_spacing; }
        // → seg_w ≈ 2 * (5 + 80.41) = 170.82 px
        x += seg_w;                             // x = 564.94 + 170.82 = 735.76
    }
    // 인라인 TAC 표 렌더 (L1888-1903)
    if let Some(Control::Table(t)) = ... {
        if t.common.treat_as_char {
            self.layout_table(... Some(x) ...);   // x ≈ 735.76 ≈ 실측 739.87 ✓
            tree.set_inline_shape_position(... x, table_y);
        }
    }
    x += tac_w;
    seg_start = tac_rel;
}
```

**산식 일치**: x_start(564.94) + 2 spaces × (5 + 80.41 extra) = 735.76 px ≈ 실측 739.87 (±4 px 폰트 metric 오차).

## 4. 영향 범위

### 4.1 narrow segment_width + 인라인 TAC 표 보유 paragraph 스캔 (exam_science.hwp)

임시 example `diag_task568.rs` 로 전체 paragraph 스캔:

| pi | text 미리보기 | narrow_lines (sw HU) | inline TAC 표 |
|----|--------------|---------------------|--------------|
| 21 | "5.-그림은 밀폐된 진공 용기에…" | sw=19592 (lines 0-5) | ✗ (Picture wrap=Square — 기존 fix 적용됨) |
| 37 | "8.-그림은 수소와 원소 로…" | sw=17546 | ✗ (Picture wrap=Square) |
| 60 | "12.-그림은 원자 의 중성자수와…" | sw=20069 | ✗ (Picture wrap=Square) |
| **61** | "  는? (단, 는 임의의 원소…" | **sw=18939 (lines 0-1)** | **✓ (분수 2×1)** ← 본 결함 트리거 |

→ 본 문서 본문(컬럼 직접 paragraph)에는 pi=61 만 narrow seg + inline TAC 표 조합.

### 4.2 셀 내부 paragraph (사용자 보고 페이지 3/4 항목)

사용자 보고:
- p3 13번 "(다)에서 반응한 의 양()" — pi=68 외곽 3×3 보기 표 의 셀[5] 내부 paragraph "ㄴ. 이다." 에 인라인 분수(2×1 표)
- p3 15번 "제 이온화 에너지" — 마찬가지 보기 셀 내부 분수 paragraph
- p3 16번 "ㄴ. ㄷ." — 마찬가지
- p4 19번 "제 이온화 에너지" — 마찬가지

이 cell 내부 paragraph 들도 `layout_composed_paragraph` 를 통과하며 (cell_ctx Some), 동일한 `effective_col_w = col_area.width` (cell inner width) 분기를 탄다. **셀 폭이 인라인 분수 폭보다 충분히 크면 동일 결함이 발현** (cell width 30562 vs 분수 ~15311 HU 차이만큼 slack 과대 산출).

→ **본질 결함은 단일** — 인라인 TAC 표가 있는 줄의 `comp_line.segment_width` 를 `effective_col_w` 로 사용하지 않는 것. 본문 paragraph 든 셀 내부 paragraph 든 같은 코드 경로.

## 5. 정상 케이스 대조

### 5.1 has_picture_shape_square_wrap 분기 (이미 동작)

pi=21/37/60 (그림 wrap=Square) — 이 분기는 narrow `segment_width` 를 인식하여 `effective_col_w = sw_px` 로 좁힘. Justify slack 도 좁은 폭 기준 산출 → 정상 배치.

### 5.2 인라인 표만 (수식 없음) — `layout_inline_table_paragraph`

`has_inline_tables && !has_other_inline_ctrls` 분기는 별도 함수 사용. 이 함수는 인라인 표 폭을 직접 처리하므로 본 결함 미발현. (Task #565 가 가린 결함이 별개 경로에서 노출된 것.)

### 5.3 Equation tac=true 만 (표 없음) — full segment_width

수식만 있는 paragraph 는 HWP 가 segment_width 를 좁히지 않음 (수식은 작아서). full sw 사용 → 본 결함 미발현.

## 6. 본질 정정 방향 (Stage 2 제안 후보)

### 안 (a) — `effective_col_x/effective_col_w` 분기 확장

`has_picture_shape_square_wrap` 조건에 **인라인 TAC 표 보유 줄** 도 포함:

```rust
let has_narrow_seg_for_tac = comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu - 200
    && /* 이 줄에 inline TAC 표 또는 큰 inline TAC 가 있고 sw 가 그 폭 부근 */;

let (effective_col_x, effective_col_w) = if (has_picture_shape_square_wrap || has_narrow_seg_for_tac)
    && comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu - 200
{
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)
};
```

**장점**: 기존 분기 패턴 재사용, 작은 변경 면적, Picture/Shape Square wrap 과 동등성 유지.
**단점**: "인라인 TAC 표 보유 줄" 판정 로직 필요 (tac_offsets_px + line char range 교차).

### 안 (b) — Justify slack 산출 시 segment_width 클램프

`available_width` 자체는 col_area 폭 그대로 두되, slack 계산 시 segment_width 만큼 클램프.

**단점**: 안 (a) 와 효과는 같으나 분기 흩어짐 (slack/x_start/available_width 서로 다른 위치). 본질이 effective_col_w 인데 부분 정정.

### 안 (c) — 인라인 TAC 표 보유 paragraph 라우팅 재변경

Task #565 fix 부분 되돌리고, 인라인 표는 `layout_inline_table_paragraph` 로, 수식만 별도 처리.

**단점**: 두 함수 양쪽 다 인라인 수식 처리 추가 필요 → 변경 면적 큼, 회귀 위험 증가.

→ **안 (a) 권고**. Stage 2 에서 정확한 조건과 회귀 검증 fixture 후보 확정.

## 7. 회귀 검증 후보 (Stage 2 에서 확정)

본 정정이 영향을 줄 수 있는 분류:

1. **pi=21/37/60 (Picture Square wrap)** — 기존 분기와 동일 효과 → 동일 출력 보존되어야 함.
2. **인라인 TAC 표만 (수식 없음)** — 다른 함수 경로 → 영향 없음.
3. **인라인 TAC 표 + 수식 (full sw)** — pi=79, 110, 118, 120 — sw 가 full column 이면 새 분기 미진입 (조건 `sw < col_area_w_hu - 200` 불충족) → 동일.
4. **셀 내부 paragraph** — 셀 폭과 sw 비교, narrow 인 경우만 진입 → 의도된 정정.

광범위 fixture sweep (15+ 샘플) 에서 byte-identical 확인 + exam_science 만 의도된 diff.

## 8. 산출물

- 본 보고서: `mydocs/working/task_m100_568_stage1.md`
- 임시 진단 example (커밋 안 함 — 분석 후 삭제됨): `examples/diag_task568.rs`

## 9. 승인 요청

본 진단 결과를 바탕으로 Stage 2 (구현 계획서 작성, 안 (a) 기반 상세화) 진입을 승인 요청합니다.
