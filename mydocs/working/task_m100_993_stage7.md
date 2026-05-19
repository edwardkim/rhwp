# Task M100-993 Stage 7 Working Note

## 1. 목적

`tb-org-02` 조직도 표에서 한컴 에디터가 셀 내부 텍스트를 한글자씩 줄나눔하여 셀 높이가 증가하는
원인을 정답 HWP, 생성 HWP, HWPX 원본의 셀 텍스트 구조와 HWP5 record 차이로 분리한다.

## 2. 생성 산출물

```text
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/table_field_diff.md
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/table_bundles.md
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/inventory_diff_lcs.md
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/oracle_section0_inventory.jsonl
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/generated_section0_inventory.jsonl
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/tb_org_cell_text_structure_oracle.md
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/tb_org_cell_text_structure_generated.md
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/tb_org_cell_text_structure_hwpx.md
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/tb_org_cell_linebreak_contract_findings.md
```

## 3. 핵심 확인

정답 HWP, 생성 HWP, HWPX 원본 모두 대표 조직도 셀에서 2글자 단위 문단 구조를 가진다.

| 대상 | 구조 |
|---|---|
| HWPX 원본 | 여러 `<hp:p>`에 `한국`, `산업`, `안전`, `보건`, `공단`처럼 2글자 단위 저장 |
| 정답 HWP | `한국|산업|안전|보건|공단`, paras=5 |
| 생성 HWP | `한국|산업|안전|보건|공단`, paras=5 |

따라서 문제는 HWPX parser나 IR에서 문단을 합치는 것이 아니다.

## 4. HWP5 record 차이

고정 table/cell field diff는 0건이다.

```text
candidate_count: 0
diff field count: 0
```

하지만 raw inventory diff에서 대표 셀에 다음 차이가 반복된다.

```text
1. 셀 LIST_HEADER
   oracle:    size=47
   generated: size=34

2. 셀 내부 PARA_HEADER
   oracle:    size=24
   generated: size=22
```

대표 셀 `한국산업안전보건공단`의 LIST_HEADER 차이:

```text
oracle:
05 00 00 00 00 00 01 00 ... 09 00 da 08 00 00 00 00 00 00 00 00 00 00 00

generated:
05 00 00 00 00 00 00 00 ... 09 00
```

해석:

```text
bytes 6..7: oracle=0x0001, generated=0x0000
extra first u32: 2266, 해당 셀 폭과 동일
```

대표 셀 `학교법인한국폴리텍`도 같은 패턴이다.

```text
bytes 6..7: oracle=0x0001, generated=0x0000
extra first u32: 2568, 해당 셀 폭과 동일
```

## 5. 판정

Stage 7로 원인 축은 다음과 같이 좁힌다.

```text
한컴 에디터의 1글자 줄나눔은 텍스트 조각 분실 문제가 아니다.
HWP5 셀 LIST_HEADER의 width_ref/extra 및 셀 내부 PARA_HEADER 확장 필드가 빠져,
한컴이 셀 내부 문단 폭을 정답 HWP와 다르게 해석하는 문제다.
```

우선순위 후보:

```text
1. 셀 LIST_HEADER bytes 6..7 = 0x0001
2. 셀 LIST_HEADER 34바이트 이후 extra 13바이트
   - 첫 4바이트는 셀 폭
   - 나머지 9바이트는 현재 관찰상 0
3. 셀 내부 PARA_HEADER byte 21 = 0x80 및 tail 2바이트
```

## 6. 다음 단계

Stage 8에서는 셀 높이를 보정하지 않고, 위 record 축만 분리하는 probe를 생성한다.

```text
01_current
02_target_org_cell_list_width_ref_only
03_target_org_cell_list_extra_only
04_target_org_cell_list_width_ref_plus_extra
05_target_org_cell_para_header_tail_only
06_target_org_cell_list_bundle_plus_para_header_tail
07_all_org_cells_list_width_ref_plus_extra
08_all_org_cells_list_bundle_plus_para_header_tail
```

성공 기준은 한컴 에디터에서 조직도 셀 텍스트가 다시 2글자 단위로 표시되고, 표 높이가 정답과
같아지는 것이다.

## 7. 실행한 검증

```text
cargo run --quiet --bin rhwp -- dump samples/hwpx/hancom-hwp/tb-org-02.hwp
cargo run --quiet --bin rhwp -- dump output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp
cargo run --quiet --bin rhwp -- dump samples/hwpx/tb-org-02.hwpx
cargo run --quiet --bin rhwp -- hwp5-inventory-diff samples/hwpx/hancom-hwp/tb-org-02.hwp output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp --align lcs --report table-fields --section 0
cargo run --quiet --bin rhwp -- hwp5-inventory-diff samples/hwpx/hancom-hwp/tb-org-02.hwp output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp --align lcs --report diff --section 0
cargo run --quiet --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/tb-org-02.hwp --format jsonl --section 0
cargo run --quiet --bin rhwp -- hwp5-inventory output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp --format jsonl --section 0
```

