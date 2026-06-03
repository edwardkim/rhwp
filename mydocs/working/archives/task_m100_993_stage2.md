# Task M100-993 Stage 2 작업 기록

## 1. 목적

Stage 1에서 분리한 `mel-001` 한컴 `파일손상` 후보를 판정용 HWP probe로 생성한다.

이번 단계는 구현 변경이 아니라 oracle HWP의 일부 record contract를 generated HWP에 graft해
한컴 에디터 판정을 받기 위한 파일을 만드는 단계다.

## 2. 입력

```text
oracle HWP:
samples/hwpx/hancom-hwp/mel-001.hwp

generated baseline:
output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/generated-mel-001.hwp
```

## 3. 산출물

판정 파일 디렉터리:

```text
output/poc/hwpx2hwp/task993/stage2_mel001_table_contract_probe/
```

생성 파일:

| variant | file | graft 내용 | rhwp load |
|---|---|---|---|
| `01_current_generated` | `01_current_generated.hwp` | Stage 1 baseline | 20 pages |
| `02_budget_table_ctrl_common_attr_oracle` | `02_budget_table_ctrl_common_attr_oracle.hwp` | `예산 현황` 직후 표 `CTRL_HEADER(Table).common_attr` oracle 값 | 20 pages |
| `03_table_tail_oracle_all_changed_tables` | `03_table_tail_oracle_all_changed_tables.hwp` | changed `TABLE` tail 3건 전체 oracle 값 | 20 pages |
| `04_missing_ctrl_data_oracle` | `04_missing_ctrl_data_oracle.hwp` | oracle-only `CTRL_DATA` 1건 삽입 | 20 pages |
| `05_budget_common_attr_plus_table_tail` | `05_budget_common_attr_plus_table_tail.hwp` | `common_attr` + changed `TABLE` tail 전체 | 20 pages |
| `06_all_table_axes_plus_ctrl_data` | `06_all_table_axes_plus_ctrl_data.hwp` | `common_attr` + changed `TABLE` tail 전체 + `CTRL_DATA` | 20 pages |

## 4. 생성 방식

기존 진단 도구를 재사용했다.

```text
target/debug/rhwp hwp5-table-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/generated-mel-001.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage2_mel001_table_contract_probe_builtin

target/debug/rhwp hwp5-contract-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/generated-mel-001.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage2_mel001_contract_probe_builtin
```

`06_all_table_axes_plus_ctrl_data`는 `08_all_table_axes.hwp`를 다시 `hwp5-contract-probe`에
입력해 `CTRL_DATA`를 추가한 파일이다.

```text
target/debug/rhwp hwp5-contract-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage2_mel001_table_contract_probe_builtin/08_all_table_axes.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage2_mel001_all_table_axes_plus_contract_builtin
```

## 5. 도구 생성 로그

원본 도구 산출물:

```text
output/poc/hwpx2hwp/task993/stage2_mel001_table_contract_probe_builtin/stage9_generation.md
output/poc/hwpx2hwp/task993/stage2_mel001_contract_probe_builtin/stage11_generation.md
output/poc/hwpx2hwp/task993/stage2_mel001_all_table_axes_plus_contract_builtin/stage11_generation.md
```

중요한 카운트:

```text
ctrl_common_attr graft: 1건
table_tail graft: 3건
CTRL_DATA insert: 1건
```

주의:

```text
현재 hwp5-table-probe는 특정 TABLE 하나만 지정해 tail을 graft하지 않는다.
따라서 Stage 2의 03번 파일은 8x12 표 단일 tail이 아니라,
LCS로 매칭된 changed TABLE tail 3건 전체를 oracle 값으로 graft한 파일이다.
```

## 6. 해시

