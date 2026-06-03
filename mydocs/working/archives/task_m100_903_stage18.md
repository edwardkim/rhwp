# Task m100 #903 Stage 18

## 1. 단계 목적

Stage 17 판정에서 의미 있는 경계 이동이 있었다.

```text
02/04/05/06 variant는 < 국가별 동향(억 달러, %) > 까지 출력 후 파일손상
```

따라서 Stage 18은 두 문제를 분리한다.

```text
1. 문단 0:21 국가별 동향 표 진입부가 다음 파일손상 원인인지 확인
2. 업종별 동향 표 셀 내용이 셀 영역 위에 걸치는 재조판 문제를 별도 확인
```

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 18 산출물:

```text
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:20 "< 국가별 동향(억 달러, %) >"
문단 0:21 국가별 동향 표, 4행 x 6열
```

Stage 17 기준선에서도 문단 `0:20` 제목은 보존된다.

## 4. Stage 18 기준선

공통 기준선:

```text
Stage 16 06_chart_tuple_plus_following_title_text
```

국가별 표 probe `01~06`은 여기에 Stage 17 `06_industry_tuple_plus_next_boundary` 상태를 더해 시작한다.

즉 다음이 포함된다.

```text
문단 0:14 업종별 표 host paragraph
문단 0:14 업종별 표 full object with encoded zone tail
문단 0:15 next boundary paragraph
```

재조판 probe `07~09`는 업종별 표의 셀 세로 배치 원인만 보기 위해 더 작은 기준선을 사용한다.

```text
Stage 16 06 기준선 + 문단 0:14 업종별 표 TABLE record with encoded zone tail
```

## 5. Variant

국가별 동향 표 경계 probe:

| variant | 적용 payload |
|---|---|
| 01_country_title_para_record | 문단 `0:20` 국가별 제목 paragraph record |
| 02_country_table_ctrl_header | `01` + 문단 `0:21` 표 CTRL_HEADER |
| 03_country_table_record_with_tail | `01` + 문단 `0:21` TABLE record + encoded zone tail |
| 04_country_table_all_cell_headers | `01` + 문단 `0:21` 전체 cell LIST_HEADER/PARA_HEADER |
| 05_country_table_full_object_with_tail | `01` + 문단 `0:21` 표 full object + encoded zone tail |
| 06_country_host_para_plus_table_full_tuple | `01` + 문단 `0:21` host paragraph + 표 full object |

업종별 표 재조판 probe:

| variant | 적용 payload |
|---|---|
| 07_industry_record_plus_cell_linesegs | 문단 `0:14` TABLE record + cell paragraph lineSeg |
| 08_industry_record_plus_cell_para_records | 문단 `0:14` TABLE record + cell paragraph records |
| 09_industry_record_headers_plus_cell_para_records | 문단 `0:14` TABLE record + cell headers + cell paragraph records |

