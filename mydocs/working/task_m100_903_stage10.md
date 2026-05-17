# Task m100 #903 Stage 10

## 1. 재고 배경

Stage 9 판정은 두 종류를 구분해야 한다.

```text
01~05: 파일을 읽거나 저장하는데 오류가 있습니다.
06: 1페이지 첫 표 출력 후 파일손상
```

따라서 Stage 10을 바로 문서 전체 payload 이식으로 넓히면 안 된다.
우선 목표는 "한컴 파일 읽기/저장 오류"를 "렌더링 후 파일손상" 단계로 바꾼
최소 조건을 찾는 것이다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 9 비교 기준:

```text
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/05_first_table_structural_payload_bundle.hwp
output/poc/hwpx2hwp/task903/stage9_first_table_payload_probe/06_first_table_picture_records_from_reference.hwp
```

Stage 10 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/
```

작업지시자 시각 판정용 파일은 반드시 `output/` 아래에 생성한다.

## 3. Stage 9 해석 수정

배제 가능한 것:

- 첫 표 `CTRL_HEADER(tbl )` 단독
- 첫 표 전체 셀 `LIST_HEADER` 단독
- 첫 표 `TABLE` record 단독
- 첫 표 셀 문단 `PARA_HEADER` 단독
- 위 structural payload bundle

아직 배제하면 안 되는 것:

- 첫 표 그림 2개 중 어느 그림 record가 읽기 오류를 파일손상 단계로 바꾸는지
- 그림 record 내부의 어느 record 계층이 영향을 주는지
- `CommonObjAttr`, `ShapeComponentAttr`, `SC_PICTURE` 중 어느 부분이 최소 조건인지

## 4. Stage 10 variant 계획

생성 위치:

```text
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/
```

### 01_first_picture_0_common_only

첫 표의 첫 번째 그림에 대해서만 정답 HWP의 `CommonObjAttr`를 이식한다.

목표:

- `CTRL_HEADER(gso)` 계층만으로 파일 읽기/저장 오류가 변화하는지 확인한다.

### 02_first_picture_0_shape_component_only

첫 표의 첫 번째 그림에 대해서만 정답 HWP의 `ShapeComponentAttr`를 이식한다.

목표:

- `SHAPE_COMPONENT` 계층만으로 파일 읽기/저장 오류가 변화하는지 확인한다.

### 03_first_picture_0_sc_picture_only

첫 표의 첫 번째 그림에 대해서만 정답 HWP의 `SC_PICTURE` 계열 필드를 이식한다.

목표:

- `SHAPE_COMPONENT_PICTURE` payload만으로 파일 읽기/저장 오류가 변화하는지 확인한다.

### 04_first_picture_0_full

첫 표의 첫 번째 그림 record 전체를 정답 HWP에서 이식한다.

목표:

- Stage 9의 06 변화가 첫 번째 그림만으로 재현되는지 확인한다.

### 05_first_picture_1_full

첫 표의 두 번째 그림 record 전체를 정답 HWP에서 이식한다.

목표:

- Stage 9의 06 변화가 두 번째 그림만으로 재현되는지 확인한다.

### 06_first_picture_0_and_1_full

첫 표의 두 그림 record 전체를 정답 HWP에서 이식한다.

목표:

- Stage 9의 06을 재현하는 control sample로 고정한다.
- 04/05와 비교해 둘 중 하나인지, 둘 다 필요한지 확인한다.

## 5. 판정 기준

한컴 판정은 반드시 다음처럼 구분해서 기록한다.

```text
파일 읽기/저장 오류:
  "파일을 읽거나 저장하는데 오류가 있습니다."

파일손상:
  문서 일부를 렌더링한 뒤 손상/복구 계열 메시지
```

Stage 10의 성공 기준은 "완전 정상"이 아니다.
우선순위는 다음과 같다.

1. 파일 읽기/저장 오류가 사라지고 파일손상 단계로 이동하는 최소 조건 찾기
2. 한컴이 출력하는 마지막 위치 기록
3. rhwp-studio 정상 렌더링 유지

## 6. 내부 검증

각 variant 생성 후 다음을 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter task903_stage10_generate_minimal_read_error_probe_variants -- --nocapture
```

검증 기준:

- rhwp 재로드가 성공해야 한다.
- 페이지 수는 9를 유지해야 한다.
- 첫 표와 이미지가 사라지면 실패다.

## 7. 작업지시자 판정 항목

판정 파일은 `output/` 아래에 생성한다.

판정 기록 형식:

