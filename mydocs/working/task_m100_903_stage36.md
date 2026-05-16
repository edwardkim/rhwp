# Task m100 #903 Stage 36 작업 기록

## 1. 목적

Stage35에서 DocInfo `BIN_DATA`/`PARA_SHAPE` 보정만으로는 한컴 파일 읽기 오류가 해결되지 않았다.
Stage36은 Stage34에서 관찰한 BodyText record payload 차이를 분리한다.

분리 대상:

```text
CTRL_HEADER
LIST_HEADER
PARA_HEADER
Section raw stream
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

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/
```

## 3. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/01_ctrl_header_records.hwp
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/02_list_header_records.hwp
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/03_para_header_records.hwp
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/04_ctrl_plus_list_headers.hwp
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/06_section0_raw_only.hwp
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/07_section0_section1_raw.hwp
```

## 4. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage36_generate_bodytext_payload_probe_variants -- --nocapture
```

결과:

```text
test task903_stage36_generate_bodytext_payload_probe_variants ... ok
```

모든 파일은 rhwp 재로드 기준 9페이지다.

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_ctrl_header_records | 374784 | ok, pages=9 |
| 02_list_header_records | 375296 | ok, pages=9 |
| 03_para_header_records | 374784 | ok, pages=9 |
| 04_ctrl_plus_list_headers | 375296 | ok, pages=9 |
| 05_ctrl_list_para_headers | 375808 | ok, pages=9 |
| 06_section0_raw_only | 375808 | ok, pages=9 |
| 07_section0_section1_raw | 375808 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_ctrl_header_records | `43f89e4c3ecdb68fadf840f7ffb29eeea46a257f64ce549bc175305c285f74f8` |
| 02_list_header_records | `557e4b3f450dd27e56fe0989e1ec8915e93c477b06540b2b32082c3e1858b5c7` |
| 03_para_header_records | `d3a6236800bce2d3bf8abdac4e47bc20562dd5bcc9f8fdd3443c22c0adcbb725` |
| 04_ctrl_plus_list_headers | `a47876a686d6198e2e23b26defd71a3ac475864be17fcfce4b71cc96f92b9ac4` |
| 05_ctrl_list_para_headers | `7d9594ad077a42a1a5fdcfd85966e28f9e75a20a2ec99885643f6bc6c34b6ce6` |
| 06_section0_raw_only | `26f3c3e565d667f4187a7ce879f81ff353828f2dcdab00d378c757766281fa80` |
| 07_section0_section1_raw | `e484a081c2fc4d6cc273f2d6830ec4f432d9e3cc23e3cc0f1e73da8fe4f00d11` |

대표 IR 비교:

```text
target/debug/rhwp ir-diff \
  output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp \
  output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp \
  --summary
```

결과:

```text
=== 비교 완료: 차이 0 건 ===
```

`06_section0_raw_only`, `07_section0_section1_raw`도 positive와 IR diff 0건이다.

## 5. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | 이미지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_ctrl_header_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 개선 |
| 02_list_header_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 버그 |
| 03_para_header_records | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 개선 |
| 04_ctrl_plus_list_headers | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 개선 |
| 05_ctrl_list_para_headers | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | 성공 | rhwp-studio 이미지 렌더링 개선 |
| 06_section0_raw_only | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |
| 07_section0_section1_raw | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |

추가 관찰:

```text
rhwp-studio 이미지 렌더링 개선:
  1페이지 표 안 2개 이미지 렌더링 성공

rhwp-studio 이미지 렌더링 버그:
  1페이지 표 안 2개 이미지 중 첫 번째 이미지만 출력
```

판정 포인트:

```text
- 01에서 한컴 판정이 움직이면 CTRL_HEADER 확장 payload가 핵심이다.
- 02에서 표/셀/이미지가 움직이면 LIST_HEADER 확장 payload가 핵심이다.
- 03에서 한컴 판정이 움직이면 PARA_HEADER extra payload가 핵심이다.
- 05에서만 움직이면 header-class payload 조합 문제다.
- 06/07에서만 움직이면 record 타입 단위 graft로 부족하고 Section raw stream 전체 차이를 더 봐야 한다.
```

## 6. 판정 해석

Stage36 판정으로 다음을 확정한다.

```text
1. 한컴 파일 읽기 오류의 직접 원인은 Section0 BodyText payload 안에 있다.
2. Section1 raw stream은 필수 조건이 아니다.
   - 06_section0_raw_only 성공
   - 07_section0_section1_raw 성공
3. CTRL_HEADER/LIST_HEADER/PARA_HEADER record 타입 단위 graft만으로는 부족하다.
   - 01~05 모두 한컴 파일 읽기 오류
4. 따라서 Section0 안에서 header-class 외 record payload 또는 record 조합/순서/범위가 원인이다.
```

이미지 렌더링은 `CTRL_HEADER` 또는 `PARA_HEADER` graft만으로도 개선된다.

```text
01, 03, 04, 05: 1페이지 표 안 2개 이미지 렌더링 성공
02: 1페이지 표 안 2개 이미지 중 첫 번째만 출력
```

즉 이미지 렌더링 문제와 한컴 파일 읽기 오류는 같은 축에 걸쳐 있지만,
이미지 쪽은 `CTRL_HEADER/PARA_HEADER` 일부로 회복되고,
한컴 읽기 오류는 Section0 전체 raw payload가 필요하다.

## 7. 다음 단계

Stage37은 Section0 raw 전체와 `05_ctrl_list_para_headers` 사이의 남은 차이를 좁힌다.

비교 기준:

```text
성공:
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/06_section0_raw_only.hwp

실패:
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

두 파일은 모두 positive와 IR diff 0건이다.
따라서 Stage37은 Section0 record payload 중 `CTRL_HEADER/LIST_HEADER/PARA_HEADER`를 제외하고
남은 차이를 record 타입별로 비교한다.

우선 후보:

```text
- SHAPE_COMPONENT
- SHAPE_PICTURE
- TABLE
- PAGE_DEF / PAGE_BORDER_FILL / FOOTNOTE_SHAPE
- PARA_LINE_SEG
```

