# Task m100 #903 Stage 35 작업 기록

## 1. 목적

Stage34에서 한컴 파일 읽기 오류의 남은 차이가 CFB stream 누락이 아니라
`DocInfo`와 `BodyText` record payload 차이임을 확인했다.

Stage35는 그중 `DocInfo` 축만 먼저 분리한다.

확인 대상:

```text
1. HWPTAG_BIN_DATA metadata
2. HWPTAG_PARA_SHAPE payload
```

## 2. 기준 파일

Positive:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Failing baseline:

```text
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/
```

## 3. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/01_bin_data_model_fields_only.hwp
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/02_bin_data_raw_records_only.hwp
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/03_para_shape_model_fields_no_raw.hwp
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/04_para_shape_raw_records_only.hwp
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/05_bin_data_model_plus_para_shape_model.hwp
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/06_bin_data_raw_plus_para_shape_raw.hwp
```

## 4. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage35_generate_docinfo_payload_probe_variants -- --nocapture
```

결과:

```text
test task903_stage35_generate_docinfo_payload_probe_variants ... ok
```

모든 파일은 rhwp 재로드 기준 9페이지다.

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_bin_data_model_fields_only | 374272 | ok, pages=9 |
| 02_bin_data_raw_records_only | 374272 | ok, pages=9 |
| 03_para_shape_model_fields_no_raw | 374272 | ok, pages=9 |
| 04_para_shape_raw_records_only | 374272 | ok, pages=9 |
| 05_bin_data_model_plus_para_shape_model | 374272 | ok, pages=9 |
| 06_bin_data_raw_plus_para_shape_raw | 374272 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_bin_data_model_fields_only | `e411578415e43f3cc64a9b4296ff77c771d330b55c107c2ce35f36ecd372fd6c` |
| 02_bin_data_raw_records_only | `e411578415e43f3cc64a9b4296ff77c771d330b55c107c2ce35f36ecd372fd6c` |
| 03_para_shape_model_fields_no_raw | `26f070169b5b690df07235d7e85273835eb76755a30dfb194193e14c35e21e89` |
| 04_para_shape_raw_records_only | `26f070169b5b690df07235d7e85273835eb76755a30dfb194193e14c35e21e89` |
| 05_bin_data_model_plus_para_shape_model | `3c3d33ab370a3a2629e0fd2bce1065a71ea14d3b18cb068253f52896a3a6b4ee` |
| 06_bin_data_raw_plus_para_shape_raw | `3c3d33ab370a3a2629e0fd2bce1065a71ea14d3b18cb068253f52896a3a6b4ee` |

관찰:

```text
01 == 02
03 == 04
05 == 06
```

즉 현재 serializer 기준에서는 positive의 모델 필드만 복사해도 positive raw record payload와 동일한 바이트를 만든다.
DocInfo의 `BIN_DATA`와 `PARA_SHAPE`에 한해서는 raw_data 복사가 별도 효과를 만들지 않았다.

대표 variant IR 비교:

```text
target/debug/rhwp ir-diff \
  output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/05_bin_data_model_plus_para_shape_model.hwp \
  output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp \
  --summary
```

결과:

```text
=== 비교 완료: 차이 0 건 ===
```

## 5. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_bin_data_model_fields_only | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 버그 |
| 02_bin_data_raw_records_only | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 버그 |
| 03_para_shape_model_fields_no_raw | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 하지 않음 |
| 04_para_shape_raw_records_only | 파일 읽기 오류 | 실패 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 하지 않음 |
| 05_bin_data_model_plus_para_shape_model | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 버그 |
| 06_bin_data_raw_plus_para_shape_raw | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 버그 |

추가 관찰:

```text
- 1페이지 표 안의 이미지 2개 중 1개만 렌더링된다.
- 2페이지 이미지 개체 묶기가 처리되지 않는다.
```

판정 포인트:

```text
- 01/02에서 rhwp-studio 이미지 렌더링이 회복되는지
- 03/04에서 표/셀 배치가 회복되는지
- 05/06에서 한컴 파일 읽기 오류가 사라지는지
```

## 6. 현재 해석

Stage35가 한컴 파일 읽기 오류를 해결하지 못하면,
DocInfo의 `BIN_DATA`와 `PARA_SHAPE`만으로는 부족하다는 뜻이다.

그 경우 Stage34에서 관찰된 BodyText record payload 차이로 이동한다.

우선 후보:

```text
- CTRL_HEADER 47/246 bytes vs 28/46 bytes
- LIST_HEADER 65/47 bytes vs 34 bytes
- PARA_HEADER 24 bytes vs 22 bytes
```

## 7. 판정 해석

Stage35 판정으로 다음을 확정한다.

```text
1. DocInfo BIN_DATA payload만 보정해도 한컴 파일 읽기 오류는 사라지지 않는다.
2. DocInfo ParaShape payload만 보정해도 한컴 파일 읽기 오류는 사라지지 않는다.
3. BIN_DATA + ParaShape를 함께 보정해도 한컴 파일 읽기 오류는 사라지지 않는다.
4. 따라서 한컴 파일 읽기 오류의 직접 원인은 DocInfo 단독이 아니라 BodyText record payload 차이에 있다.
```

rhwp-studio 이미지 렌더링은 `BIN_DATA` 보정 여부에 영향을 받는다.

```text
BIN_DATA 보정 있음: 1페이지 표 안 이미지 2개 중 1개만 렌더링
BIN_DATA 보정 없음: 이미지 렌더링 하지 않음
```

따라서 이미지 문제는 두 층으로 나뉜다.

```text
1. DocInfo BIN_DATA metadata: 일부 이미지 참조 회복에 필요
2. BodyText shape/group picture payload: 남은 이미지/묶음 개체 렌더링에 필요
```

다음 단계는 BodyText record payload를 좁혀야 한다.

Stage34에서 이미 확인한 우선 후보:

```text
1. CTRL_HEADER 확장 payload
   - positive: 47/246 bytes
   - failing: 28/46 bytes

2. LIST_HEADER 확장 payload
   - positive: 65/47 bytes
   - failing: 34 bytes

3. PARA_HEADER extra payload
   - positive: 24 bytes
   - failing: 22 bytes
```

Stage36은 이 세 축을 분리한다.
