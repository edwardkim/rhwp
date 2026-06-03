# Task m100 #903 Stage 42 계획

## 1. 목적

Stage41에서 TABLE payload의 직접 원인이 확인되었다.

```text
HWP TABLE record row_sizes는 행별 셀 수여야 한다.
현재 HWPX parser는 row_sizes를 행별 셀 높이 최대값으로 채운다.
```

Stage42는 TABLE 축의 최소 구현을 수행한다.

## 2. 구현 범위

### 2.1 할 것

```text
HWPX -> HWP adapter에서 모든 HWPX table의 row_sizes를 행별 셀 수로 보정한다.
```

대상 함수:

```text
src/document_core/converters/hwpx_to_hwp.rs
  adapt_table()
  materialize_table_record_row_sizes()
```

현재 `materialize_table_record_row_sizes()`는 이미 행별 셀 수를 계산한다.
문제는 이 함수가 특정 조건에서만 호출된다는 점이다.

변경 방향:

```text
adapt_table()에서 HWPX 출처 table이면 조건과 무관하게
materialize_table_record_row_sizes()를 호출한다.
```

### 2.2 하지 않을 것

```text
- TABLE record n_zones=0 tail 구현
- SHAPE_COMPONENT / SHAPE_PICTURE payload 구현
- TABLE raw payload 전체 보존
- HWP 출처 문서의 table row_sizes 변경
```

## 3. 근거

Stage41 분석:

```text
필수 TABLE index:
48,103,286,433,563,742,1619,2944,6466

필수 TABLE target row_sizes:
셀 높이 또는 잘못된 큰 값

필수 TABLE positive row_sizes:
행별 셀 수
```

제외 가능 TABLE:

```text
819,6596,6986,7376

row_sizes는 이미 positive와 일치
차이는 n_zones=0 tail뿐
한컴 성공에는 영향 없음
```

## 4. 구현 후 검증

### 4.1 자동 검증

추가/수정할 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage42_generate_table_rowsizes_probe -- --nocapture
```

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp
```

리포트:

```text
output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/table_payload_after_rowsizes.md
```

검증 내용:

```text
1. TABLE_MIN 필수 index의 row_sizes가 positive와 일치하는지
2. rhwp 재로드 기준 9페이지인지
3. 파일 크기가 정상 범위인지
```

### 4.2 작업지시자 판정

| 파일 | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp` |  |  |  |  |  |  |  |

## 5. 기대 해석

```text
한컴 성공:
  TABLE row_sizes 보정만으로 TABLE 축은 해결.
  남은 SHAPE payload도 현재 adapter 산출물에서 충분했음을 의미.

한컴 파일 손상 위치가 뒤로 이동:
  TABLE row_sizes 축은 해결됐고, 다음 원인은 SHAPE payload 또는 n_zones tail.

한컴 파일 손상 위치가 그대로:
  row_sizes 보정 적용 범위가 부족하거나 serializer가 아직 positive TABLE payload를 재현하지 못함.
```

## 6. 후속 후보

Stage42 판정 후:

```text
1. 성공하면 wasm/rhwp-studio 통합 검증 단계로 이동.
2. 실패하면 Stage43에서 SHAPE_COMPONENT/SHAPE_PICTURE payload를 byte-level 분석.
3. TABLE만 일부 실패하면 n_zones=0 tail 구현 여부를 별도 검증.
```
