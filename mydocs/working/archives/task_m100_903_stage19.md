# Task m100 #903 Stage 19

## 1. 단계 목적

Stage 18 판정에서 `TABLE record + encoded zone tail` 계열 payload가 한컴 출력 경계를 움직이는 신호를 확인했다.

```text
03_country_table_record_with_tail
05_country_table_full_object_with_tail
06_country_host_para_plus_table_full_tuple
```

위 variant들은 파일손상 판정은 남았지만 출력 위치가 `< 국가별 동향(억 달러, %) >`에서 `< 지역별 동향(억 달러, %) >`로 이동했고, 처음으로 2페이지까지 렌더링되었다.

Stage 19는 같은 패턴을 다음 표인 `지역별 동향` 표에 반복 적용해서 파일손상 경계가 더 뒤로 이동하는지 확인한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 19 산출물:

```text
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:22 "< 지역별 동향(억 달러, %) >"
문단 0:23 지역별 동향 표, 4행 x 8열
문단 0:24 빈 문단
문단 0:25 지역별 동향 다음 본문 문단
```

Stage 19 공통 기준선은 Stage 18의 `06_country_host_para_plus_table_full_tuple` 상태다.

즉 다음이 이미 포함된다.

```text
문단 0:14 업종별 표 full object with encoded zone tail
문단 0:15 업종별 표 다음 boundary paragraph
문단 0:20 국가별 제목 paragraph record
문단 0:21 국가별 표 host paragraph + full object with encoded zone tail
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_region_title_para_record | 문단 `0:22` 지역별 제목 paragraph record |
| 02_region_table_ctrl_header | `01` + 문단 `0:23` 표 CTRL_HEADER |
| 03_region_table_record_with_tail | `01` + 문단 `0:23` TABLE record + encoded zone tail |
| 04_region_table_all_cell_headers | `01` + 문단 `0:23` 전체 cell LIST_HEADER/PARA_HEADER |
| 05_region_table_full_object_with_tail | `01` + 문단 `0:23` 표 full object + encoded zone tail |
| 06_region_host_para_plus_table_full_tuple | `01` + 문단 `0:23` host paragraph + 표 full object |
| 07_region_record_plus_following_empty_para | `03` + 문단 `0:24` 빈 문단 paragraph record |
| 08_region_full_tuple_plus_following_text_para | `06` + 문단 `0:24` 빈 문단 + 문단 `0:25` 본문 paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/01_region_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/02_region_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/03_region_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/04_region_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/05_region_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/06_region_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/07_region_record_plus_following_empty_para.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/08_region_full_tuple_plus_following_text_para.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage19_generate_region_boundary_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 48 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/01_region_title_para_record.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/02_region_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/03_region_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/04_region_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/05_region_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/06_region_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/07_region_record_plus_following_empty_para.hwp
output/poc/hwpx2hwp/task903/stage19_region_boundary_probe/08_region_full_tuple_plus_following_text_para.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 셀 세로 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_region_title_para_record | 파일손상 | `< 지역별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 다음 표 이전까지 출력됨. 2페이지까지 렌더링 |
| 02_region_table_ctrl_header | 파일손상 | `< 지역별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 다음 표 이전까지 출력됨. 2페이지까지 렌더링 |
| 03_region_table_record_with_tail | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. 다음 표 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 04_region_table_all_cell_headers | 파일손상 | `< 지역별 동향(억 달러, %) >` | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 | 다음 표 이전까지 출력됨. 2페이지까지 렌더링 |
| 05_region_table_full_object_with_tail | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 06_region_host_para_plus_table_full_tuple | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |
| 07_region_record_plus_following_empty_para | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 2페이지 비정상 배치 |  |
| 08_region_full_tuple_plus_following_text_para | 파일손상 | `< 지역별 동향(억 달러, %) >` 다음 표와 문단까지 출력. `"※ 2024년 해외직접투자"` 전에서 중지 | 셀 영역 위에 걸쳐 윗부분 글자가 안 보임. 특정 셀은 셀 가운데 잘 정렬되어 보임 | 정상 |  |

판정 포인트:

```text
- 한컴 파일손상 경계가 지역별 표 이후로 이동하는지
- 문단 0:25 본문이 출력되는지
- 셀 세로 배치 문제가 유지되는지 또는 특정 variant에서 완화되는지
- Stage 17 이후 복원한 '<', '>' 출력이 유지되는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 판정 해석

### 8.1 파일손상 경계

Stage 19에서도 `TABLE record + encoded zone tail` 계열이 가장 강한 신호다.

```text
03_region_table_record_with_tail
05_region_table_full_object_with_tail
06_region_host_para_plus_table_full_tuple
08_region_full_tuple_plus_following_text_para
```

이 variant들은 지역별 표 이후의 다음 표와 문단까지 한컴 출력 경계를 이동시켰다.
특히 `05`, `06`, `08`은 `"※ 2024년 해외직접투자"` 직전까지 출력된다.

반대로 다음 payload는 경계 전진 효과가 약하다.

```text
02_region_table_ctrl_header
04_region_table_all_cell_headers
```

이는 Stage 18과 같은 패턴이다.
즉 이 샘플의 표 단위 손상 경계에서는 `CTRL_HEADER`나 cell header보다 `TABLE record + encoded zone tail`이 핵심 경계 payload다.

### 8.2 rhwp-studio 판정

대부분 정상 렌더링이지만 `07_region_record_plus_following_empty_para`는 2페이지 비정상 배치가 발생했다.

```text
07 = region TABLE record + following empty paragraph
```

이는 `TABLE record`만 복사하고 표 full object는 복사하지 않은 상태에서 다음 빈 문단 record를 섞으면 rhwp layout 쪽에서도 일관성이 깨질 수 있음을 보여준다.
이 케이스는 한컴 호환성만의 문제가 아니라 IR 자체의 조판 입력이 불안정해진 상태다.

따라서 `07`은 "한컴 출력 경계가 이동했으니 유효"가 아니라, 부분 graft 조합이 위험하다는 음성 대조군으로 본다.
다음 단계의 기준선은 `05`, `06`, `08`처럼 표 full object까지 포함하고 rhwp-studio 조판이 정상인 variant를 우선한다.

### 8.3 셀 세로 배치

셀 텍스트가 셀 영역 위에 걸치는 현상은 Stage 19에서도 유지된다.
따라서 파일손상 경계 전진과 셀 세로 배치 문제는 계속 분리해서 다룬다.

## 9. 다음 단계

Stage 20은 `"※ 2024년 해외직접투자"` 직전의 다음 손상 경계를 찾는다.

우선 다음을 확인한다.

```text
1. 정답 HWP에서 `"※ 2024년 해외직접투자"` 문단 index 확인
2. 그 직전의 표/문단 tuple 확인
3. Stage 19 `06` 또는 `08` 기준선 위에 다음 표/문단의 TABLE record + encoded zone tail 패턴을 반복 적용
```

Stage 20 기준선 후보:

```text
Stage 19 06_region_host_para_plus_table_full_tuple
Stage 19 08_region_full_tuple_plus_following_text_para
```

`07`은 rhwp-studio 배치가 비정상이므로 기준선에서 제외한다.
