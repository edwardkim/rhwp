# Task m100 #903 Stage 37 작업 기록

## 1. 목적

Stage36에서 다음 대조가 성립했다.

```text
05_ctrl_list_para_headers:
  - positive와 IR diff 0건
  - 한컴 파일 읽기 오류

06_section0_raw_only:
  - positive와 IR diff 0건
  - 한컴 성공
```

따라서 남은 원인은 IR 모델 차이가 아니라 `BodyText/Section0`의 record payload 바이트 차이다.
Stage37은 `05` 실패 파일과 `06` 성공 파일의 Section0 record payload 차이를 타입별로 분리한다.

## 2. 기준 파일

Positive raw source:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

실패 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

성공 baseline:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/06_section0_raw_only.hwp
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/
```

## 3. Section0 diff inventory

진단 리포트:

```text
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/stage37_section0_diff.md
```

요약:

| 항목 | 값 |
|---|---:|
| failing Section0 bytes | 225272 |
| success Section0 bytes | 225296 |
| record count | 7879 |
| differing records | 24 |

차이가 나는 record tag는 세 종류뿐이다.

| tag | name | diff records |
|---:|---|---:|
| 76 | `SHAPE_COMPONENT` | 6 |
| 77 | `TABLE` | 13 |
| 85 | `SHAPE_PICTURE` | 5 |

관찰:

```text
- PAGE_DEF / PAGE_BORDER_FILL / FOOTNOTE_SHAPE / PARA_LINE_SEG 차이는 실제로 없었다.
- Section0 raw 전체 성공과 실패의 차이는 SHAPE_COMPONENT, SHAPE_PICTURE, TABLE payload로 좁혀졌다.
- TABLE record는 주로 size diff이며, SHAPE_COMPONENT/SHAPE_PICTURE는 data diff다.
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/01_shape_component_records.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/02_shape_picture_records.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/03_table_records.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/04_para_line_seg_records.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/05_page_records.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/06_shape_component_picture.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/07_shape_picture_table.hwp
output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe/08_all_remaining_candidate_types.hwp
```

## 5. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage37_generate_section0_record_type_probe_variants -- --nocapture
```

결과:

```text
test task903_stage37_generate_section0_record_type_probe_variants ... ok
```

모든 파일은 rhwp 재로드 기준 9페이지다.

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_shape_component_records | 375808 | ok, pages=9 |
| 02_shape_picture_records | 375808 | ok, pages=9 |
| 03_table_records | 375808 | ok, pages=9 |
| 04_para_line_seg_records | 375808 | ok, pages=9 |
| 05_page_records | 375808 | ok, pages=9 |
| 06_shape_component_picture | 375808 | ok, pages=9 |
| 07_shape_picture_table | 375808 | ok, pages=9 |
| 08_all_remaining_candidate_types | 375808 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_shape_component_records | `59e3cdf0ad8cc0fa0ed98e0bd45c31bcd4a71aa1184a7176294a616bb203ff52` |
| 02_shape_picture_records | `0ed7ecbbc797c8fd98482a4e80062641b8253495cb79bbdfbb1fc1bd4924613b` |
| 03_table_records | `44fecf122e25aa35fa2a61922e746dc3d74694b8a7d89b49f7299e937bc9dc07` |
| 04_para_line_seg_records | `7d9594ad077a42a1a5fdcfd85966e28f9e75a20a2ec99885643f6bc6c34b6ce6` |
| 05_page_records | `7d9594ad077a42a1a5fdcfd85966e28f9e75a20a2ec99885643f6bc6c34b6ce6` |
| 06_shape_component_picture | `d991123d54295510fe3831aa5d4c86afe50f8a5848954f18bc211a079df78f38` |
| 07_shape_picture_table | `fd330c684d81e4966b16e8c9f81b32549bd79da490abc68feee9947f385d7434` |
| 08_all_remaining_candidate_types | `fd330c684d81e4966b16e8c9f81b32549bd79da490abc68feee9947f385d7434` |

관찰:

```text
04_para_line_seg_records == 05_page_records == Stage36 05_ctrl_list_para_headers
07_shape_picture_table == 08_all_remaining_candidate_types
```

