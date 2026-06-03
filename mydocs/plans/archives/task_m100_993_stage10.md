# Task M100-993 Stage 10 계획서

## 1. 목적

Stage 9에서 조직도 셀 줄나눔은 개선되었지만, `mel-001`의 2페이지 `2. 인원 현황` 다음 표에서
새로운 잔존 문제가 확인되었다.

```text
현상:
  한컴 에디터에서 2. 인원 현황 다음 표의 병합된 아래쪽 셀 높이가 과도하게 렌더링된다.

결과:
  rhwp-studio에서는 같은 페이지에 들어가는 다음 표가 한컴 에디터에서는 다음 페이지로 밀린다.

중요:
  rhwp-studio에서는 정상 배치된다.
```

Stage 10의 목적은 이 문제를 셀 높이 보정으로 처리하지 않고, 정답 HWP와 Stage 9 생성 HWP의
HWP5 raw record contract 차이로 분리하는 것이다.

## 2. 대상

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

생성본:

```text
output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001.hwp
```

문제 위치:

```text
section: 0
paragraph: 22
table: 8x12, cells=87
위치: "󰊲  인원 현황(’25.11.30. 기준)" 바로 다음 표
```

## 3. 현재 확인

`dump-pages` 기준으로 정답지와 Stage 9 생성본의 rhwp 배치는 같은 형태다.

```text
Table pi=22  8x12  약 630.4x146.1px
다음 FullParagraph pi=23
다음 FullParagraph pi=24
다음 Table pi=25
```

`dump -s 0 -p 22` 기준으로도 표 크기, 셀 높이, 병합 구조, 주요 line segment 값은 IR 수준에서
거의 같은 형태다.

따라서 Stage 10은 IR 렌더링 비교가 아니라 한컴 에디터가 민감하게 보는 HWP5 raw record 차이를
확인한다.

## 4. 비교 축

우선 비교할 record 축:

```text
1. pi=22 table bundle 전체 record 순서/레벨/크기
2. TABLE record payload
3. cell LIST_HEADER payload
4. cell raw_list_extra 13 bytes
5. 병합된 아래쪽 셀의 LIST_HEADER/paragraph bundle
6. 셀 내부 PARA_HEADER payload/tail
7. 셀 내부 PARA_LINE_SEG payload
8. BorderFill 참조값과 해당 DocInfo BORDER_FILL payload
```

특히 Stage 9에서 모든 셀에 보강한 다음 항목이 `pi=22`에서는 과보강인지 확인한다.

```text
cell.list_header_width_ref bit 0
cell.raw_list_extra 13 bytes
```

## 5. 산출물

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/
```

생성 문서:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/pi22_table_bundle_diff.md
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/pi22_cell_payload_diff.md
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/pi22_suspicious_cells.md
```

필요 시 probe HWP:

| file | 목적 |
|---|---|
| `01_stage9_current.hwp` | Stage 9 기준 |
| `02_pi22_without_list_extra.hwp` | pi=22 표에서만 Stage 9 LIST_HEADER extra 제거 |
| `03_pi22_oracle_list_extra.hwp` | pi=22 표에서만 정답지 LIST_HEADER extra 적용 |
| `04_pi22_para_header_tail.hwp` | pi=22 셀 내부 PARA_HEADER tail 후보 |
| `05_pi22_cell_bundle_oracle_projection.hwp` | 병합 아래쪽 셀 후보만 oracle bundle projection |

## 6. 판정표

| file | 한컴 판정 유형 | 2페이지 인원현황 표 높이 | 다음 표 페이지 분리 | 1x1 표 배경 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `01_stage9_current.hwp` |  |  |  |  |  | 기준 |
| `02_pi22_without_list_extra.hwp` |  |  |  |  |  | LIST_HEADER 과보강 확인 |
| `03_pi22_oracle_list_extra.hwp` |  |  |  |  |  | LIST_HEADER 정답지 투영 |
| `04_pi22_para_header_tail.hwp` |  |  |  |  |  | PARA_HEADER tail 후보 |
| `05_pi22_cell_bundle_oracle_projection.hwp` |  |  |  |  |  | 병합 셀 bundle 후보 |

## 7. 성공 기준

```text
1. 한컴 에디터에서 pi=22 표의 병합된 아래쪽 셀 높이가 과도하게 증가하지 않는다.
2. pi=25 표가 rhwp-studio와 같은 페이지 흐름으로 유지된다.
3. Stage 9에서 해결한 조직도 셀 줄나눔 개선이 회귀하지 않는다.
4. 2페이지 첫 1x1 표 gradient 배경이 회귀하지 않는다.
```

## 8. 비목표

이번 단계에서는 다음을 하지 않는다.

```text
- 셀 높이 직접 보정
- 문자열 재분할
- mel-001 전체 표 레이아웃을 한 번에 해결
- Stage 9 LIST_HEADER materialization 전체 롤백
```

## 9. 승인 요청

이 계획으로 Stage 10 raw contract 비교 및 제한 probe 생성을 진행한다.
