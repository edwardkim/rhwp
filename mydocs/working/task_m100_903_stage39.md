# Task m100 #903 Stage 39 작업 기록

## 1. 목적

Stage38에서 성공 조건은 다음으로 재확인되었다.

```text
SHAPE_ALL + TABLE_ALL
```

Stage39는 `SHAPE_ALL`을 고정한 뒤 TABLE 13개 diff를 누적/삭제 방식으로 더 좁힌다.

## 2. 기준 파일

실패 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
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

모든 variant는 다음 SHAPE index를 포함한다.

```text
21,22,35,36,807,808,809,810,811,812,813
```

TABLE index 순서:

```text
48,103,286,433,563,742,819,1619,2944,6466,6596,6986,7376
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/01_shape_all_table_1.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/02_shape_all_table_3.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/03_shape_all_table_5.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/04_shape_all_table_7.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/05_shape_all_table_9.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/06_shape_all_table_11.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/07_shape_all_table_12.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/08_shape_all_table_13.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/09_all_except_48.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/10_all_except_819.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/11_all_except_1619.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/12_all_except_7376.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/13_all_except_late.hwp
output/poc/hwpx2hwp/task903/stage39_table_index_min_probe/14_all_except_early.hwp
```

## 5. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage39_generate_table_index_min_probe_variants -- --nocapture
```

결과:

```text
test task903_stage39_generate_table_index_min_probe_variants ... ok
```

모든 파일은 rhwp 재로드 기준 9페이지다.

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_shape_all_table_1 | 375808 | ok, pages=9 |
| 02_shape_all_table_3 | 375808 | ok, pages=9 |
| 03_shape_all_table_5 | 375808 | ok, pages=9 |
| 04_shape_all_table_7 | 375808 | ok, pages=9 |
| 05_shape_all_table_9 | 375808 | ok, pages=9 |
| 06_shape_all_table_11 | 375808 | ok, pages=9 |
| 07_shape_all_table_12 | 375808 | ok, pages=9 |
| 08_shape_all_table_13 | 375808 | ok, pages=9 |
| 09_all_except_48 | 375808 | ok, pages=9 |
| 10_all_except_819 | 375808 | ok, pages=9 |
| 11_all_except_1619 | 375808 | ok, pages=9 |
| 12_all_except_7376 | 375808 | ok, pages=9 |
| 13_all_except_late | 375808 | ok, pages=9 |
| 14_all_except_early | 375808 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_shape_all_table_1 | `2ddba69b47a30f486e64e51d19e5bf75fd681e1ba6deeeee8dd305155b2d4b7b` |
| 02_shape_all_table_3 | `ac8e360e9c06892b20aa711d8aa0a19287a900bedb60450287021d24710f08b4` |
| 03_shape_all_table_5 | `9cc99e0b2d3eec03c4a95bd121f3d57d24dda294ecf920e02fc8b3f975c69857` |
| 04_shape_all_table_7 | `b25866ee7aacede0c80388dc72d9db802b6891689f9879cf9b439bcecd7a6b16` |
| 05_shape_all_table_9 | `626dd82a4dd109049d3c5ae5c71f86327d5a810ccad9fc084a15bc498449d6e2` |
| 06_shape_all_table_11 | `cf519c219bafe31c097ef2b57f038a51990342150158c78a9944f97b07c4f394` |
| 07_shape_all_table_12 | `ffe8201717acc8da6a918ebafdc0fc4a48e1aa059efeab56397bea367fa676af` |
| 08_shape_all_table_13 | `fd330c684d81e4966b16e8c9f81b32549bd79da490abc68feee9947f385d7434` |
| 09_all_except_48 | `82be47ac941fa86adfb77c68fb0eefc11c43b787f08174e3f911c40fe062348a` |
| 10_all_except_819 | `9cf94727c93dfb0358c618c1f62aa5996fb19ee01ecfcf48357e8b795e68f2ca` |
| 11_all_except_1619 | `289eee3e52eda7320f8f2506927c39ccb9c38bff467b62fa6feafcf3454cb90d` |
| 12_all_except_7376 | `ffe8201717acc8da6a918ebafdc0fc4a48e1aa059efeab56397bea367fa676af` |
| 13_all_except_late | `b25866ee7aacede0c80388dc72d9db802b6891689f9879cf9b439bcecd7a6b16` |
| 14_all_except_early | `f81f7d706dfba48af99c0ed2fcc11295923729433dbdc47c1c5acb75c3c5c1da` |

검산:

```text
04_shape_all_table_7 == Stage38 10_shape_all_plus_table_early
08_shape_all_table_13 == Stage38 12_shape_all_plus_table_all
12_all_except_7376 == 07_shape_all_table_12
13_all_except_late == 04_shape_all_table_7
14_all_except_early == Stage38 11_shape_all_plus_table_late
```

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_shape_all_table_1 | 파일 손상 | 1페이지 `< 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) >` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 02_shape_all_table_3 | 파일 손상 | 1페이지 `< 국가별 동향(억 달러, %) >` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 03_shape_all_table_5 | 파일 손상 | 2페이지 두 번째 표 이전까지 | 실패 | 실패 | 실패 | 성공 |  |
| 04_shape_all_table_7 | 파일 손상 | 3페이지 `□ 국가별 동향(상위 5개)` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 05_shape_all_table_9 | 파일 손상 | 7페이지 `2. 연도별 동향` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 06_shape_all_table_11 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 07_shape_all_table_12 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 08_shape_all_table_13 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 09_all_except_48 | 파일 손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 |  |
| 10_all_except_819 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 11_all_except_1619 | 파일 손상 | 3페이지 `□ 국가별 동향(상위 5개)` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 12_all_except_7376 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 13_all_except_late | 파일 손상 | 3페이지 `□ 국가별 동향(상위 5개)` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 14_all_except_early | 파일 손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 |  |

## 7. 판정 해석 기준

```text
01~08 누적 variant:
  한컴 출력 위치가 어느 TABLE index 추가 시점에 전진하는지 본다.

