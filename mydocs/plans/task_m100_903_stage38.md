# Task m100 #903 Stage 38 계획

## 1. 목적

Stage37에서 한컴 성공 조건이 다음 세 record군 조합으로 좁혀졌다.

```text
SHAPE_COMPONENT + SHAPE_PICTURE + TABLE
```

Stage38은 이 조합을 구현 가능한 단위로 다시 분해한다.

목표:

```text
1. TABLE 13개 diff 중 한컴 성공에 필요한 최소 index 범위를 찾는다.
2. SHAPE_COMPONENT/SHAPE_PICTURE 11개 diff를 그림 개체 단위로 묶는다.
3. TABLE payload와 SHAPE payload 중 어떤 조합이 한컴 성공의 최소 조건인지 확인한다.
```

## 2. 기준 파일

실패 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

성공 baseline:

```text
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/07_shape_picture_table.hwp
```

Positive raw source:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Diff inventory:

```text
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/stage37_section0_diff.md
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage38_record_index_probe/
```

## 3. Stage37에서 확인한 diff index

Stage37 diff record는 24개다.

```text
SHAPE_COMPONENT: 21, 35, 807, 808, 810, 812
SHAPE_PICTURE:   22, 36, 809, 811, 813
TABLE:           48, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596, 6986, 7376
```

## 4. Variant 설계

### 4.1 TABLE 범위 분해

| variant | 적용 내용 | 목적 |
|---|---|---|
| 01_table_first_record_only | TABLE idx 48 | 첫 표 TABLE data diff 영향 확인 |
| 02_table_early_size_diffs | TABLE idx 48,103,286,433,563,742,819 | 1~2페이지 주요 표 구간 확인 |
| 03_table_late_size_diffs | TABLE idx 1619,2944,6466,6596,6986,7376 | 후반부 표 구간 확인 |
| 04_table_all_indices | TABLE diff 13개 전체 | Stage37 03 재확인 |

### 4.2 그림 record 묶음 분해

| variant | 적용 내용 | 목적 |
|---|---|---|
| 05_first_page_picture_pair | SHAPE idx 21,22,35,36 | 1페이지 표 안 이미지 2개 후보 |
| 06_group_picture_cluster | SHAPE idx 807,808,809,810,811,812,813 | 2페이지 이미지 개체 묶기 후보 |
| 07_shape_all_indices | SHAPE diff 11개 전체 | Stage37 06 재확인 |

### 4.3 조합 확인

| variant | 적용 내용 | 목적 |
|---|---|---|
| 08_first_picture_plus_table_all | 05 + TABLE 전체 | 첫 페이지 그림 + TABLE로 충분한지 확인 |
| 09_group_picture_plus_table_all | 06 + TABLE 전체 | 2페이지 묶음 그림 + TABLE로 충분한지 확인 |
| 10_shape_all_plus_table_early | SHAPE 전체 + early TABLE | 앞쪽 TABLE만으로 충분한지 확인 |
| 11_shape_all_plus_table_late | SHAPE 전체 + late TABLE | 후반 TABLE만으로 충분한지 확인 |
| 12_shape_all_plus_table_all | SHAPE 전체 + TABLE 전체 | Stage37 07 재현 |

## 5. 판정표

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_table_first_record_only |  |  |  |  |  |  |  |
| 02_table_early_size_diffs |  |  |  |  |  |  |  |
| 03_table_late_size_diffs |  |  |  |  |  |  |  |
| 04_table_all_indices |  |  |  |  |  |  |  |
| 05_first_page_picture_pair |  |  |  |  |  |  |  |
| 06_group_picture_cluster |  |  |  |  |  |  |  |
| 07_shape_all_indices |  |  |  |  |  |  |  |
| 08_first_picture_plus_table_all |  |  |  |  |  |  |  |
| 09_group_picture_plus_table_all |  |  |  |  |  |  |  |
| 10_shape_all_plus_table_early |  |  |  |  |  |  |  |
| 11_shape_all_plus_table_late |  |  |  |  |  |  |  |
| 12_shape_all_plus_table_all |  |  |  |  |  |  |  |

## 6. 기대 해석

```text
04 실패, 12 성공:
  TABLE만으로는 불충분하고 SHAPE payload와 결합해야 한다.

07 파일손상, 12 성공:
  SHAPE만으로는 이미지 일부를 회복하지만 문서 유효성에는 TABLE payload가 필요하다.

10 성공:
  앞쪽 TABLE diff가 핵심이다.

11 성공:
  후반 TABLE diff가 핵심이다.

10/11 모두 실패, 12 성공:
  TABLE diff 전체 또는 특정 분산 조합이 필요하다.

08 또는 09만 성공:
  특정 그림 cluster와 TABLE의 결합이 핵심이다.
```

## 7. 하지 않을 것

```text
- 아직 TABLE/SHAPE raw graft를 구현 해법으로 확정하지 않는다.
- HWPX 저장 전체에 Section raw 보존을 적용하지 않는다.
- rhwp-studio 렌더링 성공만으로 한컴 호환 성공을 판단하지 않는다.
```
