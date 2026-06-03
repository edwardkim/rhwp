# Task m100 #949 Stage 7 계획서: TABLE field diff report

## 1. 목적

Stage 6의 `table_bundles.md`는 TABLE 후보 주변 record window를 보여준다.
하지만 아직 차이는 raw byte/head 수준이다.

Stage 7에서는 TABLE 관련 record를 필드 단위로 해석하는 보고서를 만든다.

목표 산출물:

```text
output/poc/hwpx2hwp/task949/stage7/hwpx-h-01/table_field_diff.md
```

## 2. 구현 범위

추가 CLI:

```bash
--report table-fields
```

대상:

```text
1. CTRL_HEADER(Table)
2. TABLE
3. TABLE 직후 LIST_HEADER
4. TABLE 직후 PARA_HEADER
```

Stage 7에서는 우선 HWP5 record payload의 알려진/관찰 가능한 위치를 해석한다.
정식 contract 확정은 하지 않는다.

## 3. 보고서 구성

```text
# HWP5 Table Field Diff

## Input
## Summary
## Table Field Rows
```

각 row는 다음을 가진다.

```text
record key
record tag
oracle/generator size
field name
offset
oracle value
generated value
diff 여부
```

## 4. 필드 해석 P0

`CTRL_HEADER(Table)`:

```text
0x00 u32 ctrl_id
0x04 u32 common_attr
0x08 i32 x
0x0c i32 y
0x10 i32 width
0x14 i32 height
0x18 u32 z_order_or_instance
0x1c u16 out_margin_left
0x1e u16 out_margin_right
0x20 u16 out_margin_top
0x22 u16 out_margin_bottom
```

`TABLE`:

```text
0x00 u32 table_attr
0x04 u16 rows
0x06 u16 cols
0x08 u16 cell_spacing
0x0a u16 in_margin_left
0x0c u16 in_margin_right
0x0e u16 in_margin_top
0x10 u16 in_margin_bottom
0x12 u16 row_count_hint
0x14 u16 col_count_hint
```

`LIST_HEADER`와 `PARA_HEADER`는 이번 단계에서 size와 head/tail 중심으로 비교한다.

## 5. 검증 명령

```bash
cargo build

./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --report table-fields \
  --out output/poc/hwpx2hwp/task949/stage7/hwpx-h-01/table_field_diff.md
```

## 6. 완료 기준

```text
1. cargo build 통과
2. 기존 diff/hints/bundles report 유지
3. TABLE field diff 산출물 생성
4. CTRL_HEADER(Table)와 TABLE의 반복 차이가 필드명으로 보임
5. Stage 7 작업 보고서 작성
```

## 7. 다음 단계

Stage 8에서는 Stage 7 결과를 바탕으로 TABLE contract 후보를 실제 probe 또는 저장기 구현
후보로 분리한다.