```text
2b78935a903ca0edcc329976fa450b1d6155bf683f4f0fa0a7149534462f10b7  01_current_generated.hwp
1d69572ac2f5daf7e565e7594f5d1a95c735662be101db33366f1b17b0d4c59a  02_budget_table_ctrl_common_attr_oracle.hwp
db65243642d23d06afc8e3992ca2879e4f711ba36fbe647695230af82e89ee37  03_table_tail_oracle_all_changed_tables.hwp
d830d27e88197696a30c4b8fc5d314b4eba28cc3273aa7ec218a5339b25a6cd1  04_missing_ctrl_data_oracle.hwp
f14caf5dd8e54b65406f8275f6411d126d3a391d4bd9455a39082c3310d66e09  05_budget_common_attr_plus_table_tail.hwp
7174f445963b30d88cb65ee09db2630f8db1a44a8e422c2f3708f7aa2b4d8e08  06_all_table_axes_plus_ctrl_data.hwp
```

## 7. 작업지시자 판정표

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 2페이지 중단 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `01_current_generated` | 파일손상 |  |  |  |  | baseline |
| `02_budget_table_ctrl_common_attr_oracle` | 열림 |  |  |  |  | common_attr 1건 |
| `03_table_tail_oracle_all_changed_tables` | 파일손상 |  |  |  |  | TABLE tail 3건 |
| `04_missing_ctrl_data_oracle` | 파일손상 |  |  |  |  | CTRL_DATA 1건 |
| `05_budget_common_attr_plus_table_tail` | 열림 |  |  |  |  | common_attr + TABLE tail |
| `06_all_table_axes_plus_ctrl_data` | 열림 |  |  |  |  | common_attr + TABLE tail + CTRL_DATA |

## 8. 판정 해석 기준

```text
- 02만 성공하면 예산 표 CTRL_HEADER common_attr bit 0x20000000이 핵심이다.
- 03만 성공하면 TABLE tail materialization이 핵심이다.
- 04만 성공하면 oracle-only CTRL_DATA 누락이 핵심이다.
- 05만 성공하면 common_attr와 TABLE tail 복합 조건이다.
- 06만 성공하면 table axes와 CTRL_DATA가 함께 필요하다.
- 06도 실패하면 budget table 주변의 LIST_HEADER/PARA_HEADER/셀 child tuple까지 더 세분화한다.
```

## 9. 판정 해석

작업지시자 판정 결과:

```text
파일손상: 01, 03, 04
열림: 02, 05, 06
```

결론:

```text
1. `02_budget_table_ctrl_common_attr_oracle` 단독으로 파일손상 판정이 사라졌다.
2. `03_table_tail_oracle_all_changed_tables`와 `04_missing_ctrl_data_oracle` 단독으로는
   파일손상 판정을 제거하지 못했다.
3. `05`, `06`이 열리는 이유는 두 파일 모두 `02`의 common_attr 보정을 포함하기 때문이다.
```

따라서 `mel-001`의 한컴 파일손상 원인은 다음 1건으로 확정한다.

```text
2페이지 "󰊳  예산 현황" 직후 12x5 표의 CTRL_HEADER(Table).common_attr

oracle:    0x282a2311
generated: 0x082a2311
missing:   0x20000000
```

## 10. 별도 분리할 문제

작업지시자가 지적한 2페이지 첫 1x1 표 셀 배경이 검게 출력되는 현상은 여전히 남아 있다.
다만 Stage 2 판정상 이 현상은 한컴 파일손상 판정을 직접 제거한 원인이 아니다.

현재 확인:

```text
- 2페이지 첫 1x1 표는 oracle/generated dump에서 모두 cell border_fill_id=7로 보인다.
- 그러나 실제 한컴 시각 판정에서 generated는 배경이 검게 출력된다.
- 따라서 이 문제는 파일손상 원인과 분리해 border fill payload/reference 또는 DocInfo
  BorderFill materialization 문제로 별도 추적해야 한다.
```

다음 단계는 두 갈래로 분리한다.

```text
1. P0 파일손상 수정: Table CTRL_HEADER common_attr 0x20000000 materialization 구현
2. P1 시각 fidelity 수정: 2페이지 첫 1x1 표의 셀 배경/테두리 BorderFill contract 비교
```
