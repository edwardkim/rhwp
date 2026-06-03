# Task m100 #903 Stage 17

## 1. 단계 목적

Stage 16 이후 결론:

```text
문단 0:10 차트 표 tuple과 문단 0:13 제목 텍스트를 맞춰도 한컴 파일손상은 남는다.
한컴 출력 경계는 문단 0:14 업종별 동향 표 진입 시점으로 보인다.
```

따라서 Stage 17은 문단 `0:14`의 4행 x 6열 `업종별 동향` 표 raw tuple을 분리 검증한다.

추가로 Stage 16 중 발견된 HWPX XML entity 텍스트 누락도 #903 안에서 같이 처리했다.

```text
&lt;  -> <
&gt;  -> >
&amp; -> &
```

## 2. 입력과 기준

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 17 산출물:

```text
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 사전 수정: XML entity 텍스트 보존

수정 파일:

```text
src/parser/hwpx/section.rs
tests/hwpx_to_hwp_adapter.rs
```

핵심:

```text
quick-xml은 &lt;, &gt; 같은 XML entity를 Event::GeneralRef로 전달한다.
기존 HWPX 파서는 Event::Text만 누적해서 제목의 '<', '>'를 버렸다.
```

적용:

```text
decode_xml_general_ref 추가
read_text_content_with_tabs에서 GeneralRef 누적
field/compose/ruby/form text reader에도 동일 처리
```

회귀 테스트:

```text
cargo test test_parse_text_preserves_xml_general_refs -- --nocapture
cargo test --test hwpx_to_hwp_adapter task903_hwpx_h_01_preserves_xml_entity_text -- --nocapture
```

결과:

```text
ok
```

수정 후 원본 HWPX IR 확인:

```text
문단 0:9  "< 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) >"
문단 0:13 "< 업종별 동향(억 달러, %) >"
```

## 4. Stage 17 기준선

Stage 17은 Stage 16 `06_chart_tuple_plus_following_title_text` 상태를 기준선으로 재현한다.

기준선 포함:

```text
Stage 8 section core
first table picture/full structural payload
second table child headers
chart host paragraph record
chart table full object with encoded zone tail
paragraph 0:13 following title record
```

그 위에 문단 `0:14` industry table 쪽만 additive하게 적용한다.

## 5. Variant

| variant | 적용 payload |
|---|---|
| 01_industry_table_ctrl_header | 문단 `0:14` 표 CTRL_HEADER만 정답에서 복사 |
| 02_industry_table_record_with_tail | 문단 `0:14` TABLE record + encoded zone tail |
| 03_industry_table_all_cell_headers | 문단 `0:14` 전체 cell LIST_HEADER/PARA_HEADER |
| 04_industry_table_full_object_with_tail | 문단 `0:14` 표 전체 object + encoded zone tail |
| 05_industry_host_para_plus_table_full_tuple | 문단 `0:14` host paragraph + 표 전체 object |
| 06_industry_tuple_plus_next_boundary | variant 05 + 문단 `0:15` next boundary paragraph |

## 6. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/01_industry_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/02_industry_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/03_industry_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/04_industry_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/05_industry_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/06_industry_tuple_plus_next_boundary.hwp
```

## 7. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage17_generate_industry_table_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 46 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 8. Raw 확인

정답 HWP 문단 `0:14` 표:

```text
attr=0x0000000c
raw=[10, 23, 2A, 08, ...]
```

Stage 16 기준선 문단 `0:14` 표:

```text
attr=0x00000004
raw=[10, 03, 2A, 00, ...]
```

Stage 17 `02_industry_table_record_with_tail`:

```text
attr=0x0000000c
raw=[10, 03, 2A, 00, ...]
```

해석:

```text
TABLE record attr는 정답값으로 바뀌지만, CTRL_HEADER raw는 아직 compact 상태다.
```

Stage 17 `04_industry_table_full_object_with_tail`:

```text
attr=0x0000000c
raw=[10, 23, 2A, 08, ...]
```

해석:

