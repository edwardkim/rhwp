# Task m100 #903 Stage 9

## 1. 단계 목적

Stage 8의 01~06 probe는 모두 한컴 에디터에서 다음 손상 판정을 받았다.

```text
파일을 읽거나 저장하는데 오류가 있습니다.
```

반면 rhwp-studio에서는 모두 정상 렌더링했다.

따라서 Stage 9는 `SectionDef`, 첫 셀 일부, 첫 그림 `CommonObjAttr` 단독 가설을
버리고, 한컴 정답 HWP와 1페이지 첫 표의 record payload를 더 넓은 단위로 맞춰보는
RED/Probe 단계로 진행한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 8 기준 산출물:

```text
output/poc/hwpx2hwp/task903/stage8_core_field_probe/06_section_def_first_cell_picture_common.hwp
```

Stage 9 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/
```

작업지시자 시각 판정용 파일은 반드시 `output/` 아래에 생성한다.

## 3. Stage 8 실패 결론

실패한 가설:

```text
SectionDef CTRL_HEADER(secd) 47B + core field 보강
첫 표 첫 셀 LIST_HEADER 65B 보강
첫 표 첫 그림 CTRL_HEADER(gso) 246B description 보강
위 조합 보강
```

판단:

- 한컴 손상은 단일 record tail 또는 단일 그림 description 누락 때문이 아니다.
- rhwp-studio가 정상 렌더링하므로 IR 구조 자체는 크게 무너지지 않았다.
- 한컴 reader가 첫 페이지 첫 표의 더 넓은 payload 조합을 요구할 가능성이 높다.

## 4. 정답 HWP와 Stage 산출물의 고신호 차이

정답 HWP 첫 표 주변에서 관찰된 차이:

```text
TABLE CTRL_HEADER:
  정답: z_order / instance_id / 일부 flag 값 존재
  Stage: 일부 값 0

TABLE record:
  정답: TABLE sz=24
  Stage: TABLE sz=22 또는 24 probe 실패

LIST_HEADER:
  정답: 첫 표의 셀마다 47B, 65B, 67B 등 가변 tail
  Stage: 대부분 34B, Stage 8은 첫 셀만 65B

PARA_HEADER:
  정답: 다수 24B
  Stage: 다수 22B, Stage 8의 전체 24B 보강 단독 실패

PICTURE:
  정답: 첫 그림 CTRL_HEADER(gso) 246B
  Stage 8: 첫 그림 246B까지 보강했으나 SHAPE_COMPONENT/SC_PICTURE payload는 여전히 차이 존재
```

## 5. Stage 9 variant 계획

생성 위치:

```text
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/
```

### 01_first_table_ctrl_header_from_reference

첫 표의 `raw_ctrl_data`를 정답 HWP 첫 표에서 가져온 값으로 교체한다.

목표:

- `TABLE CTRL_HEADER(tbl )`의 z-order, instance id, prevent/flag 계열 값이 한컴 손상에 영향을 주는지 확인한다.

### 02_first_table_all_cell_list_headers_from_reference

첫 표의 모든 셀에 대해 정답 HWP의 `raw_list_extra`를 셀 순서대로 복사한다.

목표:

- Stage 8에서 첫 셀 65B만 맞춘 것이 부족했는지 확인한다.
- 정답 HWP의 47B/65B/67B 가변 `LIST_HEADER` 패턴이 필요한지 확인한다.

### 03_first_table_table_record_from_reference

첫 표의 `raw_table_record_attr`, `raw_table_record_extra`, `row_sizes`, `border_fill_id`를
정답 HWP 첫 표 기준으로 맞춘다.

목표:

- `TABLE` record 자체의 attr/row payload 조합이 필요한지 확인한다.

### 04_first_table_cell_para_header_from_reference

첫 표 셀 안 문단들의 `raw_header_extra`를 정답 HWP 셀 문단과 가능한 범위에서 순서대로 맞춘다.

목표:

- 셀 내부 문단 `PARA_HEADER 24B`가 단순 zero tail이 아니라 정답의 instance id/예약값을 가져야 하는지 확인한다.

### 05_first_table_structural_payload_bundle

01~04를 모두 적용한다.

목표:

- 첫 표 payload 조합이 한컴 손상을 해소하는지 확인한다.

### 06_first_table_picture_records_from_reference

첫 표 안 그림 2개의 `CommonObjAttr`, `ShapeComponentAttr`, `SC_PICTURE` 관련 필드를
정답 HWP에서 가져온 값으로 맞춘다.

목표:

- Stage 8에서 `CTRL_HEADER(gso)`만 맞춘 것이 부족했는지 확인한다.
- `SHAPE_COMPONENT`와 `SC_PICTURE` payload 차이가 손상 원인인지 확인한다.

## 6. 내부 검증

각 variant 생성 후 다음을 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter task903_stage9_generate_first_table_payload_probe_variants -- --nocapture
cargo test --test hwpx_to_hwp_adapter -- --nocapture
cargo test --test hwpx_roundtrip_integration -- --nocapture
```

레코드 덤프 확인:

```text
cargo run --bin rhwp -- dump-records output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/<variant>.hwp
```

검증 기준:

- rhwp-studio 재로드/렌더링은 정상이어야 한다.
- 페이지 수는 9를 유지해야 한다.
- 1페이지 첫 표와 이미지가 사라지면 실패다.
- 한컴 손상 판정이 사라지는 variant가 나오면 해당 payload 조합을 production 후보로 올린다.

