# Task m100 #903 Stage 39 계획

## 1. 목적

Stage38에서 성공 조건은 다음으로 재확인되었다.

```text
SHAPE_ALL + TABLE_ALL
```

하지만 `TABLE_ALL` 안에는 13개 TABLE record diff가 있다.
Stage39는 `SHAPE_ALL`을 고정한 뒤 TABLE diff를 문서 진행 순서대로 더 좁힌다.

목표:

```text
1. TABLE 13개가 모두 필요한지 확인한다.
2. TABLE diff를 누적했을 때 한컴 출력 위치가 어떻게 전진하는지 확인한다.
3. TABLE diff 중 특정 index 하나를 빼도 성공하는지 확인한다.
```

## 2. 기준 파일

실패 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

성공 baseline:

```text
output/poc/hwpx2hwp/task903/stage38_record_index_probe/12_shape_all_plus_table_all.hwp
```

Positive raw source:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/
```

## 3. 고정 조건

모든 variant는 다음 SHAPE index를 적용한다.

```text
SHAPE_ALL:
21,22,35,36,807,808,809,810,811,812,813
```

TABLE index는 다음 순서로 다룬다.

```text
48,103,286,433,563,742,819,1619,2944,6466,6596,6986,7376
```

## 4. Variant 설계

### 4.1 TABLE 누적 확인

| variant | 적용 TABLE index | 목적 |
|---|---|---|
| 01_shape_all_table_1 | `48` | 첫 TABLE data diff만 적용 |
| 02_shape_all_table_3 | `48,103,286` | 초기 TABLE size diff 일부 적용 |
| 03_shape_all_table_5 | `48,103,286,433,563` | 초기 주요 TABLE 중간 누적 |
| 04_shape_all_table_7 | `48,103,286,433,563,742,819` | Stage38 early TABLE 재현 |
| 05_shape_all_table_9 | early + `1619,2944` | 3페이지 이후 진행 여부 확인 |
| 06_shape_all_table_11 | 05 + `6466,6596` | 후반 TABLE 누적 확인 |
| 07_shape_all_table_12 | 06 + `6986` | 마지막 직전 TABLE 누적 확인 |
| 08_shape_all_table_13 | 전체 13개 | Stage38 12 성공 재현 |

### 4.2 TABLE leave-one-out 확인

성공 파일에서 일부 TABLE index를 하나 또는 구간으로 뺀다.

| variant | 제외 TABLE index | 목적 |
|---|---|---|
| 09_all_except_48 | `48` | 첫 TABLE data diff가 필수인지 확인 |
| 10_all_except_819 | `819` | early 구간 마지막 TABLE이 필수인지 확인 |
| 11_all_except_1619 | `1619` | late 구간 첫 TABLE이 필수인지 확인 |
| 12_all_except_7376 | `7376` | 마지막 TABLE diff가 필수인지 확인 |
| 13_all_except_late | `1619,2944,6466,6596,6986,7376` | Stage38 early 재현 |
| 14_all_except_early | `48,103,286,433,563,742,819` | Stage38 late 재현 |

## 5. 판정표

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_shape_all_table_1 |  |  |  |  |  |  |  |
| 02_shape_all_table_3 |  |  |  |  |  |  |  |
| 03_shape_all_table_5 |  |  |  |  |  |  |  |
| 04_shape_all_table_7 |  |  |  |  |  |  |  |
| 05_shape_all_table_9 |  |  |  |  |  |  |  |
| 06_shape_all_table_11 |  |  |  |  |  |  |  |
| 07_shape_all_table_12 |  |  |  |  |  |  |  |
| 08_shape_all_table_13 |  |  |  |  |  |  |  |
| 09_all_except_48 |  |  |  |  |  |  |  |
| 10_all_except_819 |  |  |  |  |  |  |  |
| 11_all_except_1619 |  |  |  |  |  |  |  |
| 12_all_except_7376 |  |  |  |  |  |  |  |
| 13_all_except_late |  |  |  |  |  |  |  |
| 14_all_except_early |  |  |  |  |  |  |  |

## 6. 기대 해석

```text
누적 variant가 특정 지점에서 성공:
  그 지점 이후 TABLE index는 한컴 유효성에는 불필요할 수 있다.

08만 성공:
  TABLE 13개 전체가 필요하거나, 마지막 index까지 누적되어야 한다.

leave-one-out variant가 성공:
  제외한 TABLE index는 성공 최소 조건에서 빠질 수 있다.

leave-one-out variant가 실패:
  제외한 TABLE index는 성공 조건에 포함될 가능성이 높다.

13/14는 Stage38 10/11 재현:
  Stage39 테스트 구성이 Stage38 결과와 일치하는지 검산한다.
```

## 7. 하지 않을 것

```text
- 아직 TABLE record raw graft를 최종 구현으로 확정하지 않는다.
- 13개 TABLE payload를 한 번에 저장하는 식으로 바로 구현하지 않는다.
- Stage39 판정 전에는 serializer 필드 매핑을 시작하지 않는다.
```
