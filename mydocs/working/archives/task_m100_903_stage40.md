# Task m100 #903 Stage 40 작업 기록

## 1. 목적

Stage39에서 성공 후보 TABLE index가 다음으로 좁혀졌다.

```text
48,103,286,433,563,742,819,1619,2944,6466,6596
```

또한 `819`는 제외해도 성공하는 것으로 관찰되었다.
Stage40은 `819`를 제외한 성공 후보에서 TABLE index를 하나씩 제거하여 최소 set을 확인한다.

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
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/
```

## 3. 고정 조건

모든 variant는 `SHAPE_ALL`을 적용한다.

```text
21,22,35,36,807,808,809,810,811,812,813
```

기준 TABLE set:

```text
BASE_TABLE_11:
48,103,286,433,563,742,819,1619,2944,6466,6596

BASE_WITHOUT_819:
48,103,286,433,563,742,1619,2944,6466,6596
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/01_base_table_11.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/02_base_without_819.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/03_without_48.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/04_without_103.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/05_without_286.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/06_without_433.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/07_without_563.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/08_without_742.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/09_without_1619.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/10_without_2944.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/11_without_6466.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/13_base_without_819_plus_6986.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/14_base_without_819_plus_7376.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/15_base_without_819_plus_6986_7376.hwp
```

## 5. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage40_generate_table_min_leave_one_out_variants -- --nocapture
```

결과:

```text
test task903_stage40_generate_table_min_leave_one_out_variants ... ok
```

모든 파일은 rhwp 재로드 기준 9페이지다.

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_base_table_11 | 375808 | ok, pages=9 |
| 02_base_without_819 | 375808 | ok, pages=9 |
| 03_without_48 | 375808 | ok, pages=9 |
| 04_without_103 | 375808 | ok, pages=9 |
| 05_without_286 | 375808 | ok, pages=9 |
| 06_without_433 | 375808 | ok, pages=9 |
| 07_without_563 | 375808 | ok, pages=9 |
| 08_without_742 | 375808 | ok, pages=9 |
| 09_without_1619 | 375808 | ok, pages=9 |
| 10_without_2944 | 375808 | ok, pages=9 |
| 11_without_6466 | 375808 | ok, pages=9 |
| 12_without_6596 | 375808 | ok, pages=9 |
| 13_base_without_819_plus_6986 | 375808 | ok, pages=9 |
| 14_base_without_819_plus_7376 | 375808 | ok, pages=9 |
| 15_base_without_819_plus_6986_7376 | 375808 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_base_table_11 | `cf519c219bafe31c097ef2b57f038a51990342150158c78a9944f97b07c4f394` |
| 02_base_without_819 | `df61a5106e22bc1f4db104957225910d659934dfd39bf6dd2c50a2191f6e6198` |
| 03_without_48 | `210090e15a66fd76f424913ad5031dd9b13280e9544277eab02b8e40e4ecbc23` |
| 04_without_103 | `4c44147ac11fed49630b0069460e91724f3d31f02a7f99a6b2df16190f856e32` |
| 05_without_286 | `ce8f6b4d6a676f1c9bbcfef0dbc035a3cc3e20f46bb1468882e46da2be2525d0` |
| 06_without_433 | `78a302cc3695ea5278b67c2d9373e3386ef86235366c75b07056370508c2f17e` |
| 07_without_563 | `7f70066cf1c1e314b65b66907a8fb230e6e966bd1f7177c7b87ceadce2feaa0d` |
| 08_without_742 | `4b05eae2a6524c56ac83c2b331da8157dae17b92419d69e29edb5346ffac1af4` |
| 09_without_1619 | `a548b40da0e704fa358616712dfcf3fe9cb99fa671b1e0800086aadfb5edb3ef` |
| 10_without_2944 | `9d540e6d9515978d230ba5a60245625238fe8bba25b9527ddb352fe037072e5c` |
| 11_without_6466 | `e0a8500f4869b2cd213c975103b1434d228916acd3372495172d1bee8465aab5` |
| 12_without_6596 | `e7a954d46fa4bd50d48e0d99b8d61564f393bc0d7384ae40811edf6d92b48320` |
| 13_base_without_819_plus_6986 | `2483536c3be2e2d61c44f32d5e95f6c2f311ef78a1d295afedbddbd1a20f5b45` |
| 14_base_without_819_plus_7376 | `014d520ef2bcb026df9e93ef8a0459233819d33ce8e688d3917b7e9ca87089fc` |
| 15_base_without_819_plus_6986_7376 | `9cf94727c93dfb0358c618c1f62aa5996fb19ee01ecfcf48357e8b795e68f2ca` |