## 6. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/01_country_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/02_country_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/03_country_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/04_country_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/05_country_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/06_country_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/07_industry_record_plus_cell_linesegs.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/08_industry_record_plus_cell_para_records.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/09_industry_record_headers_plus_cell_para_records.hwp
```

## 7. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage18_generate_country_reflow_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 47 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 8. Raw 확인

정답 HWP 문단 `0:21` 국가별 표:

```text
attr=0x0000000c
raw=[10, 23, 2A, 08, ...]
```

Stage 18 `03_country_table_record_with_tail`:

```text
attr=0x0000000c
raw=[10, 03, 2A, 00, ...]
```

해석:

```text
TABLE attr는 정답값으로 바뀌지만 CTRL_HEADER raw는 compact 상태다.
```

Stage 18 `05_country_table_full_object_with_tail`:

```text
attr=0x0000000c
raw=[10, 23, 2A, 08, ...]
```

해석:

```text
국가별 표 전체 object 복사에서는 CTRL_HEADER raw까지 정답 수준으로 들어간다.
```

## 9. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/01_country_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/02_country_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/03_country_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/04_country_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/05_country_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/06_country_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/07_industry_record_plus_cell_linesegs.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/08_industry_record_plus_cell_para_records.hwp
output/poc/hwpx2hwp/task903/stage18_country_reflow_probe/09_industry_record_headers_plus_cell_para_records.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 셀 세로 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_country_title_para_record | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 02_country_table_ctrl_header | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 03_country_table_record_with_tail | 파일손상 | `< 지역별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 다음 표 이전까지 출력됨. 처음으로 2페이지까지 렌더링 |
| 04_country_table_all_cell_headers | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 05_country_table_full_object_with_tail | 파일손상 | `< 지역별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 다음 표 이전까지 출력됨. 2페이지까지 렌더링 |
| 06_country_host_para_plus_table_full_tuple | 파일손상 | `< 지역별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 다음 표 이전까지 출력됨. 2페이지까지 렌더링 |
| 07_industry_record_plus_cell_linesegs | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 08_industry_record_plus_cell_para_records | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 09_industry_record_headers_plus_cell_para_records | 파일손상 | `< 국가별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |

판정 포인트:

```text
- 한컴 파일손상 경계가 국가별 표 이후로 이동하는지
- 국가별 동향 표가 출력되는지
- 07~09에서 업종별 표 셀 내용이 셀 영역 위에 걸치는 현상이 사라지는지
- Stage 17에서 확인한 '<', '>' 출력은 유지되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 10. 판정 해석

### 10.1 파일손상 경계

Stage 18에서 의미 있는 개선 신호는 `03`, `05`, `06`이다.

```text
03_country_table_record_with_tail
05_country_table_full_object_with_tail
06_country_host_para_plus_table_full_tuple
```

이 세 variant는 한컴 출력 경계가 `< 국가별 동향(억 달러, %) >`에서 `< 지역별 동향(억 달러, %) >`로 이동했다.
즉 국가별 표 진입부의 `TABLE record + encoded zone tail`이 한컴 파서의 다음 경계를 전진시키는 핵심 payload로 보인다.

반대로 다음 variant는 경계를 움직이지 못했다.

```text
02_country_table_ctrl_header
04_country_table_all_cell_headers
```

따라서 국가별 표에서는 `CTRL_HEADER` 또는 cell header 단독보다 `TABLE record` 자체와 그 tail 쪽이 더 큰 신호다.

### 10.2 셀 세로 배치

모든 variant에서 셀 텍스트가 셀 영역 위에 걸치는 현상은 유지되었다.

특히 다음 재조판 probe도 개선이 없었다.

```text
07_industry_record_plus_cell_linesegs
08_industry_record_plus_cell_para_records
09_industry_record_headers_plus_cell_para_records
```

따라서 셀 세로 배치 문제는 단순히 cell paragraph `LINE_SEG` 또는 paragraph record를 정답에서 복사하는 방식으로는 해결되지 않는다.
한컴 에디터에서 해당 셀에 스페이스를 입력하면 정상 배치로 재계산된다는 Stage 17 관찰과 결합하면, 저장된 reflow/cache 계열 데이터가 한컴의 재계산 조건을 만족하지 못하거나, 셀 내부 paragraph/header/height 조합이 서로 일관되지 않을 가능성이 높다.

## 11. 다음 단계

Stage 19는 두 축으로 분리한다.

```text
1. 지역별 동향 표에 Stage 18의 TABLE record + encoded zone tail 패턴을 적용해 파일손상 경계를 더 뒤로 이동시킨다.
2. 셀 세로 배치 문제는 별도 reflow/cache probe로 분리하고, 우선 파일손상 경계 전진을 계속 우선한다.
```

우선순위는 파일손상 경계 전진이다.
Stage 18에서 `03/05/06`이 처음으로 2페이지까지 렌더링되었으므로, 같은 패턴을 다음 표인 지역별 동향 표에 반복 적용해 경계가 그 다음 블록으로 이동하는지 확인한다.
