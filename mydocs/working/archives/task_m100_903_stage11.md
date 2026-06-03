# Task m100 #903 Stage 11

## 1. 단계 목적

Stage 10 결과는 Stage 9와 동일했다.

```text
01~05: 파일 읽기/저장 오류
06: 1페이지 첫 표 출력 후 파일손상
```

확인된 최소 조건:

- 첫 표 그림 0 full payload 단독: 부족
- 첫 표 그림 1 full payload 단독: 부족
- 첫 표 그림 0 + 1 full payload: 파일 읽기/저장 오류를 파일손상 단계로 전환

하지만 아직 검증하지 않은 조합이 있다.

```text
첫 표 두 그림 full payload + 첫 표 structural payload
```

Stage 9에서는 structural payload 단독을 봤고, Stage 10에서는 두 그림 full payload 단독을 봤다.
Stage 11은 Stage 10의 06을 기준선으로 삼고, 첫 표 structural payload를 다시 additive하게 붙여본다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 10 기준 산출물:

```text
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/06_first_picture_0_and_1_full.hwp
```

Stage 11 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/
```

작업지시자 시각 판정용 파일은 반드시 `output/` 아래에 생성한다.

## 3. Stage 11 variant 계획

모든 variant는 다음 기준선을 공통으로 사용한다.

```text
Stage 8 section core
첫 표 첫 셀 LIST_HEADER 65B
첫 표 그림 0 + 1 full payload
```

### 01_picture_full_plus_table_ctrl_header

기준선에 첫 표 `CTRL_HEADER(tbl )` raw payload를 추가한다.

목표:

- 그림 full payload 이후에는 table ctrl header가 추가 효과를 내는지 확인한다.

### 02_picture_full_plus_all_cell_list_headers

기준선에 첫 표 전체 셀 `LIST_HEADER` payload를 추가한다.

목표:

- 그림 full payload 이후에는 셀 list header 가변 tail이 추가 효과를 내는지 확인한다.

### 03_picture_full_plus_table_record

기준선에 첫 표 `TABLE` record payload를 추가한다.

목표:

- 그림 full payload 이후에는 table record attr/row/border/zones/extra가 추가 효과를 내는지 확인한다.

### 04_picture_full_plus_cell_para_headers

기준선에 첫 표 셀 내부 문단 `PARA_HEADER raw_header_extra`를 추가한다.

목표:

- 그림 full payload 이후에는 셀 문단 header extra가 추가 효과를 내는지 확인한다.

### 05_picture_full_plus_structural_bundle

기준선에 01~04 structural bundle을 모두 추가한다.

목표:

- 첫 표 그림 payload와 첫 표 structural payload 조합이 파일손상을 더 뒤로 미는지 확인한다.

### 06_picture_full_plus_first_table_ctrl_data_records

기준선에 첫 표 셀 문단의 `ctrl_data_records`를 정답 HWP에서 가능한 범위로 복사한다.

목표:

- 그림 control 뒤에 붙는 `CTRL_DATA` 누락 또는 불일치가 파일손상 원인인지 확인한다.

## 4. 판정 기준

한컴 판정은 반드시 다음처럼 구분한다.

```text
파일 읽기/저장 오류:
  "파일을 읽거나 저장하는데 오류가 있습니다."

파일손상:
  문서 일부를 렌더링한 뒤 손상/복구 계열 메시지
```

Stage 11의 성공 기준:

1. Stage 10 06보다 한컴 출력 위치가 더 뒤로 이동하는 variant 찾기
2. 파일손상 없이 열리는 variant가 있는지 확인
3. rhwp-studio 정상 렌더링 유지

## 5. 내부 검증

각 variant 생성 후 다음을 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter task903_stage11_generate_picture_structural_combo_probe_variants -- --nocapture
```

검증 기준:

- rhwp 재로드가 성공해야 한다.
- 페이지 수는 9를 유지해야 한다.
- 첫 표와 이미지가 사라지면 실패다.

## 6. 작업지시자 판정 항목

판정 파일은 `output/` 아래에 생성한다.

판정 기록 형식:

```text
| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_picture_full_plus_table_ctrl_header |  |  |  |  |
| 02_picture_full_plus_all_cell_list_headers |  |  |  |  |
| 03_picture_full_plus_table_record |  |  |  |  |
| 04_picture_full_plus_cell_para_headers |  |  |  |  |
| 05_picture_full_plus_structural_bundle |  |  |  |  |
| 06_picture_full_plus_first_table_ctrl_data_records |  |  |  |  |
```

## 11. 작업지시자 판정 결과

| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_picture_full_plus_table_ctrl_header | 파일손상 | 첫 표 출력 후 | 정상 렌더링 | Stage 10 06과 동일 |
| 02_picture_full_plus_all_cell_list_headers | 파일손상 | 첫 표 출력 후 | 정상 렌더링 | Stage 10 06과 동일 |
| 03_picture_full_plus_table_record | 파일손상 | 첫 표 출력 후 | 정상 렌더링 | Stage 10 06과 동일 |
| 04_picture_full_plus_cell_para_headers | 파일손상 | 첫 표 출력 후 | 정상 렌더링 | Stage 10 06과 동일 |
| 05_picture_full_plus_structural_bundle | 파일손상 | 첫 표 출력 후 | 정상 렌더링 | Stage 10 06과 동일 |
| 06_picture_full_plus_first_table_ctrl_data_records | 파일손상 | 첫 표 출력 후 | 정상 렌더링 | Stage 10 06과 동일 |

## 12. Stage 11 결론

확인된 점:

- Stage 10 06의 기준선에 첫 표 structural payload를 더해도 한컴 출력 위치가 뒤로 이동하지 않았다.
- 첫 표 `CTRL_HEADER`, 셀 `LIST_HEADER`, `TABLE` record, 셀 문단 `PARA_HEADER`, 셀 문단 `ctrl_data_records`는 모두 추가 효과가 없었다.
- rhwp-studio는 모든 variant를 정상 렌더링하므로 IR/rhwp 렌더링 관점의 구조 붕괴는 아니다.

해석:

- 현재 한컴 reader는 첫 표 자체는 통과한다.
- 파일손상은 첫 표 내부 structural payload보다, 첫 표 이후에 이어지는 다음 record/control 경계에서 발생할 가능성이 높다.
- 다음 단계는 첫 표 이후 record/control의 "첫 실패 경계"를 잡는 방향으로 전환해야 한다.

## 7. 승인 요청

Stage 11은 위 계획에 따라 01~06 probe 파일을 생성한다.

## 8. 구현 결과

승인 후 Stage 11 probe 생성기를 `tests/hwpx_to_hwp_adapter.rs`에 추가했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/01_picture_full_plus_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/02_picture_full_plus_all_cell_list_headers.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/03_picture_full_plus_table_record.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/04_picture_full_plus_cell_para_headers.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/05_picture_full_plus_structural_bundle.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/06_picture_full_plus_first_table_ctrl_data_records.hwp
```

생성 기준:

- 모든 variant는 Stage 10의 `06_first_picture_0_and_1_full` 상태를 기준선으로 삼았다.
- 기준선에는 Stage 8 section core, 첫 표 첫 셀 65B, 첫 표 그림 0 + 1 full payload가 포함된다.
- 그 위에 첫 표 structural payload 또는 ctrl_data_records를 additive하게 적용했다.

적용 payload:

| variant | 추가 payload |
|---|---|
| 01 | 첫 표 `CTRL_HEADER(tbl )` raw payload |
| 02 | 첫 표 전체 셀 `LIST_HEADER` 계열 필드와 `raw_list_extra` |
| 03 | 첫 표 `TABLE` record attr/row/border/zones/extra |
| 04 | 첫 표 셀 내부 문단 `PARA_HEADER raw_header_extra` |
| 05 | 01~04 structural bundle |
| 06 | 첫 표 셀 문단 `ctrl_data_records` |

## 9. 내부 검증 결과

```text
cargo test --test hwpx_to_hwp_adapter task903_stage11_generate_picture_structural_combo_probe_variants -- --nocapture
=> ok, 1 passed

cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> ok, 42 passed
```

내부 재로드 기준:

- 6개 variant 모두 rhwp 재로드 성공
- 6개 variant 모두 페이지 수 9 유지
- 6개 variant 모두 첫 표 셀 수 유지
- 6개 variant 모두 첫 표 그림 수 유지

## 10. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/01_picture_full_plus_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/02_picture_full_plus_all_cell_list_headers.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/03_picture_full_plus_table_record.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/04_picture_full_plus_cell_para_headers.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/05_picture_full_plus_structural_bundle.hwp
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/06_picture_full_plus_first_table_ctrl_data_records.hwp
```

판정 기록 형식:

```text
| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_picture_full_plus_table_ctrl_header |  |  |  |  |
| 02_picture_full_plus_all_cell_list_headers |  |  |  |  |
| 03_picture_full_plus_table_record |  |  |  |  |
| 04_picture_full_plus_cell_para_headers |  |  |  |  |
| 05_picture_full_plus_structural_bundle |  |  |  |  |
| 06_picture_full_plus_first_table_ctrl_data_records |  |  |  |  |
```
