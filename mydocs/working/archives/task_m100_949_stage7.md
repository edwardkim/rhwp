# Task m100 #949 Stage 7 작업 보고서: TABLE field diff

## 1. 목적

Stage 6의 `table_bundles.md`는 TABLE 후보 주변 record window를 보여주었다.
Stage 7에서는 같은 후보를 필드 단위로 해석하는 `--report table-fields`를 추가했다.

목적은 raw byte 차이를 다음 단계 probe/구현 후보로 바로 옮길 수 있는 필드명 수준으로
낮추는 것이다.

## 2. 구현 내용

수정 파일:

```text
src/diagnostics/hwp5_inventory.rs
src/diagnostics/hwp5_inventory_diff.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report table-fields \
  --out <path>
```

내부 변경:

```text
1. inventory item에 내부용 payload_bytes 추가
   - JSON/Markdown 출력에는 직렬화하지 않음
2. CTRL_HEADER(Table) P0 field decoder 추가
3. TABLE P0 field decoder 추가
4. field별 diff count summary 출력
```

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage7/hwpx-h-01/table_field_diff.md
```

생성 명령:

```bash
./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --report table-fields \
  --out output/poc/hwpx2hwp/task949/stage7/hwpx-h-01/table_field_diff.md
```

산출물 크기:

```text
739 lines
```

## 4. 주요 결과

후보 record:

```text
CTRL_HEADER(Table): 26
TABLE: 26
```

반복 diff field:

```text
out_margin_left:   26
out_margin_right:  26
out_margin_top:    26
out_margin_bottom: 26
record_size:       26
payload_len:       26
table_attr:        19
tail_after_0x16:   14
common_attr:        9
```

대표 사례:

```text
CTRL_HEADER(Table)
  out_margin_left/right/top/bottom:
    oracle    141 또는 283
    generated 0

TABLE
  record_size / payload_len:
    oracle    generated보다 2 bytes 큼

TABLE
  table_attr:
    oracle    0x0000000e, 0x00000006, 0x0000000c 등
    generated 0x00000005 또는 0x00000004 등

TABLE
  tail_after_0x16:
    oracle    뒤에 `00 00` 또는 추가 tail 존재
    generated 짧거나 `<none>`
```

## 5. 해석

Stage 7 결과는 TABLE 계열에서 다음 두 축이 반복적으로 다르다는 것을 보여준다.

```text
1. CTRL_HEADER(Table)의 outer margin 필드가 generated에서 모두 0으로 떨어진다.
2. TABLE record payload가 generated에서 2 bytes 짧고 table_attr 값도 다르다.
```

이 두 축은 Stage 8 probe 후보로 분리할 수 있다.

```text
1. CTRL_HEADER(Table) outer margin만 oracle 값으로 보정
2. TABLE table_attr만 oracle 값으로 보정
3. TABLE tail_after_0x16만 oracle 값으로 보정
4. 위 세 축을 조합한 최소 positive probe
```

## 6. 주의 사항

필드명은 Stage 7 P0 decoder 기준이다.
HWP5 정식 필드 의미를 확정한 것이 아니라, 반복적으로 차이가 나는 payload 위치를 사람이
다룰 수 있도록 이름 붙인 것이다.

특히 다음 이름은 후속 검증이 필요하다.

```text
z_order_or_instance
row_count_hint
col_count_hint
tail_after_0x16
```

## 7. 검증

```text
cargo build: 통과
--report table-fields: 산출물 생성 통과
--report bundles smoke: 통과
```

## 8. 다음 단계

Stage 8에서는 TABLE 필드 diff 결과를 기준으로 probe를 만든다.

우선순위:

```text
1. CTRL_HEADER(Table) outer margin 보정 probe
2. TABLE table_attr 보정 probe
3. TABLE tail 2 bytes 보정 probe
4. 1+2+3 조합 probe
```