```text
표 전체 object 복사에서는 CTRL_HEADER raw까지 정답 수준으로 들어간다.
```

## 9. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/01_industry_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/02_industry_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/03_industry_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/04_industry_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/05_industry_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage17_industry_table_probe/06_industry_tuple_plus_next_boundary.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 셀 세로 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_industry_table_ctrl_header | 파일손상 | `< 업종별 동향(억 달러, %) >` | 셀 영역 위에 걸침 | 정상 | 셀 위에 걸친 경우 스페이스 하나 입력하면 한컴 에디터가 정상 배치 처리 |
| 02_industry_table_record_with_tail | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸침 | 정상 |  |
| 03_industry_table_all_cell_headers | 파일손상 | `< 업종별 동향(억 달러, %) >` | 셀 영역 위에 걸침 | 정상 |  |
| 04_industry_table_full_object_with_tail | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸침 | 정상 | 셀에 따라 정상 위치로 배치되는 경우도 존재 |
| 05_industry_host_para_plus_table_full_tuple | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸침 | 정상 | 셀에 따라 정상 위치로 배치되는 경우도 존재 |
| 06_industry_tuple_plus_next_boundary | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸침 | 정상 | 셀에 따라 정상 위치로 배치되는 경우도 존재 |

판정 포인트:

```text
- 한컴 파일손상이 사라지는지
- 한컴 출력 경계가 문단 0:14 이후로 이동하는지
- 업종별 동향 표가 출력되는지
- Stage 16에서 확인한 '<', '>' 출력이 유지되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 10. 판정 해석

Stage 17에서 파일손상은 사라지지 않았다.

다만 출력 경계는 의미 있게 이동했다.

```text
01, 03: < 업종별 동향(억 달러, %) > 에서 손상
02, 04, 05, 06: < 국가별 동향(억 달러, %) > 까지 출력 후 손상
```

해석:

```text
문단 0:14 업종별 동향 표의 TABLE record 계열 payload는 한컴 출력 경계를 다음 title까지 밀어낸다.
즉 업종별 동향 표 자체는 손상 원인의 일부였고, 다음 경계는 국가별 동향 표 진입 시점으로 이동했다.
```

셀 세로 배치 문제:

```text
모든 variant에서 셀 내용이 셀 영역 위에 걸친다.
한컴 에디터에서 스페이스 하나를 입력하면 정상 배치 처리된다.
```

이는 저장된 레이아웃 캐시성 정보, 특히 cell paragraph의 `lineSeg` 또는 paragraph/header tuple이 한컴의 재조판 조건과 맞지 않는다는 신호다.
rhwp-studio는 자체 렌더링으로 정상 표시하지만, 한컴은 저장된 HWP record를 읽으면서 lineSeg/record tuple을 더 엄격하게 해석하는 것으로 보인다.

## 11. Stage 18 제안

Stage 18은 두 축을 분리한다.

```text
1. 문단 0:21 국가별 동향 표 tuple probe
2. cell paragraph lineSeg/reflow cache probe
```

우선 경계 이동을 추적하기 위해 문단 `0:21`의 국가별 동향 표를 Stage 17 `06` 기준선 위에 additive 적용한다.

후보 variant:

```text
01_country_title_para_record
02_country_table_ctrl_header
03_country_table_record_with_tail
04_country_table_all_cell_headers
05_country_table_full_object_with_tail
06_country_host_para_plus_table_full_tuple
```

동시에 셀 세로 배치 문제를 따로 확인한다.

후보 variant:

```text
07_industry_cell_paragraph_linesegs_from_reference
08_industry_cell_paragraph_records_from_reference
09_industry_tuple_plus_cell_paragraph_records
```

판정 기준:

```text
- 한컴 파일손상 경계가 국가별 동향 표 이후로 이동하는지
- 셀 내용이 셀 영역 위에 걸치는 문제가 사라지는지
- 스페이스 입력 후 정상화되는 현상과 같은 재조판 트랩인지 확인
- rhwp-studio 정상 렌더링을 유지하는지
```
