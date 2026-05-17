# Task m100 #903 Stage 38 작업 기록

## 1. 목적

Stage37에서 한컴 성공 조건이 다음 세 record군 조합으로 좁혀졌다.

```text
SHAPE_COMPONENT + SHAPE_PICTURE + TABLE
```

Stage38은 이를 record index 단위로 분해한다.

확인 목표:

```text
1. TABLE 13개 diff 중 앞쪽/뒤쪽 어느 범위가 필요한지 확인한다.
2. SHAPE_COMPONENT/SHAPE_PICTURE 11개 diff를 1페이지 이미지와 2페이지 묶음 이미지 cluster로 분리한다.
3. TABLE payload와 SHAPE payload의 최소 조합을 확인한다.
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

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage38_record_index_probe/
```

## 3. Record index map

생성된 index map:

```text
output/poc/hwpx2hwp/task903/stage38_record_index_probe/stage38_record_index_map.md
```

Stage37에서 확인한 24개 diff record:

| group | indices |
|---|---|
| 1페이지 이미지 후보 | `21,22,35,36` |
| 2페이지 묶음 이미지 후보 | `807,808,809,810,811,812,813` |
| TABLE early | `48,103,286,433,563,742,819` |
| TABLE late | `1619,2944,6466,6596,6986,7376` |

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage38_record_index_probe/01_table_first_record_only.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/02_table_early_size_diffs.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/03_table_late_size_diffs.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/04_table_all_indices.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/05_first_page_picture_pair.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/06_group_picture_cluster.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/07_shape_all_indices.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/08_first_picture_plus_table_all.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/09_group_picture_plus_table_all.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/10_shape_all_plus_table_early.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/11_shape_all_plus_table_late.hwp
output/poc/hwpx2hwp/task903/stage38_record_index_probe/12_shape_all_plus_table_all.hwp
```

## 5. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage38_generate_record_index_probe_variants -- --nocapture
```

결과:

```text
test task903_stage38_generate_record_index_probe_variants ... ok
```

모든 파일은 rhwp 재로드 기준 9페이지다.

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_table_first_record_only | 375808 | ok, pages=9 |
| 02_table_early_size_diffs | 375808 | ok, pages=9 |
| 03_table_late_size_diffs | 375808 | ok, pages=9 |
| 04_table_all_indices | 375808 | ok, pages=9 |
| 05_first_page_picture_pair | 375808 | ok, pages=9 |
| 06_group_picture_cluster | 375808 | ok, pages=9 |
| 07_shape_all_indices | 375808 | ok, pages=9 |
| 08_first_picture_plus_table_all | 375808 | ok, pages=9 |
| 09_group_picture_plus_table_all | 375808 | ok, pages=9 |
| 10_shape_all_plus_table_early | 375808 | ok, pages=9 |
| 11_shape_all_plus_table_late | 375808 | ok, pages=9 |
| 12_shape_all_plus_table_all | 375808 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_table_first_record_only | `df1c9fcd08353000c9b067a3822e512d4b1159faa18ae2ec6e00d2122fbd534c` |
| 02_table_early_size_diffs | `812d3969fb67453a454f0497e09acd79ad433237094f237973efe36a680a394d` |
| 03_table_late_size_diffs | `cf449d6a0749cf2eec423057af7a1dcca2d0bda44baf2422571a27b968114803` |
| 04_table_all_indices | `44fecf122e25aa35fa2a61922e746dc3d74694b8a7d89b49f7299e937bc9dc07` |
| 05_first_page_picture_pair | `fd7a4514d9d8870d6f20efbe03dae4e1d2a76c8a4056b3a48d3d85e6283589fa` |
| 06_group_picture_cluster | `81244fc40e3e6a310992d5ebd3ec8da6bf07aa381df86d3ae7e6f5ee27eaee55` |
| 07_shape_all_indices | `d991123d54295510fe3831aa5d4c86afe50f8a5848954f18bc211a079df78f38` |
| 08_first_picture_plus_table_all | `63bf70f011ed2f4b533a041f3e6af21daae381d65d32e855461529e2e5cbfd2c` |
| 09_group_picture_plus_table_all | `4853f172a7fe7c45c5e0f3fbcf9a486e4a17e867820db3bc55577a69869733c4` |
| 10_shape_all_plus_table_early | `b25866ee7aacede0c80388dc72d9db802b6891689f9879cf9b439bcecd7a6b16` |
| 11_shape_all_plus_table_late | `f81f7d706dfba48af99c0ed2fcc11295923729433dbdc47c1c5acb75c3c5c1da` |
| 12_shape_all_plus_table_all | `fd330c684d81e4966b16e8c9f81b32549bd79da490abc68feee9947f385d7434` |

