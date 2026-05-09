# Task #712 Stage 2 (분석) 완료 보고서

**Issue**: [#712](https://github.com/edwardkim/rhwp/issues/712)
**Stage**: 2 — Root cause 분석
**작성일**: 2026-05-08

---

## 1. 트레이스 인스트루먼트

다음 위치에 임시 디버그 인쇄 추가 (Stage 3 GREEN 종료 시 제거):

| 위치 | 출력 |
|------|------|
| `layout.rs:1556` (column_item 진입/종료) | `item_para`, `y_offset`, `new_y`, `was_tac` |
| `layout.rs:2402+` (`layout_table_item`) | `pi`, `ci`, `y_offset`, `table_y_start`, `para_y_for_table` |
| `layout.rs:2686+` (`layout_partial_table_item`) | `pi`, `ci`, `pt_y_start`, `vert_off`, `wrap`, `vrel` |
| `table_partial.rs:62+` (gate ENTER) | `y_start_in`, `y_start_after_gate`, `vert_off` |
| `table_partial.rs:227+` (table node 생성 직전) | `table_y`, `table_x`, `partial_h` |

환경변수: `RHWP_TASK712_DEBUG=1`

## 2. 트레이스 결과 (page index 35, page_count=40)

```
TASK712: column_item ENTER item_para=585 y_offset=94.48 prev_layout_para=Some(585)
TASK712: layout_table_item pi=585 ci=0 y_offset_before=98.25 table_y_start=98.25 para_y_for_table=94.48 is_tac=true
TASK712: layout_table returned pi=585 y_offset_after=137.11
TASK712: column_item EXIT  item_para=585 new_y=148.88 was_tac=true

TASK712: column_item ENTER item_para=586 y_offset=148.88 prev_layout_para=Some(586)
TASK712: layout_partial_table_item pi=586 ci=0 y_offset=148.88 pt_y_start=148.88 para_start_y=-999.00 vert_off=4294965500 wrap=TopAndBottom vrel=Para tac=false is_continuation=false start_row=0 end_row=9
TASK712: partial_table ENTER pi=586 ci=0 y_start_in=148.88 y_start_after_gate=124.93 vert_off=4294965500 wrap=TopAndBottom vrel=Para tac=false is_cont=false
TASK712: partial_table BEFORE_NODE pi=586 table_y=124.93 table_x=75.59 table_w=636.71 partial_h=879.37
TASK712: layout_partial_table returned pi=586 y_offset=1004.31
TASK712: column_item EXIT  item_para=586 new_y=1004.31 was_tac=false
```

### 핵심 시퀀스 분석

| 단계 | y 값 (px) | 이벤트 |
|------|----------|-------|
| pi=585 진입 | 94.48 | body_top, 정상 |
| pi=585 layout_table 반환 | 137.11 | 1x3 표 cell 하단 (정상) |
| pi=585 종료 | **148.88** | + line_spacing(8.0) + outer_margin_bottom(3.77) (정상) |
| pi=586 진입 | 148.88 | y_offset 정상 누적 |
| pi=586 pt_y_start | 148.88 | gate 통과 후에도 정상 (para_start_y 미존재로 unwrap_or fallback) |
| **`partial_table` 게이트 진입** | y_start_in=148.88 | |
| **게이트 통과 후** | **y_start_after_gate=124.93** | ← **148.88 - 23.95 = -1796 HU 적용** |
| layout_partial_table 반환 | 1004.31 | table 하단 = 124.93 + 879.37 |

**침범 = 148.88 - 124.93 = 23.95 px ≡ vert_offset(-1796 HU) 절댓값**

## 3. Root cause 확정

### 코드 (`src/renderer/layout/table_partial.rs:62-71`)

```rust
// 분할 표 첫 부분: vert_offset 적용 (자리차지 표의 세로 오프셋)
let y_start = if !is_continuation && !table.common.treat_as_char
    && matches!(table.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
    && table.common.vertical_offset > 0          // ← BUG
{
    y_start + hwpunit_to_px(table.common.vertical_offset as i32, self.dpi)
} else {
    y_start
};
```

### 메커니즘

1. `HwpUnit = u32` (확인: `src/model/mod.rs:21`)
2. pi=586 의 12x5 표 `vertical_offset = -1796 HU` 가 u32로 저장 → `0xFFFFF8FC = 4294965500u32`
3. `vertical_offset > 0` → `4294965500 > 0` → **TRUE** (의도된 양수 가드 우회)
4. 게이트 통과 후 `vertical_offset as i32` 는 비트 그대로 i32 해석 → **-1796**
5. `hwpunit_to_px(-1796, 96) = -23.95 px`
6. `y_start + (-23.95) = 148.88 - 23.95 = 124.93` → 표가 위로 점프, pi=585 영역 침범

### 동일 코드 패턴 (또 다른 게이트)

`src/renderer/layout.rs:2673-2685` 의 `pt_y_start` 게이트도 동일한 `vertical_offset > 0` 사용:

```rust
let pt_y_start = if let Some(para) = paragraphs.get(para_index) {
    if let Some(Control::Table(t)) = para.controls.get(control_index) {
        if !t.common.treat_as_char
            && matches!(t.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
            && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
            && t.common.vertical_offset > 0     // ← 동일 BUG
        {
            para_start_y.get(&para_index).copied().unwrap_or(y_offset)
        } else {
            y_offset
        }
    } else { y_offset }
} else { y_offset };
```

본 케이스에서는 `para_start_y` 미존재로 `unwrap_or(y_offset)` 분기를 통해 우연히 정상 동작 (pt_y_start = 148.88). 그러나 다른 케이스에서 동일 결함을 발생시킬 수 있어 **함께 정정** 필요.

### 비-PartialTable 경로와의 비교

`src/renderer/layout/table_layout.rs:1069-1124` 의 동일 분기:

```rust
let v_offset = hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
// ...
let raw_y = match vert_align {
    VertAlign::Top => ref_y + v_offset + caption_top_offset,
    // ...
};
// Para 기준 + bit 13: 본문 영역으로 제한
if matches!(vert_rel_to, crate::model::shape::VertRelTo::Para) {
    let pushed = if matches!(table_text_wrap, TextWrap::TopAndBottom) {
        raw_y.max(y_start)   // ← 음수 offset 으로 위로 가는 것 방지
    } else {
        raw_y
    };
    pushed.clamp(body_top, body_bottom.max(body_top))
}
```

비-Partial 경로에는 **`raw_y.max(y_start)` 클램프**가 있어 음수 offset이 위로 점프하는 것을 차단. **PartialTable 경로에는 동일 클램프 부재** — 정정 시 클램프 또는 게이트 signed 비교 둘 중 하나 적용.

## 4. 가설 매핑

| 가설 (Stage 0) | 결과 |
|----------------|------|
| H1 (vpos-reset base 오류) | ❌ 무관 |
| H2 (vert_offset 이중 적용) | ❌ 무관 (단일 적용) |
| **H3 (pt_y_start 가드 음수 누락)** | ✅ **CONFIRMED** — `table_partial.rs:62-71` 의 `> 0` 가드가 u32 의 음수 비트표현에서 거짓 통과 + 클램프 부재 |

## 5. 정정 방향 (Stage 3)

**최소 외과적 패치 — 두 곳의 게이트를 signed 비교로 변경**:

```rust
// table_partial.rs:62-71
let vert_off_signed = table.common.vertical_offset as i32;
let y_start = if !is_continuation && !table.common.treat_as_char
    && matches!(table.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
    && vert_off_signed > 0   // signed
{
    y_start + hwpunit_to_px(vert_off_signed, self.dpi)
} else {
    y_start
};
```

`layout.rs:2673-2685` 도 동일하게 `(t.common.vertical_offset as i32) > 0` 으로 변경.

### 안전성 분석

- **양수 offset (의도된 케이스)**: 변경 없음 — 그대로 적용
- **0 offset**: 게이트 차단 (변경 없음)
- **음수 offset (현 결함)**: 게이트 차단 → y_start 유지 → pi=585 침범 0
- 비-Partial 경로의 `raw_y.max(y_start)` 와 동등한 효과 (음수 offset 무시)

## 6. 다음 단계 (Stage 3 — GREEN)

1. 트레이스 인스트루먼트 제거
2. `table_partial.rs:62-71` 와 `layout.rs:2673-2685` 게이트 signed 비교로 변경
3. `cargo test --test issue_712` PASS 확인
4. 단계별 보고서 + 커밋

## 승인 요청

Stage 2 분석 단계 완료. Root cause 확정. Stage 3 (GREEN) 진행 승인 요청.
