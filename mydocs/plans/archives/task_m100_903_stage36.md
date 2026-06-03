# Task m100 #903 Stage 36 계획

## 1. 목적

Stage35 결과, DocInfo의 `BIN_DATA`/`PARA_SHAPE` 보정만으로는 한컴 파일 읽기 오류가 사라지지 않았다.

Stage36은 Stage34에서 확인한 BodyText record payload 차이를 분리한다.

우선 후보:

```text
1. CTRL_HEADER 확장 payload
2. LIST_HEADER 확장 payload
3. PARA_HEADER extra payload
```

## 2. 기준 파일

Positive:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Failing baseline:

```text
output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/05_bin_data_model_plus_para_shape_model.hwp
```

이 baseline은 DocInfo의 `BIN_DATA`와 `PARA_SHAPE`를 보정한 상태다.
따라서 Stage36 결과는 BodyText payload 축으로 해석한다.

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/
```

## 3. Variant 설계

Stage36은 raw BodyText 전체를 graft하지 않는다.
먼저 record 타입 단위로 positive payload를 이식한다.

| variant | 적용 내용 | 목적 |
|---|---|---|
| 01_ctrl_header_records | BodyText에서 `CTRL_HEADER` record payload만 positive에서 복사 | 컨트롤 헤더 확장 payload가 한컴 읽기 오류의 직접 원인인지 확인 |
| 02_list_header_records | `LIST_HEADER` record payload만 positive에서 복사 | 셀/텍스트박스/그룹 list header 확장 payload 영향 확인 |
| 03_para_header_records | `PARA_HEADER` record payload만 positive에서 복사 | 문단 header extra payload 영향 확인 |
| 04_ctrl_plus_list_headers | 01 + 02 | 개체/셀 container payload 조합 확인 |
| 05_ctrl_list_para_headers | 01 + 02 + 03 | BodyText header-class payload 조합 확인 |
| 06_section0_raw_only | BodyText/Section0 raw stream을 positive에서 복사 | Section0 BodyText 전체가 필요하면 한컴 판정이 움직이는지 확인 |
| 07_section0_section1_raw | BodyText/Section0, Section1 raw stream을 positive에서 복사 | BodyText 전체 positive graft의 상한선 확인 |

## 4. 판정표

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_ctrl_header_records |  |  |  |  |  |  |  |
| 02_list_header_records |  |  |  |  |  |  |  |
| 03_para_header_records |  |  |  |  |  |  |  |
| 04_ctrl_plus_list_headers |  |  |  |  |  |  |  |
| 05_ctrl_list_para_headers |  |  |  |  |  |  |  |
| 06_section0_raw_only |  |  |  |  |  |  |  |
| 07_section0_section1_raw |  |  |  |  |  |  |  |

## 5. 기대 해석

```text
01만으로 한컴 판정이 움직임:
  CTRL_HEADER 확장 payload가 핵심.

02만으로 표/셀/이미지 배치가 움직임:
  LIST_HEADER 확장 payload가 핵심.

03만으로 한컴 판정이 움직임:
  PARA_HEADER extra payload가 핵심.

04/05에서만 회복:
  단일 record 타입이 아니라 header-class payload 조합 문제.

06/07에서만 회복:
  개별 record 타입 graft로 부족하며, 더 깊은 BodyText payload 차이를 비교해야 한다.
```

## 6. 하지 않을 것

```text
- 구현 확정
- serializer 수정
- DocInfo 탐색 반복
```

