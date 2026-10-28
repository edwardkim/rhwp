---
kind: working
status: active
issue: 5469
---

# HWP5 table-probe 축 지도 (#5469)

`hwp5-inventory-diff --report table-probe-plan` 과
`hwp5-table-probe` 가 나누는 네 축을 픽스처 언어로 고정한다.
페이지 수는 이 지도의 출력이 아니다.

## 1. 언제 이 지도를 펼치나

oracle/generated 인벤토리에서 표 후보가 나왔을 때.

```text
rhwp hwp5-inventory-diff oracle.hwp generated.hwp \
  --align lcs --report table-probe-plan --focus table --section 0
rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir out --section 0
```

이 저장소의 같은 문장:

```text
tools/hwp5_inventory/transcripts/inventory_diff/T01.table-probe-plan.md
tools/hwp5_inventory/transcripts/table_probe/T01.generation.md
```

## 2. 네 축

| 축 | 레코드 | 오프셋 | T01–T04 |
|---|---|---|---|
| `ctrl_outer_margin` | CTRL_HEADER(Table) | 0x1c left, 0x1e right, 0x20 top, 0x22 bottom | T01 |
| `ctrl_common_attr` | CTRL_HEADER(Table) | 0x04 u32 | T04 |
| `table_attr` | TABLE | 0x00 u32 | T02 |
| `table_tail` | TABLE | 0x16 .. end | T03 |

같이 나오는 필드인데 축이 아닌 것:

- `x` / `y` / `width` / `height` (0x08..0x17) — 상자. probe 이식이 아니다.
- `z_order_or_instance` (0x18) — 관찰명.
- `rows` / `cols` / in-margin / count hint — count 계약(C). graft 축이 아니다.
- `tail_after_0x24` — 관찰명. `table_tail` 과 다른 레코드다.

## 3. 8 variant

| 파일 | 축 | 쓰는 때 |
|---|---|---|
| `01_ctrl_outer_margin_only` | margin | 표가 왼쪽에 붙음 |
| `02_table_attr_only` | table_attr | 배치만 이상, 여백은 같음 |
| `03_table_tail_only` | tail | 표 직후 손상 |
| `04_ctrl_common_attr_only` | common_attr | 흐름 비트 후보 |
| `05_outer_margin_table_attr` | margin+attr | 01/02 가 반만 회복 |
| `06_outer_margin_table_tail` | margin+tail | 위치+손상 |
| `07_table_attr_tail` | attr+tail | TABLE 내부만 |
| `08_all_table_axes` | 넷 | positive guard. 원인 분리 아님 |

규칙:

```text
08 만 성공하고 01-04 가 실패하면 승격하지 않는다.
한 축 no-op (이식 0) 인 variant 를 한컴 판정에 쓰지 않는다.
rhwp-studio 재로드 성공은 한컴 호환이 아니다.
```

## 4. 픽스처가 재는 것

`fatten_catalog.py` 는 probe HWP 바이트를 쓰지 않는다. 축별로
`affected_records` 만 센다. `T01` 은 `ctrl_outer_margin` 만 1,
나머지는 0 이어야 한다. `T08` 은 네 축이 함께 선다.

`reports/probe_axis_matrix.md` 가 전 케이스 표다.

## 5. 표가 아닌데 table-probe 를 부르지 마라

그림/수식/필드/DocInfo 후보는 이 네 축이 아니다.

| 증상 | 케이스 | 다음 |
|---|---|---|
| 이미지 없음 | S01 | BIN_DATA + SHAPE_PICTURE |
| 수식 직후 손상 | C01 | EQEDIT graft |
| 필드 종류 붕괴 | F04 | fourcc / command |
| section_count | D01 | DocInfo. 쪽수 계산기 아님 |

## 6. 페이지 수

한컴에서 쪽이 늘어나는 현상은 `#4882` / `#4898` 석이다.
table-probe 리포트의 `rhwp_pages` 칸을 여기서 채우거나 serializer 에
넣지 않는다.
