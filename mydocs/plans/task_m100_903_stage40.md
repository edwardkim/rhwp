# Task m100 #903 Stage 40 계획

## 1. 목적

Stage39에서 `SHAPE_ALL + TABLE 누적` 결과가 다음처럼 정리되었다.

```text
idx 6596까지 누적하면 한컴 성공
idx 6986, 7376까지 더 넣어도 성공 유지
idx 819 제외도 성공
idx 48 제외는 실패
idx 1619 제외는 실패
```

Stage40은 성공 후보 TABLE index에서 하나씩 제거하여 최소 TABLE set을 확정한다.

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

TABLE 기준 set은 Stage39에서 성공한 `06_shape_all_table_11`이다.

```text
48,103,286,433,563,742,819,1619,2944,6466,6596
```

추가 검산 set:

```text
48,103,286,433,563,742,1619,2944,6466,6596
```

위 set은 Stage39에서 불필요 후보로 나온 `819`를 뺀 것이다.

## 4. Variant 설계

### 4.1 기준 재현

| variant | 적용 TABLE index | 목적 |
|---|---|---|
| 01_base_table_11 | `48,103,286,433,563,742,819,1619,2944,6466,6596` | Stage39 06 성공 재현 |
| 02_base_without_819 | `48,103,286,433,563,742,1619,2944,6466,6596` | 819 제외 성공 재검산 |

### 4.2 leave-one-out

`02_base_without_819` 기준으로 하나씩 뺀다.

| variant | 제외 index | 목적 |
|---|---:|---|
| 03_without_48 | 48 | 필수 여부 재확인 |
| 04_without_103 | 103 | 필수 여부 확인 |
| 05_without_286 | 286 | 필수 여부 확인 |
| 06_without_433 | 433 | 필수 여부 확인 |
| 07_without_563 | 563 | 필수 여부 확인 |
| 08_without_742 | 742 | 필수 여부 확인 |
| 09_without_1619 | 1619 | 필수 여부 재확인 |
| 10_without_2944 | 2944 | 필수 여부 확인 |
| 11_without_6466 | 6466 | 필수 여부 확인 |
| 12_without_6596 | 6596 | 필수 여부 확인 |

### 4.3 optional tail 검산

| variant | 적용 TABLE index | 목적 |
|---|---|---|
| 13_base_without_819_plus_6986 | 02 + `6986` | 6986 추가 영향 없음 재확인 |
| 14_base_without_819_plus_7376 | 02 + `7376` | 7376 추가 영향 없음 재확인 |
| 15_base_without_819_plus_6986_7376 | 02 + `6986,7376` | tail optional 전체 검산 |

## 5. 판정표

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_base_table_11 |  |  |  |  |  |  |  |
| 02_base_without_819 |  |  |  |  |  |  |  |
| 03_without_48 |  |  |  |  |  |  |  |
| 04_without_103 |  |  |  |  |  |  |  |
| 05_without_286 |  |  |  |  |  |  |  |
| 06_without_433 |  |  |  |  |  |  |  |
| 07_without_563 |  |  |  |  |  |  |  |
| 08_without_742 |  |  |  |  |  |  |  |
| 09_without_1619 |  |  |  |  |  |  |  |
| 10_without_2944 |  |  |  |  |  |  |  |
| 11_without_6466 |  |  |  |  |  |  |  |
| 12_without_6596 |  |  |  |  |  |  |  |
| 13_base_without_819_plus_6986 |  |  |  |  |  |  |  |
| 14_base_without_819_plus_7376 |  |  |  |  |  |  |  |
| 15_base_without_819_plus_6986_7376 |  |  |  |  |  |  |  |

## 6. 기대 해석

```text
02 성공:
  819는 최소 조건에서 제외한다.

leave-one-out 실패:
  제외한 index는 필수 후보로 남긴다.

leave-one-out 성공:
  제외한 index는 최소 조건에서 제외 가능하다.

13~15 성공:
  6986/7376은 tail optional로 확정한다.
```

## 7. 다음 단계

Stage40 결과로 TABLE 최소 index set을 확정한 뒤,
Stage41에서 해당 TABLE payload의 byte diff를 필드 의미로 분석한다.

특히 TABLE diff 대부분은 다음 패턴이다.

```text
target size = positive size - 2
```

따라서 Stage41의 핵심 질문은 다음이다.

```text
HWP TABLE record 끝에 필요한 2 bytes tail이 무엇인가?
그리고 어떤 TABLE에는 data byte만 다른 이유가 무엇인가?
```
