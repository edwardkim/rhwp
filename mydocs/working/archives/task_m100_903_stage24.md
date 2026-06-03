# Task m100 #903 Stage 24

## 1. 단계 목적

Stage 23에서 첫 번째 연도별 동향 표의 `TABLE record + encoded zone tail`이 들어가면 한컴 출력 경계가 두 번째 연도별 동향 제목까지 이동했다.

```text
7페이지 2. 연도별 동향
```

Stage 24는 이 다음 경계인 문단 `0:87` 제목과 문단 `0:89` 두 번째 연도별 동향 표를 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 24 산출물:

```text
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:85 빈 문단
문단 0:86 빈 문단
문단 0:87 "2. 연도별 동향"
문단 0:88 빈 문단
문단 0:89 두 번째 연도별 동향 표, 3행 x 7열
문단 0:90 빈 문단
문단 0:91 빈 문단
문단 0:92 "□ 업종별 동향(상위 5개)"
```

Stage 24 공통 기준선은 Stage 23의 `08_year_full_tuple_plus_next_boundary` 상태를 재현한다.

즉 다음이 이미 포함된다.

```text
문단 0:50 "2. 연도별 동향"
문단 0:52 첫 번째 연도별 동향 표 full object + encoded zone tail
문단 0:53 빈 문단
문단 0:54 "□ 업종별 동향(상위 5개)"
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_pre_second_year_empty_paras | 문단 `0:85`, `0:86` 빈 문단 paragraph record |
| 02_second_year_title_para_record | `01` + 문단 `0:87` 제목 paragraph record |
| 03_second_year_table_ctrl_header | `02` + 문단 `0:88` 빈 문단 + 문단 `0:89` 두 번째 연도별 표 CTRL_HEADER |
| 04_second_year_table_record_with_tail | `02` + 문단 `0:88` 빈 문단 + 문단 `0:89` 두 번째 연도별 표 TABLE record + encoded zone tail |
| 05_second_year_table_all_cell_headers | `02` + 문단 `0:88` 빈 문단 + 문단 `0:89` 두 번째 연도별 표 전체 cell LIST_HEADER/PARA_HEADER |
| 06_second_year_table_full_object_with_tail | `02` + 문단 `0:88` 빈 문단 + 문단 `0:89` 두 번째 연도별 표 full object + encoded zone tail |
| 07_second_year_host_para_plus_table_full_tuple | `06` + 문단 `0:89` host paragraph record |
| 08_second_year_full_tuple_plus_next_boundary | `07` + 문단 `0:90`, `0:91`, `0:92` paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/01_pre_second_year_empty_paras.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/02_second_year_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/03_second_year_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/04_second_year_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/05_second_year_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/06_second_year_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/07_second_year_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/08_second_year_full_tuple_plus_next_boundary.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage24_generate_second_year_trend_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 53 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/01_pre_second_year_empty_paras.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/02_second_year_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/03_second_year_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/04_second_year_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/05_second_year_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/06_second_year_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/07_second_year_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage24_second_year_trend_probe/08_second_year_full_tuple_plus_next_boundary.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_pre_second_year_empty_paras | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 02_second_year_title_para_record | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 03_second_year_table_ctrl_header | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 04_second_year_table_record_with_tail | 정상 | 8페이지까지만 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 05_second_year_table_all_cell_headers | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 06_second_year_table_full_object_with_tail | 정상 | 8페이지까지만 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 07_second_year_host_para_plus_table_full_tuple | 정상 | 8페이지까지만 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 08_second_year_full_tuple_plus_next_boundary | 정상 | 8페이지까지만 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |

판정 포인트:

```text
- 한컴 출력 경계가 7페이지 "2. 연도별 동향" 이후로 이동하는지
- 문단 0:89 두 번째 연도별 동향 표가 출력되는지
- 다음 경계인 "□ 업종별 동향(상위 5개)"까지 이동하는지
- 판정 유형이 파일손상, 파일 읽기 오류, 정상 열림 중 무엇인지
- 셀 텍스트가 여전히 셀 영역 위에 걸치는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 현재 가설

Stage 24는 Stage 23의 반복 패턴 검증이다.

첫 번째 연도별 동향 표와 두 번째 연도별 동향 표는 모두 `3행 x 7열`이며, 정답 HWP의 raw table record header 형태도 동일 계열이다.

따라서 다음을 확인한다.

```text
1. 두 번째 연도별 동향 표에서도 TABLE record + encoded zone tail이 경계를 이동시키는지
2. full object가 rhwp-studio 정상 렌더링을 유지하는지
3. 다음 경계인 문단 0:92가 별도 병목인지
```

## 9. 판정 해석

### 9.1 파일손상 해소 후보

Stage 24에서 처음으로 한컴 에디터 정상 열림 variant가 나왔다.

```text
04_second_year_table_record_with_tail
06_second_year_table_full_object_with_tail
07_second_year_host_para_plus_table_full_tuple
08_second_year_full_tuple_plus_next_boundary
```

이들은 모두 `TABLE record + encoded zone tail`을 포함한다.

즉 현재까지 추적한 한컴 파일손상 문제의 핵심 조건은 거의 확정적이다.

```text
각 표의 TABLE record payload와 encoded zone tail을 보존해야 한다.
```

### 9.2 파일손상과 누락 출력 분리

정상 열림 variant는 8페이지까지만 출력되고, 9페이지 마지막은 출력되지 않았다.

따라서 Stage 24 이후 문제는 두 갈래로 분리된다.

```text
1. 파일손상 문제: Stage 24 기준으로 해결 가능성이 높음
2. 마지막 페이지 누락 문제: 아직 materialize하지 않은 9페이지 블록 때문
```

### 9.3 다음 경계

정답 HWP 기준 마지막 페이지 진입부는 다음이다.

```text
문단 0:92 "□ 업종별 동향(상위 5개)"
문단 0:93 빈 문단
문단 0:94 마지막 업종별 동향 표, 11행 x 7열
문단 0:95 "□ 국가별 동향(상위 5개)"
문단 0:96 빈 문단
문단 0:97 마지막 국가별 동향 표, 11행 x 7열
```

Stage 25는 문단 `0:92` 제목과 문단 `0:94` 마지막 업종별 동향 표를 대상으로 한다.
