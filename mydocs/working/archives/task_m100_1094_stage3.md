# Task M100-1094 Stage 3 작업 기록

## 1. 단계 목표

Stage 2 시각 판정에서 확인된 UI 차이:

```text
정답 HWP: 셀 안쪽 여백 지정값 활성
생성 HWP: 셀 안쪽 여백 지정값 비활성
```

이 차이를 해결하기 위해 HWPX table `inMargin` 값이 있는 표에 HWP5 TABLE attr bit 26
(`0x04000000`)을 materialize한다.

## 2. 구현 내용

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
tests/hwpx_to_hwp_adapter.rs
```

구현 규칙:

```text
table.padding.left/right/top/bottom 중 하나라도 0이 아니면
TABLE raw_table_record_attr에 0x04000000을 추가한다.
```

의도:

```text
1. TABLE record 안쪽 여백 수치(in_margin)는 이미 저장된다.
2. 하지만 한컴 에디터의 "셀 안쪽 여백 지정" 활성 상태는 별도 attr bit가 필요하다.
3. 정답지 aift.hwp에서 관련 TABLE 모두 0x04000000을 포함한다.
```

## 3. 생성 파일

```text
output/poc/hwpx2hwp/task1094/stage3_table_inner_margin_attr/aift-table-inner-margin-attr.hwp
```

`rhwp info` 결과:

```text
sections = 3
pages = 76
reload = ok
size = 4,605,952 bytes
```

## 4. TABLE attr 비교

Stage 3 후보와 정답지의 Section 0/1 대상 TABLE attr:

| 위치 | oracle | Stage 3 generated | 상태 |
|---|---:|---:|---|
| Section 0 table 1 | `0x04000006` | `0x04000006` | 일치 |
| Section 0 table 2 | `0x0600000e` | `0x0400000e` | bit 25만 남음 |
| Section 1 table 1 | `0x0400000e` | `0x0400000e` | 일치 |
| Section 1 table 2 | `0x06000004` | `0x04000004` | bit 25만 남음 |

중요:

```text
0x04000000은 작업지시자가 확인한 "셀 안쪽 여백 지정 활성" 차이를 해결하는 후보다.
0x02000000은 여전히 정답지에만 남아 있으나, 현재 단계에서는 원인/의미를 단정하지 않는다.
```

## 5. 판정표

| file | 한컴 판정 유형 | 셀 안쪽 여백 지정 | 메모 표시 | 2페이지 표/셀 배치 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1094/stage3_table_inner_margin_attr/aift-table-inner-margin-attr.hwp` |  |  |  |  |  |  | bit 26 materialized |

## 6. 실행한 검증

```text
cargo test --test hwpx_to_hwp_adapter task888_basic_table_materializes_hancom_table_attrs
cargo test document_core::converters::hwpx_to_hwp::tests::adapter_materializes_table_contract_fields
cargo check
target/debug/rhwp info output/poc/hwpx2hwp/task1094/stage3_table_inner_margin_attr/aift-table-inner-margin-attr.hwp
```

결과:

```text
success
```

## 7. 다음 판정 기준

작업지시자 시각 판정에서 확인할 항목:

```text
1. 한컴 에디터의 셀 안쪽 여백 지정값이 활성화되는가
2. 2페이지 표/셀 높이와 페이지 배치가 정답지에 가까워지는가
3. #1092 메모 표시가 유지되는가
```

만약 안쪽 여백 활성은 해결되지만 2페이지 표 높이가 여전히 다르면, 남은 `0x02000000` 축을 별도
Stage로 분리한다.
