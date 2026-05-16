# Task m100 #903 Stage 42 작업 기록

## 1. 목적

Stage41에서 TABLE 축의 직접 원인이 `row_sizes` 오매핑임을 확인했다.
Stage42는 HWPX -> HWP adapter에서 table `row_sizes`를 행별 셀 수로 보정한다.

## 2. 구현 내용

수정 파일:

```text
src/document_core/converters/hwpx_to_hwp.rs
```

변경:

```text
adapt_table()에서 HWPX 출처 table(raw_ctrl_data.is_empty())이면
조건과 무관하게 materialize_table_record_row_sizes()를 호출한다.
```

의도:

```text
HWPX parser가 채운 "행별 셀 높이" row_sizes를
HWP TABLE record가 요구하는 "행별 셀 수" row_sizes로 교체한다.
```

하지 않은 것:

```text
- TABLE record n_zones=0 tail 구현
- SHAPE_COMPONENT / SHAPE_PICTURE payload 구현
- TABLE raw payload 전체 보존
- HWP 출처 문서의 table row_sizes 변경
```

## 3. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp
```

리포트:

```text
output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/table_payload_after_rowsizes.md
```

## 4. 내부 검증

실행:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage42_generate_table_rowsizes_probe -- --nocapture
```

결과:

```text
test task903_stage42_generate_table_rowsizes_probe ... ok
```

adapter report:

```text
tables_ctrl_data_synthesized: 26
table_record_row_sizes_materialized: 26
file_header_compression_normalized: 1
doc_properties_section_count_normalized: 1
section_def_controls_inserted: 2
```

산출물:

| file | bytes | sha256 | rhwp reload |
|---|---:|---|---|
| `output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp` | 374272 | `0f1ca5a42892f355aadba3328449a7709af267ff66a5c9d59583f7f7ad301106` | ok, pages=9 |

## 5. TABLE_MIN row_sizes 검증

필수 TABLE 9개가 모두 positive와 같은 row_sizes를 갖는다.

| idx | output row_sizes | positive row_sizes | match |
|---:|---|---|---|
| 48 | `[1]` | `[1]` | true |
| 103 | `[4, 9, 10, 10]` | `[4, 9, 10, 10]` | true |
| 286 | `[6, 6, 6, 6]` | `[6, 6, 6, 6]` | true |
| 433 | `[6, 6, 6, 6]` | `[6, 6, 6, 6]` | true |
| 563 | `[8, 8, 8, 8]` | `[8, 8, 8, 8]` | true |
| 742 | `[6, 6]` | `[6, 6]` | true |
| 1619 | `[4, 9, 10, 9, 10, 9, 10, 9, 10, 9, 10, 9]` | `[4, 9, 10, 9, 10, 9, 10, 9, 10, 9, 10, 9]` | true |
| 2944 | `[7, 7, 7]` | `[7, 7, 7]` | true |
| 6466 | `[7, 7, 7]` | `[7, 7, 7]` | true |

## 6. 남은 차이

row_sizes는 맞았지만 positive와 아직 완전히 같지는 않다.

주요 잔여 차이:

```text
1. TABLE attr
   output:   주로 0x00000004 / 0x00000005
   positive: 주로 0x0000000c / 0x0000000e

2. TABLE record 끝 n_zones=0 tail
   output:   없음
   positive: 00 00
```

Stage41/Stage40 기준으로는 `n_zones=0` tail만 빠진 optional TABLE은 한컴 성공을 깨지 않았다.
하지만 Stage42 산출물에서는 attr 차이와 SHAPE payload 차이도 남아 있으므로,
한컴 판정으로 다음 원인을 확인해야 한다.

## 7. 작업지시자 판정 요청

| 파일 | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp` | 읽기오류 | - | - | - | - | 비정상 렌더링 |  |

## 8. 판정 해석 기준

```text
한컴 성공:
  row_sizes 보정으로 TABLE 축이 해결됨.
  남은 attr/n_zones/SHAPE 차이는 현재 산출물에서는 허용 가능.

한컴 파일 손상 위치가 뒤로 이동:
  row_sizes 보정은 유효하며, 다음 원인은 TABLE attr/n_zones 또는 SHAPE payload.

한컴 파일 손상 위치가 그대로:
  row_sizes 외에도 필수 TABLE payload 차이가 남아 있음.
```

## 9. 판정 해석

Stage42 산출물은 한컴 에디터에서 `읽기오류` 판정을 받았고, rhwp-studio에서도
비정상 렌더링되었다.

이는 다음 의미다.

```text
1. Stage42 구현으로 필수 TABLE row_sizes는 positive와 일치했다.
2. 그러나 row_sizes 보정만으로는 clean adapter 산출물을 한컴 호환 HWP로 만들 수 없다.
3. Stage37~40에서 확인한 "SHAPE_COMPONENT + SHAPE_PICTURE + TABLE raw payload 조합"의
   나머지 축이 여전히 필요하다.
```

특히 Stage42 리포트 기준으로도 다음 차이가 남아 있다.

```text
1. TABLE attr:
   output   = 주로 0x00000004 / 0x00000005
   positive = 주로 0x0000000c / 0x0000000e

2. TABLE n_zones=0 tail:
   output   = 없음
   positive = 00 00

3. SHAPE_COMPONENT / SHAPE_PICTURE raw payload:
   Stage42에서는 구현하지 않음
```

Stage40에서 TABLE row_sizes 축은 이미 필수 index가 확인되었고, Stage42는 그중
`row_sizes`만 clean adapter에 반영한 실험이다. 따라서 Stage42 실패는 Stage30 분석을
뒤집는 결과가 아니라, Stage30 이후 남은 raw payload 축을 계속 구현해야 한다는 결과다.

## 10. 다음 단계 후보

판정 후:

```text
1. Stage42 row_sizes 구현은 유지한다.
2. Stage43에서는 Stage37/38의 결론에 따라 SHAPE_COMPONENT / SHAPE_PICTURE payload 축을
   먼저 clean adapter에 반영할 방법을 설계한다.
3. TABLE attr/n_zones=0 tail은 SHAPE payload 반영 후에도 한컴 판정이 남을 때 다음 축으로
   분리한다.
```