```text
| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_first_picture_0_common_only |  |  |  |  |
| 02_first_picture_0_shape_component_only |  |  |  |  |
| 03_first_picture_0_sc_picture_only |  |  |  |  |
| 04_first_picture_0_full |  |  |  |  |
| 05_first_picture_1_full |  |  |  |  |
| 06_first_picture_0_and_1_full |  |  |  |  |
```

## 12. 작업지시자 판정 결과

Stage 9와 동일한 결과가 확인되었다.

| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_first_picture_0_common_only | 파일 읽기/저장 오류 | 없음 |  | Stage 9 01~05와 동일 |
| 02_first_picture_0_shape_component_only | 파일 읽기/저장 오류 | 없음 |  | Stage 9 01~05와 동일 |
| 03_first_picture_0_sc_picture_only | 파일 읽기/저장 오류 | 없음 |  | Stage 9 01~05와 동일 |
| 04_first_picture_0_full | 파일 읽기/저장 오류 | 없음 |  | 첫 번째 그림 full payload만으로는 부족 |
| 05_first_picture_1_full | 파일 읽기/저장 오류 | 없음 |  | 두 번째 그림 full payload만으로는 부족 |
| 06_first_picture_0_and_1_full | 파일손상 | 1페이지 첫 표 출력 후 |  | Stage 9 06과 동일 |

## 13. Stage 10 결론

확인된 점:

- 첫 표 그림 0 또는 그림 1 단독 full payload 이식으로는 파일 읽기/저장 오류가 해소되지 않는다.
- 첫 표 그림 0과 1을 모두 full payload로 맞췄을 때만 파일 읽기/저장 오류가 파일손상 단계로 이동한다.
- 따라서 Stage 9의 06 변화는 특정 한 그림 하나가 아니라, 첫 표 안 두 그림 record payload 조합에 의해 발생한다.

해석:

- 첫 표의 두 그림 record payload 조합은 한컴 reader가 첫 표까지 진행하기 위한 필요 조건에 가깝다.
- 하지만 이 조합은 충분 조건이 아니다. 이후 파일손상은 첫 표 이후의 다음 record 또는 다음 control group에서 발생한다.
- 다음 단계는 Stage 10의 06을 기준선으로 삼고, "첫 표 이후 바로 나오는 record/control"을 좁혀야 한다.

## 8. 승인 요청

Stage 10은 위 재고안에 따라 01~06 probe 파일을 생성한다.

## 9. 구현 결과

승인 후 Stage 10 probe 생성기를 `tests/hwpx_to_hwp_adapter.rs`에 추가했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/01_first_picture_0_common_only.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/02_first_picture_0_shape_component_only.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/03_first_picture_0_sc_picture_only.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/04_first_picture_0_full.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/05_first_picture_1_full.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/06_first_picture_0_and_1_full.hwp
```

생성 기준:

- Stage 8의 `section core + first cell 65B` 보강 상태를 기준선으로 사용했다.
- Stage 9와 달리 first picture common을 base에 넣지 않았다.
- 각 variant가 명시한 첫 표 그림 record 계층만 정답 HWP에서 이식했다.

적용 payload:

| variant | 적용 payload |
|---|---|
| 01 | 첫 표 그림 0의 `CommonObjAttr` |
| 02 | 첫 표 그림 0의 `ShapeComponentAttr` |
| 03 | 첫 표 그림 0의 `SC_PICTURE` 계열 필드 |
| 04 | 첫 표 그림 0의 full picture record payload |
| 05 | 첫 표 그림 1의 full picture record payload |
| 06 | 첫 표 그림 0, 1의 full picture record payload |

## 10. 내부 검증 결과

```text
cargo test --test hwpx_to_hwp_adapter task903_stage10_generate_minimal_read_error_probe_variants -- --nocapture
=> ok, 1 passed

cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> ok, 41 passed
```

내부 재로드 기준:

- 6개 variant 모두 rhwp 재로드 성공
- 6개 variant 모두 페이지 수 9 유지
- 6개 variant 모두 첫 표 셀 수 유지
- 6개 variant 모두 첫 표 그림 수 유지

## 11. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/01_first_picture_0_common_only.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/02_first_picture_0_shape_component_only.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/03_first_picture_0_sc_picture_only.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/04_first_picture_0_full.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/05_first_picture_1_full.hwp
output/poc/hwpx2hwp/task903/stage10_minimal_read_error_probe/06_first_picture_0_and_1_full.hwp
```

판정 기록 형식:

```text
| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_first_picture_0_common_only |  |  |  |  |
| 02_first_picture_0_shape_component_only |  |  |  |  |
| 03_first_picture_0_sc_picture_only |  |  |  |  |
| 04_first_picture_0_full |  |  |  |  |
| 05_first_picture_1_full |  |  |  |  |
| 06_first_picture_0_and_1_full |  |  |  |  |
```
