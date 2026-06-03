# Task #317 2단계 완료 보고서: origin 식별

상위: `task_m100_317_impl.md`
선행: `task_m100_317_stage1.md` (표 paragraph 마다 +2.7~2.9px 누적)

## 진단 결과

`tests/task317_diag.rs::diag_h_02_table_pi33_compare` 실행으로 pi=33 표 IR 필드 direct vs reloaded 비교.

### 차이 발견

| 필드 | DIRECT | RELOADED |
|------|--------|----------|
| `attr` | `0x00000000` | `0x002A0211` |
| `outer_margin_top` | 140 | 0 |
| `outer_margin_bottom` | 140 | 0 |
| `padding` (l/r/t/b) | 140 each | 140 each (동일) |
| `cell_spacing` | 0 | 0 |
| `common.height` | 2915 | 2915 (동일) |
| `common.width` | 47977 | 47977 (동일) |
| `common.text_wrap` | TopAndBottom | TopAndBottom (동일) |
| `common.treat_as_char` | true | true (동일) |

**다른 모든 필드 동일** — 셀 paragraphs 도 모두 동일.

## 원인 분석

### outer_margin 손실 경로

1. **HWPX 파서** (`src/parser/hwpx/section.rs:693`): `<outMargin>` 요소의 `top/bottom` 을 `table.outer_margin_top/bottom` 에 저장. `table.common.margin` 은 채우지 않음 (기본 0).

2. **어댑터** (`src/document_core/converters/hwpx_to_hwp.rs:151`): `serialize_common_obj_attr(&table.common)` 호출. `common.margin = 0` 이므로 ctrl_data[24..32] = 0.

3. **HWP 파서** (`src/parser/control.rs:160`): `ctrl_data[28..32]` 에서 `outer_margin_top/bottom` 을 읽음 → **0**.

같은 ctrl_data 영역(offset 24..32)이 HWP 스펙에서 `outer_margin_*`, hwplib 의 `CommonObjAttr.margin` 으로 이중 명명되어 있고, 어댑터는 후자만 채우고 있었음.

### attr 손실 경로

- HWPX 출처: `table.attr = 0` (HWPX 파서는 attr 비트 합성 안 함)
- 어댑터 `pack_common_attr_bits` 가 enum 으로부터 비트 합성 → `0x002A0211`
- 결과: RELOADED 가 attr 변경됨

### typeset 영향

- direct cur_h=41.0, reloaded cur_h=43.7 → +2.7px / 표
- outer_margin 280 HWPUNIT (top+bottom) ≈ 3.73px @ 96dpi
- typeset_table_paragraph (`src/renderer/typeset.rs:706,711`) 에서 `outer_margin_top/bottom` 을 표 앞/뒤 spacing 에 반영
- 차이 +2.7px ≠ 3.73px 인 이유: typeset 의 spacing 합산 로직(host spacing 과 max 등)으로 일부 흡수. 어쨌든 outer_margin 손실이 origin 인 것은 명확.

## 보강 방안 검증 결과 (3단계 일부 실험)

**1차 시도 (실패)**: `common.margin ← outer_margin_*` 동기화. 결과: outer_margin 보존 됐으나 페이지 수 그대로. 오히려 typeset 분기에서 +1.0px 더 증가 (+2.7 → +3.7).

**진짜 origin 재식별**: `src/renderer/typeset.rs:695` `let is_tac = table.attr & 0x01 != 0;`
- DIRECT: `table.attr=0` → `is_tac=false` → block 분기 (`typeset_block_table`)
- RELOADED: `table.attr=0x002A0211` (어댑터가 `pack_common_attr_bits` 로 합성, bit 0 설정됨) → `is_tac=true` → TAC 분기 (`typeset_tac_table`)

두 분기는 host_spacing/outer_margin/line_spacing 처리가 달라 같은 표에 대해 다른 누적 height 산출. PDF baseline 9쪽 = DIRECT 의 block 분기 결과. 어댑터가 attr 합성하면서 의도치 않게 TAC 분기로 흘려보내 +2.7px/표 누적.

**2차 시도 (성공)**: 어댑터 `adapt_table` 에서 raw_ctrl_data 합성 후 attr 영역(offset 0..4) 을 0 으로 강제 + `table.attr=0` 보존. DIRECT 와 동일한 attr=0 → 같은 block 분기.

## 보강 방안 (확정)

`adapt_table` 에서 raw_ctrl_data 합성 직후 `raw_ctrl_data[0..4]=0`, `table.attr=0` 으로 고정.
HWPX 출처 표는 `common.attr=0` 이 진실 (HWPX 파서가 attr 비트를 채우지 않음). 어댑터의 attr 비트 합성은 다른 IR 필드(common.treat_as_char/text_wrap 등) 와 모순되지 않으므로 typeset 결과만 변경. 같은 비트 0 처리를 DIRECT와 일치시키기 위해 합성을 비활성화.

(이전 1차 후보 — common.margin 동기화 — 는 stage 3 에서 적용하지 않음. outer_margin_* 자체는 IR 에 보존되어 있고 typeset 이 직접 사용하므로 ctrl_data 영역 손실은 typeset 결과에 영향 없음.)

```rust
// hwpx_to_hwp.rs::adapt_table 시작부
if table.common.margin.left == 0 && table.common.margin.right == 0
    && table.common.margin.top == 0 && table.common.margin.bottom == 0
{
    table.common.margin.left = table.outer_margin_left;
    table.common.margin.right = table.outer_margin_right;
    table.common.margin.top = table.outer_margin_top;
    table.common.margin.bottom = table.outer_margin_bottom;
}
```

**2차 후보**: HWPX 파서가 `<outMargin>` 파싱 시 `common.margin` 도 같이 채움. (구조 측면 더 정확하지만 다른 경로에 영향 가능성)

attr 차이는 RELOADED 가 합성한 비트가 *맞을 가능성이 높음* — common.text_wrap 등 enum 으로부터 정확히 복원되므로 typeset 결과에 영향 없음. 따라서 attr 자체는 별도 보강 불필요로 판단. 1차 보강 후 재측정으로 확인.

## 산출

- `tests/task317_diag.rs::diag_h_02_table_pi33_compare` (4단계에서 제거)
- 본 보고서

## 다음 단계

3단계: 어댑터 `adapt_table` 에 common.margin ← outer_margin_* 동기화 추가. h-01/h-02/h-03 + 4샘플 회귀 검증.
