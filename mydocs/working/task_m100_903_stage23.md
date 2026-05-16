# Task m100 #903 Stage 23

## 1. 단계 목적

Stage 22에서 국가별 표의 `TABLE record + encoded zone tail`이 들어가면 한컴 출력 경계가 다음 장 제목까지 이동했다.

```text
4페이지 2. 연도별 동향
```

Stage 23은 이 다음 경계인 문단 `0:50` 제목과 문단 `0:52` 연도별 동향 표를 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 23 산출물:

```text
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:48 빈 문단
문단 0:49 빈 문단
문단 0:50 "2. 연도별 동향"
문단 0:51 빈 문단
문단 0:52 연도별 동향 표, 3행 x 7열
문단 0:53 빈 문단
문단 0:54 "□ 업종별 동향(상위 5개)"
```

Stage 23 공통 기준선은 Stage 22의 `08_top_country_full_tuple_plus_next_boundary` 상태를 재현한다.

즉 다음이 이미 포함된다.

```text
문단 0:43 "□ 국가별 동향(상위 5개)"
문단 0:44 국가별 표 full object + encoded zone tail
문단 0:45 "□ 지역별 동향"
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_pre_year_empty_paras | 문단 `0:48`, `0:49` 빈 문단 paragraph record |
| 02_year_title_para_record | `01` + 문단 `0:50` 제목 paragraph record |
| 03_year_table_ctrl_header | `02` + 문단 `0:51` 빈 문단 + 문단 `0:52` 연도별 표 CTRL_HEADER |
| 04_year_table_record_with_tail | `02` + 문단 `0:51` 빈 문단 + 문단 `0:52` 연도별 표 TABLE record + encoded zone tail |
| 05_year_table_all_cell_headers | `02` + 문단 `0:51` 빈 문단 + 문단 `0:52` 연도별 표 전체 cell LIST_HEADER/PARA_HEADER |
| 06_year_table_full_object_with_tail | `02` + 문단 `0:51` 빈 문단 + 문단 `0:52` 연도별 표 full object + encoded zone tail |
| 07_year_host_para_plus_table_full_tuple | `06` + 문단 `0:52` host paragraph record |
| 08_year_full_tuple_plus_next_boundary | `07` + 문단 `0:53`, `0:54` paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/01_pre_year_empty_paras.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/02_year_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/03_year_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/04_year_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/05_year_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/06_year_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/07_year_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/08_year_full_tuple_plus_next_boundary.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage23_generate_year_trend_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 52 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/01_pre_year_empty_paras.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/02_year_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/03_year_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/04_year_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/05_year_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/06_year_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/07_year_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage23_year_trend_probe/08_year_full_tuple_plus_next_boundary.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_pre_year_empty_paras | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 02_year_title_para_record | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 03_year_table_ctrl_header | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 04_year_table_record_with_tail | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 05_year_table_all_cell_headers | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 06_year_table_full_object_with_tail | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 07_year_host_para_plus_table_full_tuple | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 08_year_full_tuple_plus_next_boundary | 파일손상 | 7페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |

판정 포인트:

```text
- 한컴 출력 경계가 "2. 연도별 동향" 이후로 이동하는지
- 문단 0:52 연도별 동향 표가 출력되는지
- 다음 경계인 "□ 업종별 동향(상위 5개)"까지 이동하는지
- 판정 유형이 파일손상, 파일 읽기 오류, 정상 열림 중 무엇인지
- 셀 텍스트가 여전히 셀 영역 위에 걸치는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 현재 가설

Stage 23에서도 우선 확인할 핵심 신호는 같다.

```text
TABLE record + encoded zone tail
```

다만 문단 `0:52`의 연도별 동향 표는 `3행 x 7열`로 이전의 큰 표보다 작고, `treat_as_char=false`, `horz=단`, `vert=문단(54)` 특성을 가진다.

따라서 다음을 분리해서 본다.

```text
1. 표 record tail만으로 경계가 이동하는지
2. full table object가 필요해지는지
3. host paragraph record가 있어야 한컴 파서가 안정되는지
4. 다음 boundary 문단 0:53/0:54가 별도 병목인지
```

## 9. 판정 해석

### 9.1 연도별 동향 표에서도 같은 패턴 재현

다음 variant에서 한컴 출력 경계가 `4페이지 2. 연도별 동향`에서 `7페이지 2. 연도별 동향`까지 이동했다.

```text
04_year_table_record_with_tail
06_year_table_full_object_with_tail
07_year_host_para_plus_table_full_tuple
08_year_full_tuple_plus_next_boundary
```

반대로 다음 variant는 경계를 이동시키지 못했다.

```text
01_pre_year_empty_paras
02_year_title_para_record
03_year_table_ctrl_header
05_year_table_all_cell_headers
```

따라서 작은 `3행 x 7열` 연도별 동향 표에서도 핵심 신호는 동일하다.

```text
TABLE record + encoded zone tail
```

### 9.2 다음 경계

정답 HWP에서 `2. 연도별 동향` 텍스트는 두 번 등장한다.

```text
문단 0:50 "2. 연도별 동향"
문단 0:87 "2. 연도별 동향"
```

Stage 23에서 출력 경계가 이동한 대상은 두 번째 `2. 연도별 동향`이다.

정답 HWP 기준 다음 부근은 다음과 같다.

```text
문단 0:85 빈 문단
문단 0:86 빈 문단
문단 0:87 "2. 연도별 동향"
문단 0:88 빈 문단
문단 0:89 연도별 동향 표, 3행 x 7열
```

Stage 24는 문단 `0:87` 제목과 문단 `0:89` 두 번째 연도별 동향 표를 대상으로 한다.