## 7. 작업지시자 판정 항목

판정 파일은 `output/` 아래에 생성한다.

판정 항목:

- 한컴 에디터 파일 손상 판정이 사라지는지
- rhwp-studio에서 렌더링 성공 상태가 유지되는지
- 1페이지 첫 표의 배치/이미지가 유지되는지

판정 기록 형식:

```text
| variant | 한컴 판정 | rhwp-studio 판정 | 비고 |
|---|---|---|---|
| 01_first_table_ctrl_header_from_reference |  |  |  |
| 02_first_table_all_cell_list_headers_from_reference |  |  |  |
| 03_first_table_table_record_from_reference |  |  |  |
| 04_first_table_cell_para_header_from_reference |  |  |  |
| 05_first_table_structural_payload_bundle |  |  |  |
| 06_first_table_picture_records_from_reference |  |  |  |
```

## 8. 구현 결과

승인 후 Stage 9 probe 생성기를 `tests/hwpx_to_hwp_adapter.rs`에 추가했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/01_first_table_ctrl_header_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/02_first_table_all_cell_list_headers_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/03_first_table_table_record_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/04_first_table_cell_para_header_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/05_first_table_structural_payload_bundle.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/06_first_table_picture_records_from_reference.hwp
```

생성 기준:

- 모든 variant는 Stage 8의 `06_section_def_first_cell_picture_common` 보강 상태를 기준선으로 삼았다.
- 그 위에 첫 표 payload만 variant별로 정답 HWP에서 이식했다.
- 내부 재로드 기준으로 6개 모두 페이지 수 9, 첫 표 셀 수 유지, 첫 표 그림 수 유지 확인.

적용 payload:

| variant | 적용 payload |
|---|---|
| 01 | 첫 표 `CTRL_HEADER(tbl )` raw payload |
| 02 | 첫 표 전체 셀 `LIST_HEADER` 계열 필드와 `raw_list_extra` |
| 03 | 첫 표 `TABLE` record attr/row/border/zones/extra |
| 04 | 첫 표 셀 내부 문단 `PARA_HEADER raw_header_extra` |
| 05 | 01~04 structural bundle |
| 06 | 첫 표 그림 2개의 `CommonObjAttr`, `ShapeComponentAttr`, `SC_PICTURE` 계열 필드 |

## 9. 내부 검증 결과

```text
cargo test --test hwpx_to_hwp_adapter task903_stage9_generate_first_table_payload_probe_variants -- --nocapture
=> ok, 1 passed

cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> ok, 40 passed

cargo test --test hwpx_roundtrip_integration -- --nocapture
=> ok, 17 passed
```

## 10. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/01_first_table_ctrl_header_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/02_first_table_all_cell_list_headers_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/03_first_table_table_record_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/04_first_table_cell_para_header_from_reference.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/05_first_table_structural_payload_bundle.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/06_first_table_picture_records_from_reference.hwp
```

판정 항목:

- 한컴 에디터 파일 읽기/저장 오류가 사라지는지
- rhwp-studio 정상 렌더링이 유지되는지
- 1페이지 첫 표 배치와 이미지가 유지되는지

## 11. 작업지시자 판정 결과

| variant | 한컴 판정 | rhwp-studio 판정 | 비고 |
|---|---|---|---|
| 01_first_table_ctrl_header_from_reference | 파일을 읽거나 저장하는데 오류가 있습니다. |  |  |
| 02_first_table_all_cell_list_headers_from_reference | 파일을 읽거나 저장하는데 오류가 있습니다. |  |  |
| 03_first_table_table_record_from_reference | 파일을 읽거나 저장하는데 오류가 있습니다. |  |  |
| 04_first_table_cell_para_header_from_reference | 파일을 읽거나 저장하는데 오류가 있습니다. |  |  |
| 05_first_table_structural_payload_bundle | 파일을 읽거나 저장하는데 오류가 있습니다. |  |  |
| 06_first_table_picture_records_from_reference | 1페이지 첫 표 출력 후 파일손상 |  | 첫 표 그림 record 이식은 한컴 reader 진행 위치를 바꿈 |

## 12. Stage 9 결론

확인된 점:

- 첫 표 `CTRL_HEADER(tbl )`, 전체 셀 `LIST_HEADER`, `TABLE` record, 셀 문단 `PARA_HEADER`를 정답 HWP 기준으로 맞춰도 한컴 파일 읽기/저장 오류는 해소되지 않았다.
- 01~05가 모두 같은 파일 읽기/저장 오류이므로 첫 표 structural payload 단독/조합은 읽기 오류의 주 원인 후보에서 내려도 된다.
- 06은 첫 표 그림 2개의 `CommonObjAttr`, `ShapeComponentAttr`, `SC_PICTURE` 계열 필드를 정답 HWP에서 이식했을 때 한컴이 1페이지 첫 표까지 출력한 뒤 손상 판정을 냈다.

해석:

- 그림 record payload는 한컴 판정 유형을 파일 읽기/저장 오류에서 렌더링 후 파일손상 쪽으로 바꾼다.
- 하지만 첫 표 안 그림 record만 맞추는 것은 충분하지 않다.
- 다음 단계는 문서 전체 범위로 넓히기보다, 먼저 "파일 읽기/저장 오류를 파일손상 단계로 전환시키는 최소 조건"을 분리하는 방향이 합리적이다.
