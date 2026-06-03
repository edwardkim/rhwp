# Task m100 #903 Stage 25

## 1. 단계 목적

Stage 24에서 두 번째 연도별 동향 표의 `TABLE record + encoded zone tail`이 들어가면 한컴 에디터에서 파일손상 없이 정상 열림 상태가 되었다.

다만 한컴 출력은 8페이지까지만 표시되고, 9페이지 마지막은 출력되지 않았다.

Stage 25는 마지막 페이지 진입부인 문단 `0:92` 제목과 문단 `0:94` 마지막 업종별 동향 표를 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 25 산출물:

```text
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:92 "□ 업종별 동향(상위 5개)"
문단 0:93 빈 문단
문단 0:94 마지막 업종별 동향 표, 11행 x 7열
문단 0:95 "□ 국가별 동향(상위 5개)"
문단 0:96 빈 문단
문단 0:97 마지막 국가별 동향 표, 11행 x 7열
```

Stage 25 공통 기준선은 Stage 24의 `08_second_year_full_tuple_plus_next_boundary` 상태를 재현한다.

즉 다음이 이미 포함된다.

```text
문단 0:87 "2. 연도별 동향"
문단 0:89 두 번째 연도별 동향 표 full object + encoded zone tail
문단 0:90 빈 문단
문단 0:91 빈 문단
문단 0:92 "□ 업종별 동향(상위 5개)"
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_final_industry_blank_para | 문단 `0:93` 빈 문단 paragraph record |
| 02_final_industry_table_ctrl_header | `01` + 문단 `0:94` 마지막 업종별 표 CTRL_HEADER |
| 03_final_industry_table_record_with_tail | `01` + 문단 `0:94` 마지막 업종별 표 TABLE record + encoded zone tail |
| 04_final_industry_table_all_cell_headers | `01` + 문단 `0:94` 마지막 업종별 표 전체 cell LIST_HEADER/PARA_HEADER |
| 05_final_industry_table_full_object_with_tail | `01` + 문단 `0:94` 마지막 업종별 표 full object + encoded zone tail |
| 06_final_industry_host_para_plus_table_full_tuple | `05` + 문단 `0:94` host paragraph record |
| 07_final_industry_record_plus_next_boundary | `03` + 문단 `0:95`, `0:96` paragraph record |
| 08_final_industry_full_tuple_plus_next_boundary | `06` + 문단 `0:95`, `0:96` paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/01_final_industry_blank_para.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/02_final_industry_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/03_final_industry_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/04_final_industry_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/05_final_industry_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/06_final_industry_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/07_final_industry_record_plus_next_boundary.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/08_final_industry_full_tuple_plus_next_boundary.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage25_generate_final_industry_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 54 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/01_final_industry_blank_para.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/02_final_industry_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/03_final_industry_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/04_final_industry_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/05_final_industry_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/06_final_industry_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/07_final_industry_record_plus_next_boundary.hwp
output/poc/hwpx2hwp/task903/stage25_final_industry_probe/08_final_industry_full_tuple_plus_next_boundary.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_final_industry_blank_para | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 02_final_industry_table_ctrl_header | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 03_final_industry_table_record_with_tail | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 04_final_industry_table_all_cell_headers | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 05_final_industry_table_full_object_with_tail | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 06_final_industry_host_para_plus_table_full_tuple | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 07_final_industry_record_plus_next_boundary | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |
| 08_final_industry_full_tuple_plus_next_boundary | 정상 | 8페이지까지만 출력 | 출력되지 않음 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 9페이지 마지막 출력되지 않음 |

판정 포인트:

```text
- 한컴 정상 열림 상태가 유지되는지
- 9페이지 마지막이 출력되는지
- 문단 0:94 마지막 업종별 동향 표가 출력되는지
- 다음 경계인 "□ 국가별 동향(상위 5개)"까지 이동하는지
- 셀 텍스트가 여전히 셀 영역 위에 걸치는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 현재 가설

Stage 24에서 파일손상은 사실상 해소되었고, Stage 25는 문서 끝부분 누락을 복구하는 단계다.

확인할 핵심은 다음이다.

```text
1. 마지막 업종별 표 full object만으로 9페이지가 출력되는지
2. TABLE record + encoded zone tail만으로도 9페이지 출력이 회복되는지
3. 다음 국가별 표 경계가 별도 병목으로 남는지
```

## 9. 판정 해석

### 9.1 파일손상은 계속 해소 상태

Stage 25의 모든 variant가 한컴 에디터에서 정상으로 열렸다.

따라서 Stage 24에서 얻은 파일손상 해소 조건은 Stage 25에서도 유지된다.

```text
두 번째 연도별 표까지 TABLE record + encoded zone tail이 보존되면 파일손상은 사라진다.
```

### 9.2 마지막 페이지 누락은 업종별 표 문제가 아님

문단 `0:94` 마지막 업종별 동향 표를 full object로 넣어도 9페이지는 출력되지 않았다.

```text
05_final_industry_table_full_object_with_tail
06_final_industry_host_para_plus_table_full_tuple
08_final_industry_full_tuple_plus_next_boundary
```

위 variant들도 모두 8페이지까지만 출력된다.

따라서 마지막 페이지 누락의 직접 원인은 문단 `0:94` 업종별 표가 아니다.

### 9.3 다음 경계

Stage 25의 `07/08`은 문단 `0:95`, `0:96`까지 paragraph record를 넣었지만 여전히 9페이지가 출력되지 않았다.

다음 후보는 문단 `0:97` 마지막 국가별 동향 표다.

정답 HWP 기준 마지막 페이지 부근:

```text
문단 0:95 "□ 국가별 동향(상위 5개)"
문단 0:96 빈 문단
문단 0:97 마지막 국가별 동향 표, 11행 x 7열
문단 0:98 빈 문단
문단 0:99 빈 문단
문단 0:100 "□ 지역별 동향"
문단 0:101 빈 문단
```

Stage 26은 문단 `0:97` 마지막 국가별 동향 표를 대상으로 한다.
