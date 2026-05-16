# Task m100 #903 Stage 26

## 1. 단계 목적

Stage 25에서는 마지막 업종별 동향 표를 추가해도 한컴 출력이 8페이지에서 멈췄다.

따라서 마지막 페이지 누락의 다음 후보는 문단 `0:97` 마지막 국가별 동향 표다.

Stage 26은 문단 `0:95` 제목, 문단 `0:96` 빈 문단, 문단 `0:97` 마지막 국가별 동향 표를 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 26 산출물:

```text
output/poc/hwpx2hwp/task903/stage26_final_country_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 문단 위치

정답 HWP 기준:

```text
문단 0:95 "□ 국가별 동향(상위 5개)"
문단 0:96 빈 문단
문단 0:97 마지막 국가별 동향 표, 11행 x 7열
문단 0:98 빈 문단
문단 0:99 빈 문단
문단 0:100 "□ 지역별 동향"
문단 0:101 빈 문단
```

Stage 26 공통 기준선은 Stage 25의 `08_final_industry_full_tuple_plus_next_boundary` 상태를 재현한다.

즉 다음이 이미 포함된다.

```text
문단 0:92 "□ 업종별 동향(상위 5개)"
문단 0:93 빈 문단
문단 0:94 마지막 업종별 동향 표 full object + encoded zone tail
문단 0:95 "□ 국가별 동향(상위 5개)"
문단 0:96 빈 문단
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_final_country_table_ctrl_header | 문단 `0:97` 마지막 국가별 표 CTRL_HEADER |
| 02_final_country_table_record_with_tail | 문단 `0:97` 마지막 국가별 표 TABLE record + encoded zone tail |
| 03_final_country_table_all_cell_headers | 문단 `0:97` 마지막 국가별 표 전체 cell LIST_HEADER/PARA_HEADER |
| 04_final_country_table_full_object_with_tail | 문단 `0:97` 마지막 국가별 표 full object + encoded zone tail |
| 05_final_country_host_para_plus_table_full_tuple | `04` + 문단 `0:97` host paragraph record |
| 06_final_country_record_plus_following_paras | `02` + 문단 `0:98`, `0:99`, `0:100`, `0:101` paragraph record |
| 07_final_country_full_tuple_plus_following_paras | `05` + 문단 `0:98`, `0:99`, `0:100`, `0:101` paragraph record |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage26_final_country_probe/01_final_country_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/02_final_country_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/03_final_country_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/04_final_country_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/05_final_country_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/06_final_country_record_plus_following_paras.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/07_final_country_full_tuple_plus_following_paras.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage26_generate_final_country_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 55 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage26_final_country_probe/01_final_country_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/02_final_country_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/03_final_country_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/04_final_country_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/05_final_country_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/06_final_country_record_plus_following_paras.hwp
output/poc/hwpx2hwp/task903/stage26_final_country_probe/07_final_country_full_tuple_plus_following_paras.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 마지막 국가별 표 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_final_country_table_ctrl_header | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 02_final_country_table_record_with_tail | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 03_final_country_table_all_cell_headers | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 04_final_country_table_full_object_with_tail | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 05_final_country_host_para_plus_table_full_tuple | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 06_final_country_record_plus_following_paras | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |
| 07_final_country_full_tuple_plus_following_paras | 정상 | 8페이지 까지만 출력 | 미출력 | 미확인 | 정상 | 9페이지(마지막) 출력되지 않음 |

판정 포인트:

```text
- 한컴 정상 열림 상태가 유지되는지
- 9페이지 마지막이 출력되는지
- 문단 0:97 마지막 국가별 동향 표가 출력되는지
- 문단 0:100 "□ 지역별 동향"까지 이동하는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 현재 가설

Stage 25에서 `0:94` 마지막 업종별 표는 9페이지 출력 복구에 영향을 주지 않았다.

Stage 26에서 마지막 국가별 표를 넣어도 9페이지가 출력되지 않는다면, 남은 가능성은 다음과 같다.

```text
1. 이후 문단/표까지 함께 materialize해야 페이지가 살아난다.
2. 페이지/섹션/lineSeg 관련 정보가 아직 부족하다.
3. 한컴은 정상 열림 상태에서 누락된 하위 블록을 단순히 출력하지 않는다.
```

## 9. 판정 후 해석

Stage 26 모든 variant는 한컴에서 정상 열림 상태를 유지했다.

하지만 모든 variant가 8페이지까지만 출력되고 9페이지는 출력되지 않았다.

따라서 문단 `0:97` 마지막 국가별 동향 표와 뒤따르는 문단 `0:98`~`0:101`은 9페이지 누락의 직접 원인이 아니다.

정답 HWP dump 기준으로 다음 후보는 두 가지다.

```text
1. 문단 0:102 마지막 지역별 동향 표
2. section 1의 첫 문단/표: "참고 / 해외직접투자 개념 ..."
```

특히 section 1 para 0에는 section break, section def, column def, 1x3 참고 표가 함께 존재한다.

Stage 27은 section 0 마지막 표와 section 1 첫 페이지를 직접 대상으로 삼는다.
