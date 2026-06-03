# Task M100-993 Stage 4 계획서

## 1. 목적

Stage 3에서 파일손상은 해소되었지만, 한컴 에디터 기준 1페이지 조직도 표 높이가 정답지보다
커져 다음 페이지로 넘어가는 회귀가 남았다.

Stage 4의 목적은 이 회귀가 표 셀 `LIST_HEADER` tail 13바이트와 셀 내부 `PARA_HEADER`
tail/flag 2바이트 계열 중 어디에서 발생하는지 분리하는 것이다.

## 2. 배경

Stage 3 생성 파일:

```text
output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
```

작업지시자 판정:

```text
- 한컴 에디터에서 정상 열림
- 1페이지 조직도 표 높이가 한컴 정답지보다 커짐
- 커진 표가 다음 페이지로 넘어감
- 1페이지 처음 부분에 정체 불명의 선이 출력됨
```

rhwp `dump-pages` 기준으로는 정답 HWP와 Stage 3 생성 HWP의 1페이지 조직도 표 배치가 동일하다.
따라서 이 문제는 rhwp 내부 IR 조판 차이가 아니라, 한컴 에디터가 HWP5 record를 해석하는
과정에서 드러나는 record contract 차이로 본다.

## 3. 관찰된 후보 축

정답 HWP와 Stage 3 생성 HWP의 HWP5 inventory diff에서 다음 차이가 반복된다.

```text
셀 LIST_HEADER
- 정답: size=47
- 생성: size=34
- 차이: 13 bytes

셀 내부 PARA_HEADER
- 정답: size=24
- 생성: size=22
- 차이: 2 bytes 및 flag byte 차이 가능성
```

1페이지 조직도 표 주변에서도 같은 패턴이 확인된다.

```text
BodyText.Section0#103 PARA_HEADER: 정답 size=24, 생성 size=22
BodyText.Section0#109 LIST_HEADER: 정답 size=47, 생성 size=34
조직도 표 내부 셀 LIST_HEADER/PARA_HEADER에서도 반복
```

## 4. 작업 원칙

이번 단계에서는 새로운 추론을 바로 adapter에 고정하지 않는다.

먼저 정답 HWP의 record 계약을 기준으로 다음 축을 각각 분리한 probe 파일을 생성한다.

```text
1. LIST_HEADER tail 13바이트만 보강
2. PARA_HEADER tail/flag 2바이트 계열만 보강
3. LIST_HEADER tail + PARA_HEADER tail/flag 동시 보강
```

판정 기준은 한컴 에디터의 1페이지 조직도 표 높이와 첫 부분 선 출력 여부다.

## 5. 산출물 위치

```text
output/poc/hwpx2hwp/task993/stage4_cell_header_contract_probe/
```

## 6. Probe 파일

| variant | 한컴 판정 유형 | 1페이지 조직도 표 높이 | 1페이지 첫 부분 선 | 2페이지 첫 1x1 표 배경 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `01_stage3_current.hwp` |  |  |  |  |  | 기준 파일 |
| `02_cell_list_header_tail.hwp` |  |  |  |  |  | 셀 LIST_HEADER tail 13바이트 보강 |
| `03_cell_para_header_tail.hwp` |  |  |  |  |  | 셀 내부 PARA_HEADER tail/flag 보강 |
| `04_cell_list_para_header_tail.hwp` |  |  |  |  |  | 02+03 동시 보강 |
| `05_all_table_cell_list_header_tail.hwp` |  |  |  |  |  | 모든 표 셀 LIST_HEADER tail 보강 |
| `06_all_table_cell_para_header_tail.hwp` |  |  |  |  |  | 모든 표 셀 PARA_HEADER tail/flag 보강 |
| `07_all_table_cell_list_para_header_tail.hwp` |  |  |  |  |  | 05+06 동시 보강 |

## 7. 판정 포인트

```text
1. 한컴 에디터에서 파일이 정상 열리는지
2. 1페이지 조직도 표가 정답지와 같은 높이로 유지되는지
3. 1페이지 처음 부분의 정체 불명 선이 사라지는지
4. 2페이지 첫 1x1 표 배경 문제가 변하는지
5. rhwp-studio 조판이 기존보다 악화되지 않는지
```

## 8. 승인 요청

이 계획으로 Stage 4 probe를 생성하고 한컴 에디터 판정을 요청한다.
