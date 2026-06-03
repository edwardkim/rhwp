# Task m100 #903 Stage 27

## 1. 단계 목적

Stage 26에서는 마지막 국가별 동향 표와 뒤따르는 문단을 보강해도 한컴이 정상 열림 상태에서 8페이지만 출력했다.

정답 HWP dump 기준으로 다음 후보는 다음 두 지점이다.

```text
문단 0:102 마지막 지역별 동향 표
section 1 para 0 "참고 / 해외직접투자 개념 ..." 페이지
```

Stage 27은 이 두 지점을 직접 probe한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 27 산출물:

```text
output/poc/hwpx2hwp/task903/stage27_section1_probe/
```

작업지시자 시각 판정용 파일은 프로젝트 규칙에 따라 `output/` 아래에 생성한다.

## 3. 관찰

Stage 26 출력물의 `section 1:0`은 내부 dump 기준으로 section def, column def, 참고 표를 가진다.

하지만 한컴에서는 9페이지가 표시되지 않는다.

정답 HWP와 Stage 26 출력물의 눈에 띄는 차이는 마지막 구간 표들의 TABLE attr에 남아 있는 `0x04000000` 계열 비트다.

따라서 Stage 27은 다음을 나누어 본다.

```text
1. 문단 0:102 마지막 지역별 표 TABLE record/full object를 정답지로 치환
2. section 1 para 0 참고 표 TABLE record/full object를 정답지로 치환
3. section 1 para 0 전체 문단 tuple을 정답지로 치환
4. 마지막 지역별 표와 section 1 보강을 함께 적용
```

## 4. Variant

| variant | 적용 payload |
|---|---|
| 01_final_region_table_record_with_tail | 문단 `0:102` 마지막 지역별 표 TABLE record + encoded zone tail |
| 02_final_region_table_full_object_with_tail | 문단 `0:102` 마지막 지역별 표 full object + encoded zone tail |
| 03_final_region_host_para_plus_table_full_tuple | `02` + 문단 `0:102` host paragraph record |
| 04_section1_para0_record_without_controls | `section 1:0` paragraph record만 정답지로 복사, controls는 유지 |
| 05_section1_table_record_with_tail | `section 1:0` 참고 표 TABLE record + encoded zone tail |
| 06_section1_table_full_object_with_tail | `section 1:0` 참고 표 full object + encoded zone tail |
| 07_section1_full_para0_tuple | `section 1:0` 전체 paragraph tuple을 정답지로 복사 |
| 08_final_region_full_plus_section1_table_record | `02` + `05` |
| 09_final_region_full_plus_section1_full_para0 | `02` + `07` |

## 5. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage27_section1_probe/01_final_region_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/02_final_region_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/03_final_region_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/04_section1_para0_record_without_controls.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/05_section1_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/06_section1_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/07_section1_full_para0_tuple.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/08_final_region_full_plus_section1_table_record.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/09_final_region_full_plus_section1_full_para0.hwp
```

## 6. 내부 검증

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage27_generate_section1_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 56 filtered out
```

모든 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage27_section1_probe/01_final_region_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/02_final_region_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/03_final_region_host_para_plus_table_full_tuple.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/04_section1_para0_record_without_controls.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/05_section1_table_record_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/06_section1_table_full_object_with_tail.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/07_section1_full_para0_tuple.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/08_final_region_full_plus_section1_table_record.hwp
output/poc/hwpx2hwp/task903/stage27_section1_probe/09_final_region_full_plus_section1_full_para0.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 페이지 | 마지막 페이지 출력 | 참고 표 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_final_region_table_record_with_tail | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 02_final_region_table_full_object_with_tail | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 03_final_region_host_para_plus_table_full_tuple | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 04_section1_para0_record_without_controls | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 05_section1_table_record_with_tail | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 06_section1_table_full_object_with_tail | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 07_section1_full_para0_tuple | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 08_final_region_full_plus_section1_table_record | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |
| 09_final_region_full_plus_section1_full_para0 | 정상 | 8페이지 까지만 출력 | 미출력 | 미출력 | 미기록 | 이전 stage와 동일하게 마지막 페이지가 출력되지 않음 |

판정 포인트:

```text
- 한컴 정상 열림 상태가 유지되는지
- 9페이지가 출력되는지
- 마지막 지역별 표와 section 1 참고 표 중 어느 쪽에서 출력 변화가 생기는지
- 셀 세로 배치가 여전히 위에 걸치는지
- rhwp-studio 정상 렌더링을 유지하는지
```

## 8. 현재 가설

Stage 27에서 마지막 지역별 표 variants만으로 9페이지가 살아나면, 마지막 누락 원인은 section 0의 마지막 table record/full object 매핑이다.

section 1 variants만으로 9페이지가 살아나면, section break 이후 첫 문단/참고 표 tuple 매핑이 원인이다.

둘을 함께 적용한 `08`, `09`에서만 살아나면, section 0 마지막 표와 section 1 첫 페이지 사이의 경계 조건이 한컴 호환성 조건이다.

## 9. 판정 후 해석

Stage 27의 모든 variant도 Stage 26과 동일하게 마지막 페이지가 출력되지 않았다.

따라서 마지막 페이지 누락은 다음 payload 단위에서는 해결되지 않는다.

```text
- 문단 0:102 마지막 지역별 표 TABLE record/full object
- section 1:0 paragraph record
- section 1:0 참고 표 TABLE record/full object
- section 1:0 full paragraph tuple
```

중요한 점은 rhwp 내부 재로드에서는 모든 variant가 9페이지를 유지한다는 것이다.

즉 IR 모델과 rhwp 파서는 `section 1`을 보고 있지만, 한컴 에디터의 표시/조판 경로에서는 마지막 section/page가 탈락한다.

다음 단계는 paragraph/table 단위가 아니라 HWP 저장 컨테이너와 BodyText section stream 단위로 비교한다.

Stage 28 후보:

```text
1. 정답 HWP와 생성 HWP의 BodyText/Section0, BodyText/Section1 stream 크기/hash 비교
2. Section1 stream을 정답지에서 graft했을 때 한컴 마지막 페이지가 살아나는지 확인
3. Section0 stream만 정답지에서 graft했을 때 한컴 마지막 페이지가 살아나는지 확인
4. DocInfo만 생성본/정답지를 교차했을 때 section 1 표시 여부가 바뀌는지 확인
```
