# Task m100 #903 Stage 15

## 1. 단계 목적

Stage 14 판정에서 `02_chart_table_record_only`는 한컴 출력 경계를 다음 위치까지 밀었다.

```text
업종별 동향(억 달러, %) 까지 더 출력됨
```

하지만 같은 산출물에서 셀 안 텍스트 세로 배치가 위로 올라가는 부작용도 확인됐다.

따라서 Stage 15는 새 HWP variant를 만들지 않는다. 먼저 정답 HWP와 Stage 14 `02` 산출물의 문단/표 tuple 차이를 진단한다.

## 2. 비교 대상

정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 14 산출물:

```text
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/02_chart_table_record_only.hwp
```

비교 범위:

```text
문단 0:10  차트 표 host paragraph + table object
문단 0:11  차트 표 뒤 일반 문단
문단 0:12  빈 문단
문단 0:13  "업종별 동향" 제목 문단
문단 0:14  다음 4행×6열 표
```

## 3. 산출물

산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage15_chart_tuple_diff/
```

문단 dump:

```text
reference_p010_dump.txt
reference_p011_dump.txt
reference_p012_dump.txt
reference_p013_dump.txt
reference_p014_dump.txt
stage14_02_p010_dump.txt
stage14_02_p011_dump.txt
stage14_02_p012_dump.txt
stage14_02_p013_dump.txt
stage14_02_p014_dump.txt
```

IR diff:

```text
ir_diff_p010.txt
ir_diff_p011.txt
ir_diff_p012.txt
ir_diff_p013.txt
ir_diff_p014.txt
```

Raw record dump:

```text
reference_records.txt
stage14_02_records.txt
reference_records_090_210.txt
stage14_02_records_090_210.txt
reference_records_210_330.txt
stage14_02_records_210_330.txt
```

## 4. 1차 관찰

### 문단 종료 누락 가설

문단 `0:14`의 다음 표 control 자체는 양쪽 모두 존재한다.

```text
reference:
  --- 문단 0.14 --- cc=13, text_len=4, controls=1

stage14_02:
  --- 문단 0.14 --- cc=13, text_len=4, controls=1
```

따라서 "문단 종료 control 또는 표 control이 통째로 빠졌다"는 단독 가설은 약하다.

다만 Stage 14 `02`는 이전 문단과 표 payload가 정답과 다르게 축약되어 있어, 한컴 파서의 record 해석 cursor가 뒤에서 깨질 가능성은 남는다.

### 문단 0:13 텍스트 차이

정답 HWP:

```text
--- 문단 0.13 --- cc=20, text_len=19
텍스트: "< 업종별 동향(억 달러, %) >"
```

Stage 14 `02`:

```text
--- 문단 0.13 --- cc=18, text_len=17
텍스트: " 업종별 동향(억 달러, %) "
```

`<`, `>` 두 글자가 빠져 char count와 char shape position이 밀린다.

### 문단 0:14 다음 표 raw 차이

정답 HWP:

```text
[0] 표: 4행×6열, attr=0x0000000c
[raw] [10, 23, 2A, 08, 21, 01, 00, 00, F5, 01, 00, 00, 05, B5, 00, 00, 6C, 18, 00, 00]
```

Stage 14 `02`:

```text
[0] 표: 4행×6열, attr=0x00000004
[raw] [10, 03, 2A, 00, 21, 01, 00, 00, F5, 01, 00, 00, 05, B5, 00, 00, 6C, 18, 00, 00]
```

다음 표의 `TABLE` raw flag/tail 계열이 아직 compact/generated 상태다.

### 문단 0:10 차트 표 raw record 차이

정답 HWP raw record:

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46 ... 10 23 2a 08 ...
[103] TABLE            lv=2 sz=30
[104] LIST_HEADER      lv=2 sz=47
[105] PARA_HEADER      lv=2 sz=24
```

Stage 14 `02` raw record:

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46 ... 10 03 2a 00 ...
[103] TABLE            lv=2 sz=28
[104] LIST_HEADER      lv=2 sz=34
[105] PARA_HEADER      lv=2 sz=22
```

`02_chart_table_record_only`라는 이름과 달리 저장된 HWP raw record는 정답지의 `TABLE sz=30`까지 맞지 않는다. 즉 모델 레벨의 일부 속성은 반영됐더라도 serializer가 한컴 정답 raw tuple을 그대로 재현하지 못하고 있다.

## 5. 해석

Stage 14 `02`는 한컴 출력 위치를 뒤로 이동시킨 단서이지만, 정상 baseline은 아니다.

핵심은 다음 둘이다.

```text
1. 문단 0:10 차트 표의 host paragraph + table object tuple이 정답과 다르다.
2. 문단 0:13과 0:14에도 텍스트/표 raw 차이가 남아 있다.
```

따라서 "다음 표 자체가 첫 원인"이라고 단정하면 안 된다.

더 유력한 가설:

```text
차트 표를 부분 이식하면서 한컴 파서가 다음 문단까지는 진행하지만,
불완전한 TABLE/LIST_HEADER/PARA_HEADER tuple 때문에 다음 표 진입 시점에서 손상 판정을 낸다.
```

## 6. 다음 단계 제안

다음 실험은 `업종별 동향` 표를 바로 graft하는 방식이 아니라, 문단 `0:10` 차트 표의 원자적 tuple 정합성을 먼저 확인한다.

후보 variant:

```text
01_chart_host_para_raw_headers
02_chart_ctrl_table_raw_pair
03_chart_table_all_cell_headers_raw
04_chart_host_para_plus_chart_full_raw_tuple
05_chart_title_text_angle_brackets_only
06_chart_tuple_plus_following_title_text
```

판정 기준:

```text
- 한컴에서 "업종별 동향"까지 진행하는지
- 셀 안 텍스트 세로 배치가 위로 올라가는 부작용이 사라지는지
- 다음 표 진입 시점 파일손상이 계속되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

이 단계는 Stage 15 진단 결과를 승인받은 뒤 별도 Stage 16으로 진행한다.
