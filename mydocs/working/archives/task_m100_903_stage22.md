# Task m100 #903 Stage 22

## 1. 단계 목적

Stage 21에서 문단 `0:29` 로고/배너 묶음 그림을 통과하면 한컴 출력 경계가 3페이지의 다음 블록까지 이동했다.

```text
3페이지 □ 국가별 동향(상위 5개)
```

Stage 22는 이 다음 경계인 문단 `0:43` 제목과 문단 `0:44` 국가별 표를 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 22 산출물:

```text
output/poc/hwpx2hwp/task903/stage22_top_country_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:39 "□ 업종별 동향(상위 5개)"
문단 0:41 업종별 표, 12행 x 10열
문단 0:43 "□ 국가별 동향(상위 5개)"
문단 0:44 국가별 표, 12행 x 10열
문단 0:45 다음 경계 후보 문단
```

Stage 22 공통 기준선은 Stage 21의 `08_logo_group_full_tuple_plus_attachment_title_table` 상태를 재현한다.

즉 다음이 이미 포함된다.

```text
문단 0:29 로고/배너 묶음 그림 full tuple
문단 0:30 별첨 제목 paragraph/table full object
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_top_country_title_para_record | 문단 `0:43` 제목 paragraph record |
| 02_top_country_table_ctrl_header | `01` + 문단 `0:44` 국가별 표 CTRL_HEADER |
| 03_top_country_table_record_with_tail | `01` + 문단 `0:44` 국가별 표 TABLE record + encoded zone tail |
| 04_top_country_table_all_cell_headers | `01` + 문단 `0:44` 국가별 표 전체 cell LIST_HEADER/PARA_HEADER |
| 05_top_country_table_full_object_with_tail | `01` + 문단 `0:44` 국가별 표 full object + encoded zone tail |
| 06_top_country_host_para_plus_table_full_tuple | `01` + 문단 `0:44` host paragraph record + 국가별 표 full object + encoded zone tail |
| 07_top_country_record_plus_next_boundary | `03` + 문단 `0:45` paragraph record |
| 08_top_country_full_tuple_plus_next_boundary | `06` + 문단 `0:45` paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage22_top_country_probe/01_top_country_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/02_top_country_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/03_top_country_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/04_top_country_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/05_top_country_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/06_top_country_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/07_top_country_record_plus_next_boundary.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/08_top_country_full_tuple_plus_next_boundary.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage22_generate_top_country_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 51 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage22_top_country_probe/01_top_country_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/02_top_country_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/03_top_country_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/04_top_country_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/05_top_country_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/06_top_country_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/07_top_country_record_plus_next_boundary.hwp
output/poc/hwpx2hwp/task903/stage22_top_country_probe/08_top_country_full_tuple_plus_next_boundary.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_top_country_title_para_record | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 02_top_country_table_ctrl_header | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 03_top_country_table_record_with_tail | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 04_top_country_table_all_cell_headers | 파일손상 | 3페이지 `□ 국가별 동향(상위 5개)`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 05_top_country_table_full_object_with_tail | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 06_top_country_host_para_plus_table_full_tuple | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 07_top_country_record_plus_next_boundary | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 08_top_country_full_tuple_plus_next_boundary | 파일손상 | 4페이지 `2. 연도별 동향`까지 출력 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |

판정 포인트:

```text
- 한컴 출력 경계가 "□ 국가별 동향(상위 5개)" 이후로 이동하는지
- 국가별 표가 출력되는지
- 판정 유형이 파일손상, 파일 읽기 오류, 정상 열림 중 무엇인지
- Stage 21까지의 로고/배너 묶음 그림과 별첨 제목 표가 유지되는지
- 셀 텍스트가 여전히 셀 영역 위에 걸치는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 현재 가설

Stage 14부터 Stage 21까지 반복된 신호는 다음과 같다.

```text
TABLE record + encoded zone tail
```

이 payload가 들어갈 때 한컴 출력 경계가 다음 표 또는 다음 개체로 이동했다.

Stage 22의 핵심 비교 대상은 다음 variant다.

```text
03_top_country_table_record_with_tail
05_top_country_table_full_object_with_tail
06_top_country_host_para_plus_table_full_tuple
07_top_country_record_plus_next_boundary
08_top_country_full_tuple_plus_next_boundary
```

특히 `03`이 경계를 이동시키고 `05/06/08`이 rhwp-studio 렌더링을 유지한다면, 국가별 표에서도 같은 패턴이 재현된 것으로 본다.

## 9. 다음 판단 기준

Stage 22 판정 결과에서 다음 중 하나를 확인한다.

```text
1. 국가별 표 payload가 경계를 다음 블록으로 이동시키는지
2. 출력 경계는 이동하지만 파일손상/파일 읽기 오류 유형만 바뀌는지
3. 셀 세로 배치 문제와 파일손상 문제가 같은 payload에서 함께 움직이는지
4. 국가별 표 이후의 다음 boundary 문단이 별도 병목인지
```

결과가 Stage 20/21과 같다면 다음 Stage는 문단 `0:45` 이후의 다음 표/본문 boundary를 대상으로 한다.

## 10. 판정 해석

### 10.1 국가별 표에서도 같은 패턴 재현

다음 variant에서 한컴 출력 경계가 `3페이지 □ 국가별 동향(상위 5개)`에서 `4페이지 2. 연도별 동향`까지 이동했다.

```text
03_top_country_table_record_with_tail
05_top_country_table_full_object_with_tail
06_top_country_host_para_plus_table_full_tuple
07_top_country_record_plus_next_boundary
08_top_country_full_tuple_plus_next_boundary
```

반대로 다음 variant는 경계를 이동시키지 못했다.

```text
01_top_country_title_para_record
02_top_country_table_ctrl_header
04_top_country_table_all_cell_headers
```

따라서 국가별 표에서도 핵심 신호는 `TABLE record + encoded zone tail`이다.

### 10.2 rhwp-studio 안정성

모든 Stage 22 variant는 rhwp-studio에서 정상으로 판정되었다.

즉 이 단계의 변경은 rhwp-studio 렌더러를 깨뜨리지 않고 한컴 파서 경계만 이동시키는 payload로 볼 수 있다.

다만 셀 텍스트가 셀 영역 위에 걸치는 세로 배치 문제는 Stage 22에서도 계속 남아 있다.

### 10.3 다음 경계

정답 HWP 기준으로 다음 경계는 다음 부근이다.

```text
문단 0:48 빈 문단
문단 0:49 빈 문단
문단 0:50 "2. 연도별 동향"
문단 0:51 빈 문단
문단 0:52 연도별 동향 표, 3행 x 7열
```

Stage 23은 문단 `0:50` 제목과 문단 `0:52` 연도별 동향 표를 대상으로 한다.