관찰:

```text
04_table_all_indices == Stage37 03_table_records
07_shape_all_indices == Stage37 06_shape_component_picture
12_shape_all_plus_table_all == Stage37 07_shape_picture_table
```

따라서 Stage38 variant 구성은 Stage37 결과를 index 단위로 올바르게 분해하고 있다.

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_table_first_record_only | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 02_table_early_size_diffs | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 03_table_late_size_diffs | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 04_table_all_indices | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 05_first_page_picture_pair | 파일 손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 06_group_picture_cluster | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 성공 |
| 07_shape_all_indices | 파일 손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 성공 |
| 08_first_picture_plus_table_all | 파일 읽기 오류 | 2페이지 이미지 묶기 전까지 출력 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 09_group_picture_plus_table_all | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 성공 |
| 10_shape_all_plus_table_early | 파일 손상 | 3페이지 □ 국가별 동향(상위 5개) 까지 출력 | 실패 | 실패 | 실패 | 성공 |  |
| 11_shape_all_plus_table_late | 파일 손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 |  |
| 12_shape_all_plus_table_all | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |

## 7. 판정 해석 기준

```text
04 실패, 12 성공:
  TABLE 전체만으로는 부족하고 SHAPE payload가 함께 필요하다.

07 파일손상, 12 성공:
  SHAPE 전체만으로는 부족하고 TABLE payload가 함께 필요하다.

08 성공:
  1페이지 이미지 후보 + TABLE 전체가 최소 조건일 가능성이 높다.

09 성공:
  2페이지 묶음 이미지 후보 + TABLE 전체가 최소 조건일 가능성이 높다.

10 성공:
  앞쪽 TABLE diff만으로 충분하다.

11 성공:
  후반 TABLE diff만으로 충분하다.

10/11 모두 실패, 12 성공:
  TABLE diff가 분산되어 있고 전체 또는 더 정교한 조합이 필요하다.
```

## 8. 다음 단계 후보

## 8. 판정 해석

Stage38 판정으로 다음을 확인했다.

```text
1. TABLE payload만으로는 한컴 파일 읽기 오류가 해결되지 않는다.
   - 01~04 모두 파일 읽기 오류

2. SHAPE payload만으로도 충분하지 않다.
   - 05/07은 첫 테이블까지 진행하지만 파일 손상

3. 첫 페이지 그림 cluster와 2페이지 묶음 그림 cluster는 각각 다른 효과를 가진다.
   - 05: 1페이지 이미지 2개 출력
   - 06: rhwp-studio 2페이지 이미지 개체 묶기 렌더링 성공
   - 07: 둘 다 포함되어 rhwp-studio 이미지 렌더링은 개선되지만 한컴은 여전히 파일 손상

4. TABLE payload는 한 번에 전체가 필요하다.
   - 10(SHAPE 전체 + early TABLE): 3페이지까지 진행
   - 11(SHAPE 전체 + late TABLE): 1페이지 첫 테이블에서 중지
   - 12(SHAPE 전체 + all TABLE): 성공

5. 성공 조건은 Stage37과 동일하게 재확인되었다.
   - SHAPE 전체 + TABLE 전체
```

핵심 해석:

```text
한컴 성공은 단일 record index나 단일 구간의 문제가 아니라,
문서 내 여러 TABLE payload diff가 누적되어야 한다.

SHAPE payload는 이미지와 그림 개체 구조 회복에 필요하고,
TABLE payload는 문서 진행과 표/셀 유효성 회복에 필요하다.
```

특히 `10_shape_all_plus_table_early`가 3페이지까지 진행한 반면,
`11_shape_all_plus_table_late`는 1페이지 첫 테이블에서 중지했다.
따라서 앞쪽 TABLE diff는 초반 진행에 필수이고,
후반 TABLE diff는 마지막 성공까지 가기 위해 추가로 필요하다.

## 9. 다음 단계

Stage39는 `SHAPE_ALL`을 고정한 상태에서 TABLE 13개를 문서 진행 순서대로 누적/삭제 방식으로 더 좁힌다.

목표:

```text
1. TABLE diff 13개가 모두 필요한지 확인한다.
2. 어느 TABLE index를 빼면 한컴 출력 위치가 어디까지 후퇴하는지 확인한다.
3. TABLE payload size diff의 공통 구조를 구현 후보로 연결한다.
```