이는 실제 diff inventory와 일치한다.
`PARA_LINE_SEG`와 page 계열 record에는 남은 차이가 없으므로 graft해도 파일 바이트가 변하지 않는다.

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_shape_component_records | 파일손상 | 1페이지 첫 테이블 | 실패 | 실패 | 실패 | 성공 |  |
| 02_shape_picture_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 03_table_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 04_para_line_seg_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 05_page_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 2페이지 이미지 개체 묶기 렌더링 버그 |
| 06_shape_component_picture | 파일손상 | 1페이지 첫 테이블 (이미지 2개 출력) | 실패 | 실패 | 실패 | 성공 |  |
| 07_shape_picture_table | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 08_all_remaining_candidate_types | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |

## 7. 판정 해석 기준

```text
01 성공:
  SHAPE_COMPONENT payload가 직접 원인.

02 성공:
  SHAPE_PICTURE payload가 직접 원인.

03 성공:
  TABLE payload/tail이 직접 원인.

06 성공:
  그림 개체 payload 조합(SHAPE_COMPONENT + SHAPE_PICTURE)이 직접 원인.

07 성공:
  그림 payload와 TABLE payload가 함께 필요하다.

07도 실패:
  record 타입 단위 graft로는 부족하며, index 범위 또는 record 간 payload 조합을 더 좁혀야 한다.
```

## 8. 판정 해석

Stage37 판정으로 Stage36의 "Section0 raw 전체가 필요하다"는 결론을 더 좁혔다.

```text
한컴 파일 읽기 오류 후보:
  - SHAPE_COMPONENT payload 6개
  - SHAPE_PICTURE payload 5개
  - TABLE payload 13개

후보가 아닌 것:
  - CTRL_HEADER/LIST_HEADER/PARA_HEADER 단독
  - DocInfo BIN_DATA/PARA_SHAPE 단독
  - PARA_LINE_SEG
  - PAGE_DEF/PAGE_BORDER_FILL/FOOTNOTE_SHAPE
```

특히 `07_shape_picture_table`이 성공했으므로,
Section0 raw stream 전체를 보존해야 하는 것이 아니라 다음 세 record군의 payload 조합이 필요하다.

```text
SHAPE_COMPONENT + SHAPE_PICTURE + TABLE
```

세부 해석:

```text
1. SHAPE_COMPONENT 단독은 한컴 판정을 "파일 읽기 오류"에서 "파일손상"으로 이동시킨다.
   - 한컴 parser가 더 진행하지만 1페이지 첫 테이블에서 무너진다.

2. SHAPE_COMPONENT + SHAPE_PICTURE 조합은 1페이지 이미지 2개 출력까지 회복한다.
   - 이미지 개체 payload는 두 record군이 함께 맞아야 한다.
   - 그러나 문서 전체는 여전히 파일손상이다.

3. TABLE payload를 함께 graft한 07에서 완전 성공한다.
   - TABLE payload/tail은 한컴 문서 유효성과 표/셀 배치 회복에 필요하다.

4. 08은 07과 동일 hash이며 동일 성공이다.
   - 추가 후보였던 PARA_LINE_SEG/page 계열은 영향이 없다.
```

따라서 다음 구현 후보는 다음 두 축이다.

```text
1. HWPX shape/picture control을 HWP SHAPE_COMPONENT/SHAPE_PICTURE payload와 호환되게 저장한다.
2. HWPX table control을 HWP TABLE payload/tail과 호환되게 저장한다.
```

## 9. 다음 단계

Stage38은 바로 구현하지 않는다.
먼저 24개 record를 index 단위로 더 나누어,
각 payload 차이가 어떤 IR/control 필드 누락에 대응하는지 확인한다.

우선 순위:

```text
1. TABLE 13개 size diff의 의미를 먼저 확인한다.
   - TABLE payload size가 2 bytes씩 부족한 사례가 많다.
   - 한컴 성공의 최종 조건이 TABLE 조합에서 발생했으므로 우선순위가 높다.

2. SHAPE_COMPONENT/SHAPE_PICTURE 11개 data diff를 그림 개체별로 묶는다.
   - 1페이지 표 안 이미지 2개
   - 2페이지 이미지 개체 묶기
   - 기타 그림/도형 개체

3. 구현은 위 매핑이 끝난 뒤 최소 필드 단위로 한다.
```
