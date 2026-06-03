# Task m100 #949 Stage 8 작업 보고서: TABLE probe manifest

## 1. 목적

Stage 7에서 TABLE/CTRL_HEADER(Table) 필드 차이를 확인했다.
Stage 8에서는 이 결과를 다음 판정 파일 생성 단계에서 바로 사용할 수 있는 probe manifest로
자동 정리했다.

이번 단계는 판정용 HWP를 직접 생성하지 않는다. 대신 어떤 필드 축을 어떤 record에 적용할지
기계적으로 고정한다.

## 2. 구현 내용

수정 파일:

```text
src/diagnostics/hwp5_inventory_diff.rs
src/main.rs
```

추가 CLI:

```bash
rhwp hwp5-inventory-diff <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report table-probe-plan \
  --out <path>
```

추가 report:

```text
table-probe-plan
```

## 3. 생성 산출물

```text
output/poc/hwpx2hwp/task949/stage8/hwpx-h-01/table_probe_plan.md
```

생성 명령:

```bash
./target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --align lcs \
  --report table-probe-plan \
  --out output/poc/hwpx2hwp/task949/stage8/hwpx-h-01/table_probe_plan.md
```

산출물 크기:

```text
143 lines
```

## 4. Probe 축 요약

```text
ctrl_outer_margin: 26 records
ctrl_common_attr:   9 records
table_attr:        19 records
table_tail:        26 records
```

해석:

```text
1. CTRL_HEADER(Table)의 outer margin은 26개 TABLE control 모두에서 generated가 0이다.
2. CTRL_HEADER(Table)의 common_attr는 9개 record에서 generated가 0이다.
3. TABLE table_attr는 19개 record에서 다르다.
4. TABLE tail full payload는 26개 record에서 짧거나 누락된다.
```

주의:

```text
Stage 7 table-fields의 tail_after_0x16 표시는 표시 길이 제한 때문에 14개 diff로 보였다.
Stage 9 probe 생성 과정에서 full tail을 비교하니 TABLE 26개 모두에서 tail/length 차이가 확인되었다.
따라서 Stage 8 manifest는 full tail 기준으로 재생성했다.
```

## 5. 권장 Probe Matrix

```text
01_ctrl_outer_margin_only
02_table_attr_only
03_table_tail_only
04_ctrl_common_attr_only
05_outer_margin_table_attr
06_outer_margin_table_tail
07_table_attr_tail
08_all_table_axes
```

이 matrix는 다음 단계에서 판정용 HWP를 만들 때 사용한다.

## 6. 주의 사항

이 단계의 필드명은 P0 decoder의 관찰명이다.

```text
tail_after_0x16
z_order_or_instance
row_count_hint
col_count_hint
```

위 이름들은 HWP5 contract의 최종 명칭으로 확정하지 않는다.

## 7. 검증

```text
cargo build: 통과
--report table-probe-plan: 산출물 생성 통과
```

## 8. 다음 단계

Stage 9에서는 이 manifest를 입력으로 판정용 HWP 생성 방식을 결정한다.

우선순위:

```text
1. binary graft 방식으로 8개 matrix를 생성할 수 있는지 확인
2. 가능하면 output/poc/hwpx2hwp/task949/stage9/... 아래에 판정 파일 생성
3. 불가능하거나 위험하면 저장기 projection 쪽에 같은 axis를 적용하는 실험으로 전환
```
