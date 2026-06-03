# Task m100 #903 Stage 35 계획

## 1. 목적

Stage34에서 다음을 확인했다.

```text
- FileHeader는 동일하다.
- CFB stream 목록은 동일하다.
- BinData 실제 이미지 stream은 바이트 단위로 동일하다.
- 차이는 DocInfo와 BodyText record payload에 있다.
```

Stage35는 우선 DocInfo 쪽 차이를 분리한다.
BodyText raw-tail 문제로 바로 넓히지 않고, 이미지 렌더링 실패와 직접 연결되는 `HWPTAG_BIN_DATA`,
그리고 Stage30 정상화 축이었던 `HWPTAG_PARA_SHAPE`만 먼저 확인한다.

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

## 3. Variant 설계

| variant | 적용 내용 | 목적 |
|---|---|---|
| 01_bin_data_model_fields_only | positive의 BinData 모델 필드만 복사, `raw_data=None` | HWPX parser/adapter에서 구현 가능한 BinData metadata 보정으로 충분한지 확인 |
| 02_bin_data_raw_records_only | positive의 `HWPTAG_BIN_DATA` raw record payload 복사 | BinData serializer가 아직 부족한지 확인 |
| 03_para_shape_model_fields_no_raw | positive의 ParaShape 모델 필드 복사, `raw_data=None` | Stage30 ParaShape 정상화가 모델 필드 재직렬화만으로 재현되는지 확인 |
| 04_para_shape_raw_records_only | positive의 ParaShape raw record payload 복사 | ParaShape serializer 또는 미모델링 필드 부족 여부 확인 |
| 05_bin_data_model_plus_para_shape_model | 01 + 03 | 구현 가능한 DocInfo 보정 조합 확인 |
| 06_bin_data_raw_plus_para_shape_raw | 02 + 04 | DocInfo raw payload가 한컴 판정을 바꾸는지 확인 |

## 4. 판정표

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_bin_data_model_fields_only |  |  |  |  |  |  |  |
| 02_bin_data_raw_records_only |  |  |  |  |  |  |  |
| 03_para_shape_model_fields_no_raw |  |  |  |  |  |  |  |
| 04_para_shape_raw_records_only |  |  |  |  |  |  |  |
| 05_bin_data_model_plus_para_shape_model |  |  |  |  |  |  |  |
| 06_bin_data_raw_plus_para_shape_raw |  |  |  |  |  |  |  |

## 5. 기대 해석

```text
01에서 이미지/rhwp-studio가 회복:
  HWPX BinData metadata parser/adapter 보정이 필요하다.

02만 회복:
  BinData serializer가 positive raw record와 동등한 payload를 못 만든다.

03 또는 05에서 표/셀 배치가 회복:
  Stage30의 ParaShape 결론은 유효하며, 현재 Stage31 구현이 아직 필드 매핑을 충분히 못 했다.

04만 회복:
  ParaShape serializer 또는 미모델링 raw 필드가 필요하다.

06도 한컴 파일 읽기 오류가 유지:
  DocInfo만으로 부족하며 BodyText CTRL_HEADER/LIST_HEADER/PARA_HEADER raw-tail 축으로 이동한다.
```

## 6. 하지 않을 것

```text
- BodyText raw-tail graft를 Stage35에 섞지 않는다.
- serializer 구현 변경을 먼저 하지 않는다.
- Stage34 결과를 무시하고 대량 probe로 돌아가지 않는다.
```