검산:

```text
01_base_table_11 == Stage39 06_shape_all_table_11
15_base_without_819_plus_6986_7376 == Stage39 10_all_except_819
```

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_base_table_11 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 02_base_without_819 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 03_without_48 | 파일 손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 |  |
| 04_without_103 | 파일 손상 | 1페이지 `< 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) >` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 05_without_286 | 파일 손상 | 1페이지 `< 업종별 동향(억 달러, %) >` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 06_without_433 | 파일 손상 | 1페이지 `< 국가별 동향(억 달러, %) >` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 07_without_563 | 파일 손상 | 2페이지 `< 지역별 동향(억 달러, %) >` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 08_without_742 | 파일 손상 | 2페이지 두 번째 표 이전까지 | 실패 | 실패 | 실패 | 성공 |  |
| 09_without_1619 | 파일 손상 | 3페이지 `□ 국가별 동향(상위 5개)` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 10_without_2944 | 파일 손상 | 4페이지 `2. 연도별 동향` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 11_without_6466 | 파일 손상 | 7페이지 `2. 연도별 동향` 까지 | 실패 | 실패 | 실패 | 성공 |  |
| 12_without_6596 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 13_base_without_819_plus_6986 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 14_base_without_819_plus_7376 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 15_base_without_819_plus_6986_7376 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |

## 7. 판정 해석 기준

```text
02 성공:
  819는 최소 조건에서 제외한다.

03~12 중 성공:
  해당 index는 최소 성공 조건에서 제외 가능하다.

03~12 중 실패:
  해당 index는 필수 후보로 남는다.

13~15 성공:
  6986/7376은 optional tail로 본다.
```

## 8. 판정 해석

Stage40 판정으로 TABLE 최소 후보가 확정되었다.

성공 유지:

```text
01_base_table_11
02_base_without_819
12_without_6596
13_base_without_819_plus_6986
14_base_without_819_plus_7376
15_base_without_819_plus_6986_7376
```

제외 가능 TABLE index:

```text
819
6596
6986
7376
```

제외하면 실패하는 TABLE index:

```text
48
103
286
433
563
742
1619
2944
6466
```

최소 성공 후보:

```text
SHAPE_ALL:
21,22,35,36,807,808,809,810,811,812,813

TABLE_MIN:
48,103,286,433,563,742,1619,2944,6466
```

중요한 관찰:

```text
1. TABLE index 하나가 빠질 때마다 한컴 출력 중단 위치가 해당 표/문단 주변으로 후퇴한다.
2. 이는 TABLE payload diff가 단일 전역 설정이 아니라 개별 TABLE record의 구조 유효성 문제임을 뜻한다.
3. 819,6596,6986,7376은 TABLE diff가 존재하지만 한컴 성공 최소 조건에는 필요하지 않다.
4. rhwp-studio는 모든 variant를 읽지만, 한컴은 필수 TABLE record가 하나라도 빠지면 파일 손상으로 중단한다.
```

Stage40 이후 구현 후보는 다음처럼 좁혀진다.

```text
1. SHAPE payload는 이미지/묶음 개체 호환성에 필요하다.
2. TABLE payload는 개별 table record별 한컴 유효성에 필요하다.
3. TABLE 쪽은 필수 index 9개에 공통적으로 빠진 tail/field를 찾아 serializer에 반영해야 한다.
```

## 9. 다음 단계 후보

Stage40 판정 후 다음으로 이동한다.

```text
Stage41:
  TABLE_MIN 9개와 제외 가능 TABLE 4개의 byte-level diff를 비교한다.

목표:
  - 필수 TABLE과 optional TABLE의 차이 패턴 확인
  - TABLE record 끝의 2 bytes tail 의미 추적
  - idx 48처럼 size는 같고 data만 다른 TABLE record의 의미 분리
```
