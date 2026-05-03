# Task #533 Stage 1 — Root Cause 위치 확정

**작성일**: 2026-05-02
**이슈**: [#533](https://github.com/edwardkim/edwardkim/rhwp/issues/533)
**브랜치**: `local/task533`
**범위**: 조사만 (코드 변경 0)

## 1. 결론

> **Root cause**: `src/renderer/layout.rs::layout_table_item` 의 non-TAC Square wrap 표 처리에서 y_offset 을 **표 높이 기준** (table_bottom + line_spacing) 으로 advance. 그러나 Square wrap 표는 floating 이므로 호스트 문단 텍스트가 표보다 아래로 흐를 수 있고, 이때 y_offset 은 **호스트 문단 텍스트 높이** (last_line.vpos + lh + ls) 로 advance 되어야 함. 두 값의 차이만큼 (≈ 12.8 px ≈ baseline 978 HU) 다음 문단이 위로 시프트됨.

## 2. 측정 데이터 (RHWP_VPOS_DEBUG)

```bash
RHWP_VPOS_DEBUG=1 target/release/rhwp export-svg samples/exam_kor.hwp -p 13 -o /tmp/p14_dbg
```

```
VPOS_CORR: path=lazy pi=51 prev_pi=50 prev_vpos=3676 prev_lh=1150 prev_ls=688
           vpos_end=5514 base=958 col_y=211.65 y_in=272.40 end_y=272.40 applied=true
VPOS_CORR: path=lazy pi=52 prev_pi=51 prev_vpos=7352 prev_lh=1150 prev_ls=688
           vpos_end=9190 base=958 col_y=211.65 y_in=312.24 end_y=321.41 applied=true
```

### 해석

| 항목 | 값 | 의미 |
|------|-----|------|
| pi=51 y_in | 272.40 px | pi=50 (wrap host + Square table) 처리 후 y_offset |
| pi=51 vpos_end | 5514 HU | 본 문단 첫 LINE_SEG vpos (column 누적 좌표) |
| pi=51 base | 958 HU | lazy_base = prev_vpos_end (5514) - y_delta_hu (4556) |
| pi=51 end_y | 272.40 px | col_y + (vpos_end - base)/dpi = 211.65 + (5514-958)/75 |
| pi=51 SVG y | 285.44 px | end_y + baseline 13.04 |
| **기대 SVG y** | **298.26 px** | end_y_expected (285.22) + baseline 13.04 |

→ `base=958` 이 0 이어야 정확. base 가 958 인 원인 = y_offset 이 272.40 으로 이미 잘못 advance.

## 3. y_offset Advance 경로 추적

### 3-1. pi=50 처리 순서 (PageItem 시퀀스)

dump-pages 단 1:
```
PartialParagraph  pi=50  lines=0..3  vpos=0..3676
Table          pi=50 ci=0  3x2  23.0x51.4px  wrap=Square tac=false  vpos=0..3676
```

Column 1 진입 시 y_offset = col_area.y = 211.65 px.

### 3-2. PartialParagraph(pi=50) — `src/renderer/layout.rs:2092-2106`

```rust
PageItem::PartialParagraph { para_index, start_line, end_line } => {
    if let Some(para) = paragraphs.get(*para_index) {
        let is_wrap_host = para.controls.iter().any(|c| {
            if let Control::Table(t) = c {
                !t.common.treat_as_char
                    && matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
            } else { false }
        });
        if is_wrap_host {
            return (y_offset, false);   // ← y_offset 그대로 반환
        }
        ...
```

→ y_offset 변경 없음. y_offset = 211.65.

### 3-3. Table(pi=50, ci=0) — `src/renderer/layout.rs::layout_table_item`

#### 3-3-1. 표 본체 layout (line 2408-2418)

```rust
} else {
    let table_y_start = if let Some((_, iy)) = inline_pos { iy } else { y_offset };
    y_offset = self.layout_table(
        tree, col_node, t,
        page_content.section_index, styles, col_area,
        table_y_start, bin_data_content, mt, 0,
        Some((para_index, control_index)),
        alignment, None, effective_margin, margin_right,
        tbl_inline_x, None, Some(para_y_for_table),
    );
}
```

`layout_table` 가 표 bottom y 반환 → y_offset = 211.65 + 51.4 + outer_margin_bottom ≈ 263 px (Square wrap 표 51.4 px 높이).

#### 3-3-2. 줄간격 추가 (line 2509-2512)

```rust
if !tac_seg_applied && !is_outside_body {
    ...
    if let Some(seg) = para.line_segs.last() {
        let gap = if seg.line_spacing > 0 { seg.line_spacing } else { seg.line_height };
        y_offset += hwpunit_to_px(gap, self.dpi);
    }
}
```

pi=50 ls[2].line_spacing = 688 HU = 9.17 px. y_offset += 9.17 → **272.40 px**. ✓ VPOS_CORR y_in 일치.

### 3-4. 기대 동작

pi=50 호스트 문단 (Square wrap 표 옆 흐름) 의 텍스트 영역:
- 3 lines × line_spacing 1838 HU = 5514 HU = 73.5 px
- ls[2].vpos (3676) + lh (1150) + ls (688) = 5514 HU = 73.5 px

→ y_offset 기대값 = 211.65 + 73.5 = **285.15 px**

차이 = 285.15 - 272.40 = **12.75 px ≈ baseline 978 HU = 13.04 px**.

## 4. 본질 정리

| 측면 | 현재 | 기대 |
|------|------|------|
| y_offset advance 기준 | 표 bottom + ls (51.4 + 9.17 = 60.57 px) | 호스트 문단 텍스트 영역 (73.5 px) |
| 사용 위치 | `layout_table_item` line 2410 (`layout_table` 반환값) | 호스트 문단 last LINE_SEG vpos + lh + ls |
| Square wrap 의미 | floating (텍스트 옆 흐름) | floating (텍스트 옆 흐름) |

**비-TAC Square wrap 표는 floating 이므로 호스트 문단 텍스트가 표 옆을 흐른다. 호스트 텍스트 길이 (3 LINE_SEGs) > 표 높이 (51.4 px) 일 때 y_offset 이 호스트 텍스트 영역까지 advance 되어야 하는데, 현재는 표 bottom 기준으로 advance** → 호스트 텍스트가 표보다 아래로 늘어진 만큼 (12.75 px) 다음 문단이 위로 시프트됨.

## 5. 좌측 단 동일 패턴 검증

좌측 단 측정 결과 (Stage 1 사전 분석):
- pi=33/37/40/47 직후 (모두 Square wrap host) 도 동일한 11~12 px 좁은 간격 관측

`gap = 11.41 / 11.73 / 11.31 px` — 모두 baseline 978 HU 만큼 시프트.

→ **Square wrap 표 Pi 의 last LINE_SEG vpos + lh + ls > 표 bottom + ls** 인 모든 케이스에 동일 결함 발생.

## 6. 영향 범위

### 6-1. 시각 결함 발생 조건

- `paragraph.controls` 에 `Control::Table(t)` with `t.common.treat_as_char == false && t.common.text_wrap == Square`
- 호스트 문단 텍스트 영역 (`max LINE_SEG vpos + lh`) > 표 높이 (`t.common.height`)

조건 만족 시 y_offset 이 텍스트 영역 - 표 영역 차만큼 일찍 advance → 다음 문단 시프트.

### 6-2. 회귀 위험

표 영역 > 호스트 텍스트 영역 인 경우 (대형 Square wrap 표) — 현재 동작이 정확. 본 fix 가 그쪽을 망가뜨리지 않도록 `max(table_bottom, host_text_bottom)` 사용 필수.

또한:
- exam_kor 19 페이지 다수 (Square wrap 표 + 본문)
- exam_math_no / exam_kor_math (수식 + Square wrap 인라인 표)

## 7. Stage 2 설계 방향 (다음 단계 — 별도 승인 필요)

### Option A — `layout_table_item` 에서 호스트 텍스트 영역 추가 advance

```rust
// line 2418 직후 추가:
if !is_tac && tbl_is_square {
    let host_text_bottom = ...;  // pi=50 last LINE_SEG vpos + lh + ls 기반
    y_offset = y_offset.max(host_text_bottom);
}
```

- 장점: 변경 범위 최소, 기존 표-only advance 유지하며 호스트 텍스트 영역만 max
- 단점: layout_table_item 안에서 호스트 paragraph LINE_SEGs 접근 필요

### Option B — Square wrap host 의 PartialParagraph 처리에서 advance

```rust
// line 2105 if is_wrap_host { return (y_offset, false); } 변경:
if is_wrap_host {
    let host_text_bottom = ...;
    return (y_offset.max(host_text_bottom), false);
}
```

- 장점: PartialParagraph 측면에서 호스트 advance 명확
- 단점: PartialParagraph 가 Table item 보다 먼저 처리되므로 표 advance 와 충돌 가능. Table item 측 동작을 분기로 변경 필요 → 변경 범위 큼

→ **Option A 권장**.

### Option C — VPOS 보정에서 base=0 강제

VPOS_CORR 분기 (Task #412/#332) 가 lazy_base 자체 보정을 가지므로 본 영역에서 처리. 단, lazy_base 알고리즘은 다단/표분할 등 광범위 영역에 영향 → 본 task 의 직접적 정정으로는 부적합.

## 8. 산출물 (본 단계)

| 산출물 | 내용 |
|--------|------|
| 본 보고서 | Root cause 위치 + 측정 + 기대 동작 + 옵션 비교 |
| 코드 변경 | **0** (조사만) |
| 측정 데이터 | RHWP_VPOS_DEBUG 출력 + dump-pages 출력 |

## 9. 다음 단계

작업지시자 승인 후 Stage 2 — 구현계획서 작성 (Option A 기반) + 회귀 테스트 시나리오 정의.

## 10. 승인 게이트

- [x] Root cause 위치 확정 (`src/renderer/layout.rs::layout_table_item` 비-TAC Square wrap 분기)
- [x] 측정 데이터 (VPOS_CORR pi=51 y_in=272.40, 기대 285.15, 차이 12.75 px ≈ baseline 978 HU)
- [x] 좌측 단 동일 패턴 검증 (pi=33/37/40/47)
- [x] 영향 범위 / 회귀 위험 식별 (대형 표 케이스 max 처리 필수)
- [x] Stage 2 옵션 비교 (A 권장)
