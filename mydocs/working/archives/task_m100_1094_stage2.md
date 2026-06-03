# Task M100-1094 Stage 2 작업 기록

## 1. 단계 목표

Stage 1에서 확인한 `TABLE.table_attr` 상위 비트 차이가 한컴 에디터의 표 높이/페이지 배치 차이를
만드는지 판정하기 위해, 정답지 `samples/aift.hwp`의 TABLE attr를 현재 생성 HWP에 부분 투영한
판정용 HWP를 만든다.

## 2. 산출물

판정용 파일 위치:

```text
output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/
```

생성 파일:

| file | 설명 |
|---|---|
| `01_current_baseline.hwp` | Stage 1 현재 생성본 |
| `02_section0_table_attr_only.hwp` | Section 0 TABLE attr만 정답지에서 투영 |
| `03_section1_table_attr_only.hwp` | Section 1 TABLE attr만 정답지에서 투영 |
| `04_section0_1_table_attr_only.hwp` | Section 0 + Section 1 TABLE attr를 정답지에서 투영 |
| `05_section0_table_attr_plus_section1_all_axes.hwp` | Section 0 TABLE attr + Section 1 TABLE 관련 전체 축 투영 |

중간 생성 위치:

```text
output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section0/
output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section1/
output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section0_1/
```

## 3. 실행 방식

기존 `hwp5-table-probe`는 전체 문서 LCS 매칭으로 실행하면 `aift` 샘플에서는 지나치게 오래 걸린다.
따라서 Stage 1에서 대상 TABLE record가 이미 Section 0/1로 고정된 상태를 이용해 section 단위로
나누어 probe를 생성했다.

실행한 핵심 명령:

```text
cargo run --quiet --bin rhwp -- hwp5-table-probe \
  samples/aift.hwp \
  output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/current_aift.hwp \
  --out-dir output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section0 \
  --section 0

cargo run --quiet --bin rhwp -- hwp5-table-probe \
  samples/aift.hwp \
  output/poc/hwpx2hwp/task1094/stage1_current_table_attr_trace/current_aift.hwp \
  --out-dir output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section1 \
  --section 1

cargo run --quiet --bin rhwp -- hwp5-table-probe \
  samples/aift.hwp \
  output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section0/02_table_attr_only.hwp \
  --out-dir output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_probe_section0_1 \
  --section 1
```

## 4. 적용된 TABLE attr

`04_section0_1_table_attr_only.hwp` 기준으로 Section 0/1의 TABLE attr는 정답지와 같다.

```text
Section 0 table 1: 0x04000006
Section 0 table 2: 0x0600000e
Section 1 table 1: 0x0400000e
Section 1 table 2: 0x06000004
```

`rhwp info` 기준:

```text
file = output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/04_section0_1_table_attr_only.hwp
size = 4,605,952 bytes
sections = 3
pages = 76
reload = ok
```

## 5. 판정표

| file | 한컴 판정 유형 | 메모 표시 | 2페이지 표/셀 배치 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/01_current_baseline.hwp` |  |  |  |  |  | 현재 baseline |
| `output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/02_section0_table_attr_only.hwp` |  |  |  |  |  | Section 0 TABLE attr only |
| `output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/03_section1_table_attr_only.hwp` |  |  |  |  |  | Section 1 TABLE attr only |
| `output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/04_section0_1_table_attr_only.hwp` |  |  |  |  |  | 핵심 후보 |
| `output/poc/hwpx2hwp/task1094/stage2_table_attr_high_bits_review/05_section0_table_attr_plus_section1_all_axes.hwp` |  |  |  |  |  | TABLE attr 외 축 확인용 |

## 6. 해석 기준

이번 단계의 핵심 판정은 다음이다.

```text
04_section0_1_table_attr_only.hwp가 한컴 에디터에서 정답지처럼 2페이지 표를 한 페이지 안에 배치하면,
Stage 1에서 좁힌 TABLE attr 상위 비트가 직접 원인이다.
```

반대로 `04`가 실패하고 `05`가 성공하면, TABLE attr 외에 table/ctrl header 주변 축이 추가로 필요하다.
둘 다 실패하면 Stage 1에서 보이지 않은 별도 조판 contract를 다시 찾아야 한다.

## 7. 다음 단계

작업지시자 시각 판정 후:

```text
1. 성공 후보의 TABLE attr 규칙을 adapter에 최소 구현한다.
2. low attr만으로 0x04000000/0x06000000을 결정할 수 없으므로,
   HWPX source 속성 또는 table shape/class 기준으로 분기 근거를 추가 추적한다.
3. #1092 메모 저장 결과가 유지되는지 guard로 확인한다.
```

## 8. 작업지시자 판정 보강

작업지시자 확인 결과, 정답 HWP와 생성 HWP의 UI 차이는 다음으로 확인되었다.

```text
정답 HWP: 셀 안쪽 여백 지정값이 활성화됨
생성 HWP: 셀 안쪽 여백 지정값이 비활성화됨
```

Stage 1에서 확인한 것처럼 `in_margin_left/right/top/bottom` 수치 자체는 이미 저장되어 있다.
따라서 이번 차이는 안쪽 여백 값의 누락이 아니라, 한컴 에디터가 해당 값을 "사용"으로 해석하게 하는
TABLE record attr 상위 비트 누락으로 본다.

Stage 3에서는 우선 정답지에서 모든 대상 표에 공통으로 존재한 `0x04000000`을 "셀 안쪽 여백 지정
활성" 후보로 실제 adapter에 반영한다.
