# Task m100 #903 Stage 37 계획

## 1. 목적

Stage36에서 다음이 확인되었다.

```text
05_ctrl_list_para_headers:
  - positive와 IR diff 0건
  - 한컴 파일 읽기 오류

06_section0_raw_only:
  - positive와 IR diff 0건
  - 한컴 성공
```

따라서 원인은 Section0 BodyText raw stream 안에 있으며,
`CTRL_HEADER/LIST_HEADER/PARA_HEADER` 타입 단위 graft만으로는 부족하다.

Stage37은 `05`와 `06`의 차이를 record 타입 단위로 더 좁힌다.

## 2. 기준 파일

성공 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/06_section0_raw_only.hwp
```

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
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/
```

## 3. 분석 절차

먼저 `05`와 `06`의 Section0 record-level diff inventory를 만든다.

진단 리포트:

```text
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/stage37_section0_diff.md
```

그 다음 `05` 실패 baseline에 남은 record 타입을 positive에서 graft한 variant를 만든다.

## 4. Variant 설계

| variant | 적용 내용 | 목적 |
|---|---|---|
| 01_shape_component_records | `SHAPE_COMPONENT` payload graft | common shape attr 외 미모델링 GSO 필드 확인 |
| 02_shape_picture_records | `SHAPE_PICTURE` payload graft | image ref/storage id/transform payload 확인 |
| 03_table_records | `TABLE` payload graft | table raw attr/tail payload 확인 |
| 04_para_line_seg_records | `PARA_LINE_SEG` payload graft | line_seg raw 차이가 한컴 읽기 오류에 영향 있는지 확인 |
| 05_page_records | `PAGE_DEF`, `PAGE_BORDER_FILL`, `FOOTNOTE_SHAPE` payload graft | section/page child payload 확인 |
| 06_shape_component_picture | 01 + 02 | 그림 개체 payload 조합 확인 |
| 07_shape_picture_table | 01 + 02 + 03 | 그림 + 표 payload 조합 확인 |
| 08_all_remaining_candidate_types | 01 + 02 + 03 + 04 + 05 | Section0 raw 성공을 record 타입 조합으로 재현 가능한지 확인 |

## 5. 판정표

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_shape_component_records |  |  |  |  |  |  |  |
| 02_shape_picture_records |  |  |  |  |  |  |  |
| 03_table_records |  |  |  |  |  |  |  |
| 04_para_line_seg_records |  |  |  |  |  |  |  |
| 05_page_records |  |  |  |  |  |  |  |
| 06_shape_component_picture |  |  |  |  |  |  |  |
| 07_shape_picture_table |  |  |  |  |  |  |  |
| 08_all_remaining_candidate_types |  |  |  |  |  |  |  |

## 6. 기대 해석

```text
01/02/06에서 성공:
  그림/도형 payload가 직접 원인.

03에서 성공:
  TABLE raw payload가 직접 원인.

04에서 성공:
  LINE_SEG raw payload가 한컴 읽기 오류에 영향.

08에서만 성공:
  단일 record 타입이 아니라 여러 payload 조합 문제.

08도 실패:
  record 타입 단위가 아니라 특정 record index 범위 또는 record 간 byte alignment 문제.
```

## 7. 하지 않을 것

```text
- 구현 확정
- BodyText 전체 raw graft를 곧바로 해법으로 삼기
- Stage36 성공을 충분한 구현 근거로 오해하기
```