08만 성공:
  TABLE 13개 전체가 필요할 가능성이 높다.

09~12 leave-one-out:
  하나라도 성공하면 제외한 TABLE index는 최소 조건에서 빠질 수 있다.
  실패하면 해당 TABLE index가 성공 조건에 필요할 가능성이 높다.

13/14:
  Stage38 early/late 결과를 재현하는 검산 파일이다.
```

## 8. 판정 해석

Stage39 판정으로 TABLE index의 영향이 문서 진행 순서와 거의 대응한다.

누적 variant:

```text
01: idx 48까지        -> 1페이지 첫 차트/표 제목까지
02: idx 286까지       -> 1페이지 국가별 동향까지
03: idx 563까지       -> 2페이지 두 번째 표 이전까지
04: idx 819까지       -> 3페이지 국가별 동향(상위 5개)까지
05: idx 2944까지      -> 7페이지 연도별 동향까지
06: idx 6596까지      -> 성공
07: idx 6986까지      -> 성공
08: idx 7376까지      -> 성공
```

따라서 마지막 성공에 필요한 TABLE diff는 `6596` 이전에 이미 충족된다.
`6986`, `7376`은 성공 조건에서 빠질 가능성이 높다.

leave-one-out variant:

```text
09_all_except_48: 실패
  -> idx 48은 필수 후보

10_all_except_819: 성공
  -> idx 819는 필수 조건이 아니다

11_all_except_1619: 실패
  -> idx 1619는 필수 후보

12_all_except_7376: 성공
  -> idx 7376는 필수 조건이 아니다
```

현재 성공 최소 후보:

```text
SHAPE_ALL 고정

TABLE 필수 후보:
48,103,286,433,563,742,1619,2944,6466,6596

TABLE 선택/불필요 후보:
819,6986,7376
```

주의:

```text
103,286,433,563,742,2944,6466,6596은 아직 개별 leave-one-out 검증이 없다.
따라서 위 목록은 "최소 확정"이 아니라 "다음 검증 후보"다.
```

## 9. 다음 단계

판정 후 다음 중 하나로 진행한다.

```text
Stage40:
  SHAPE_ALL + TABLE 성공 후보에서 TABLE index별 leave-one-out을 수행한다.

목표:
  - 103,286,433,563,742,2944,6466,6596의 필수 여부 확인
  - 819,6986,7376 제외 상태에서도 성공이 유지되는지 재검산
  - TABLE 구현에 필요한 최소 payload index set 확정
```
