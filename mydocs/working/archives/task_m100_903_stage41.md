# Task m100 #903 Stage 41 작업 기록

## 1. 목적

Stage40에서 TABLE 최소 후보가 다음으로 확정되었다.

```text
필수 TABLE index:
48,103,286,433,563,742,1619,2944,6466

제외 가능 TABLE index:
819,6596,6986,7376
```

Stage41은 이 TABLE record payload를 byte-level로 비교해 구현 후보를 찾는다.

## 2. 기준 파일

실패 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

성공 raw source:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

분석 리포트:

```text
output/poc/hwpx2hwp/task903/stage41_table_payload_diff/table_payload_diff.md
```

## 3. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage41_generate_table_payload_diff_report -- --nocapture
```

결과:

```text
test task903_stage41_generate_table_payload_diff_report ... ok
```

## 4. 리포트 요약

필수 TABLE과 선택 TABLE의 차이가 분명히 갈린다.

### 4.1 필수 TABLE

필수 TABLE은 대부분 다음 패턴이다.

```text
target row_sizes: 셀 높이 또는 잘못된 큰 값
positive row_sizes: 행별 셀 수
positive payload 끝: n_zones = 0 (00 00)
```

예:

```text
idx 103
target:
  rows=4, cols=10
  row_sizes=[1280,1280,1280,1280]

positive:
  rows=4, cols=10
  row_sizes=[4,9,10,10]
  n_zones=0
```

```text
idx 1619
target:
  rows=12, cols=10
  row_sizes=[1759,1759,2042,2083,2042,2042,2042,2042,2042,2042,2042,2042]

positive:
  rows=12, cols=10
  row_sizes=[4,9,10,9,10,9,10,9,10,9,10,9]
  n_zones=0
```

idx 48은 size는 같고 data만 다르다.

```text
idx 48
target row_sizes=[6155]
positive row_sizes=[1]
```

payload offset 18은 TABLE record 구조상 첫 `row_sizes` 항목이다.
따라서 idx 48도 같은 축의 문제다.

### 4.2 제외 가능 TABLE

제외 가능 TABLE은 row_sizes가 이미 positive와 일치한다.
차이는 끝의 `n_zones=0` 2 bytes뿐이다.

예:

```text
idx 6596
target row_sizes=[7,7,6,7,6,7,6,7,6,7,6]
positive row_sizes=[7,7,6,7,6,7,6,7,6,7,6]
positive extra tail=00 00
```

따라서 Stage40에서 제외 가능했던 이유는 분명하다.

```text
row_sizes가 맞으면 n_zones=0 누락만으로는 한컴 성공을 깨지 않는다.
row_sizes가 틀리면 해당 TABLE에서 한컴 파일 손상이 발생한다.
```

## 5. 코드 대조

HWP TABLE parser/serializer의 현재 구조:

```text
src/parser/control.rs
  parse_table_record()
  - row_sizes를 "행별 셀 수"로 읽음
  - 남은 2 bytes가 있으면 n_zones로 읽음

src/serializer/control.rs
  serialize_table_record()
  - table.row_sizes를 그대로 기록
  - table.raw_table_record_extra가 있을 때만 border_fill 이후 추가 바이트 기록
```

HWPX parser의 현재 구조:

```text
src/parser/hwpx/section.rs
  parse_table()
  - row_sizes를 행별 셀 높이 최대값으로 설정
```

문제 지점:

```text
HWP TABLE record의 row_sizes는 "행별 셀 수"인데,
HWPX parser는 이를 "행별 셀 높이 최대값"으로 채우고 있다.
```

adapter에는 이미 `materialize_table_record_row_sizes()`가 있다.

```text
src/document_core/converters/hwpx_to_hwp.rs
  materialize_table_record_row_sizes()
  - 각 행의 cell 개수를 세어 row_sizes로 설정
```

하지만 이 보정은 현재 특정 table 조건에서만 호출된다.
Stage41 결과상 `hwpx-h-01.hwpx`의 필수 TABLE 일부에는 이 보정이 적용되지 않아 한컴 호환성이 깨진다.

## 6. 현재 결론

TABLE 쪽 직접 원인은 `row_sizes` 오매핑이다.

```text
원인:
  HWPX table row_sizes가 셀 높이로 들어감

한컴 기대:
  HWP TABLE record row_sizes = 행별 셀 수

부가 차이:
  positive는 border_fill 뒤에 n_zones=0 (00 00)을 기록하는 경우가 많다.
  그러나 row_sizes가 맞는 optional TABLE에서는 이 2 bytes가 없어도 한컴 성공이 유지된다.
```

따라서 구현 우선순위:

```text
1. HWPX -> HWP 저장 어댑터에서 모든 HWPX table의 row_sizes를 행별 셀 수로 보정한다.
2. n_zones=0 tail은 별도 보수적 개선 후보로 남긴다.
```

## 7. 남은 SHAPE 축

Stage37~40에서 성공 조건은 여전히 다음이었다.

```text
SHAPE_ALL + TABLE_MIN
```

Stage41은 TABLE 축만 분석했다.
따라서 TABLE row_sizes 보정만으로 전체 성공이 되는지는 아직 단정하지 않는다.
다음 단계에서 TABLE 보정 구현 후,
남은 한컴 오류가 SHAPE payload 때문인지 확인해야 한다.

## 8. 다음 단계

Stage42 계획:

```text
1. HWPX -> HWP adapter에서 table.row_sizes를 모든 HWPX table에 대해 행별 셀 수로 보정한다.
2. n_zones=0 tail은 우선 구현하지 않는다.
3. samples/hwpx/hwpx-h-01.hwpx 변환 산출물을 만들어 한컴 판정을 요청한다.
4. 실패하면 SHAPE payload 축으로 이동한다.
```
