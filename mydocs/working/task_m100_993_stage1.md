# Task M100-993 Stage 1 작업 기록

## 1. 목적

#993 P1 작업의 첫 단계로 `mel-001` 샘플에서 발생하는 HWPX to HWP 저장 후
한컴 에디터 `파일손상` 판정 원인을 record contract 관점에서 좁힌다.

대상 파일:

```text
source HWPX: samples/hwpx/mel-001.hwpx
oracle HWP: samples/hwpx/hancom-hwp/mel-001.hwp
generated HWP: output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/generated-mel-001.hwp
```

작업지시자 관찰:

```text
- generated HWP를 한컴 에디터에서 열면 파일손상 판정을 받는다.
- 2페이지 첫 표의 셀 테두리/배경 처리가 잘못 저장되는 것으로 보인다.
- 한컴 에디터는 2페이지 "󰊳  예산 현황" 까지 출력한 뒤 중지한다.
```

## 2. 산출물

산출물 디렉터리:

```text
output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/
```

생성/분석 파일:

```text
generated-mel-001.hwp
inventory_oracle_section0.jsonl
inventory_generated_section0.jsonl
inventory_diff_lcs.md
inventory_diff_table_bundles.md
table_field_diff.md
contract_violation_hints.md
contract_analyze/bodytext_control_graph.md
contract_analyze/docinfo_contract.md
contract_analyze/hwpx_ir_serializer_trace.md
contract_analyze/stage12_index.md
```

## 3. 재현 결과

현재 adapter로 `samples/hwpx/mel-001.hwpx`를 HWP로 저장하면 다음 파일이 만들어진다.

```text
output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/generated-mel-001.hwp
```

rhwp 기준 정보:

| file | size | rhwp load | pages | sections |
|---|---:|---|---:|---:|
| oracle `samples/hwpx/hancom-hwp/mel-001.hwp` | 268,800 bytes | 성공 | 20 | 1 |
| generated `generated-mel-001.hwp` | 160,768 bytes | 성공 | 20 | 1 |

## 4. 페이지 단위 관찰

rhwp의 page dump 기준으로 oracle/generated의 2페이지 high-level layout은 동일하다.

```text
pi=17  Table 1x1
pi=18  "󰊱  기구 및 조직 현황"
pi=19  Table 24x47
pi=20  Table 10x65
pi=21  "󰊲  인원 현황(’25.11.30. 기준)"
pi=22  Table 8x12
pi=23  주석 문단
pi=24  "󰊳  예산 현황"
pi=25  Table 12x5
```

따라서 이번 문제는 IR pagination이 즉시 깨지는 유형이 아니다. rhwp는 generated HWP를
20페이지로 재로드하지만, 한컴 HWP5 loader는 특정 binary record contract 위반으로
2페이지 `예산 현황` 주변에서 손상 판정을 내리는 것으로 보아야 한다.

## 5. record contract 후보

### 후보 1: `예산 현황` 직후 표의 `CTRL_HEADER(Table).common_attr`

`table_field_diff.md` 기준으로 `예산 현황` 뒤 12x5 표의 `CTRL_HEADER(Table)`에서
`common_attr` 1필드가 다르다.

```text
record key: BodyText.Section0#4923~BodyText.Section0#4922
tag: CTRL_HEADER
control: Table
record_size: 46

oracle:    common_attr = 0x282a2311
generated: common_attr = 0x082a2311
diff:      0x20000000
```

나머지 주요 필드는 동일하다.

```text
x=0
y=0
width=47152
height=14976
z_order_or_instance=6
out_margin_left/right/top/bottom=141
tail_after_0x24 동일
```

이 후보는 한컴 중단 위치와 가장 가깝다. Stage 2에서 첫 번째 probe로 분리해야 한다.

### 후보 2: 2페이지 8x12 표의 `TABLE.tail_after_0x16`

`인원 현황` 아래 8x12 표의 `TABLE` record tail에 2바이트 차이가 있다.

```text
record key: BodyText.Section0#4470~BodyText.Section0#4469
tag: TABLE
rows=8
cols=12
record_size=38

oracle tail:    0a 00 0c 00 0a 00 0c 00 0a 00 0c 00 0a 00 00 00
generated tail: 0a 00 0c 00 0a 00 0c 00 0a 00 0c 00 61 00 00 00
```

표의 row/col, 안쪽 여백, `table_attr`는 동일하므로 `TABLE` payload의 후반부
materialization이 한컴 정답과 다르다. 작업지시자가 언급한 셀 테두리/배경 이상과
연관될 가능성이 있다.

### 후보 3: 2페이지 10x65 표 앞 `CTRL_DATA` 누락

`bodytext_control_graph.md`와 `contract_violation_hints.md`에서 oracle에만 존재하는
`CTRL_DATA`가 확인되었다.

```text
record key: BodyText.Section0#3215
parent: Table#3
oracle: CTRL_DATA size=104 hash=ce9e20fff7652f18
generated: 없음
```

이 위치는 `기구 및 조직 현황` 아래 10x65 표 주변이며, 한컴 중단 위치보다 앞선다.
직접 중단 지점은 아니지만, 한컴 HWP5 loader가 table control tuple 전체를 엄격하게
검증한다면 선행 표의 누락 `CTRL_DATA`도 손상 판정의 누적 원인이 될 수 있다.

## 6. 해석

Stage 1에서 확인한 사실:

```text
1. generated HWP는 rhwp로 정상 재로드되며 20페이지로 조판된다.
2. 한컴 중단 위치의 high-level IR 구조는 oracle/generated가 동일하다.
3. 한컴 중단 위치에 가장 가까운 record 차이는 12x5 예산 표의 CTRL_HEADER common_attr bit 0x20000000 누락이다.
4. 작업지시자가 지적한 표 테두리/배경 이상과 관련된 table payload 차이도 2페이지 선행 8x12 표에서 확인된다.
5. 선행 10x65 표에는 oracle-only CTRL_DATA가 있다.
```

따라서 다음 단계는 구현을 넓게 수정하는 것이 아니라, 위 세 축을 각각 독립 probe로 분리해야 한다.

## 7. 다음 단계 제안

Stage 2는 `mel-001` 전용 probe를 생성해 한컴 판정을 받는다.

```text
01_current_generated
02_budget_table_ctrl_common_attr_oracle
03_personnel_table_tail_oracle
04_org_table_missing_ctrl_data_oracle
05_budget_common_attr_plus_personnel_tail
06_all_three_candidates
```

판정 목적:

```text
- 02가 성공하면 예산 표 CTRL_HEADER common_attr bit 0x20000000이 핵심이다.
- 03이 성공하면 8x12 표 TABLE tail materialization이 핵심이다.
- 04가 성공하면 TABLE CTRL_DATA 누락이 핵심이다.
- 개별 후보가 실패하고 05/06만 성공하면 table tuple contract가 복합 조건이다.
```

## 8. 구현 후보 파일

Stage 2 probe 판정 후 실제 구현이 필요한 범위는 다음으로 예상한다.

```text
src/document_core/converters/hwpx_to_hwp.rs
src/document_core/converters/common_obj_attr_writer.rs
src/serializer/control.rs
```

현재 관찰상 우선순위는 `hwpx_to_hwp.rs`의 HWPX to HWP adapter materialization이다.
`serializer/control.rs`는 이미 IR에 채워진 `raw_ctrl_data`, `raw_table_record_extra`,
`raw_table_record_attr`를 직렬화하는 역할이므로, 먼저 IR materialization 단계에서
정확한 값을 채우는지 검증해야 한다.
