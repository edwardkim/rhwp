# Task M100-993 Stage 4 작업 기록

## 1. 목적

Stage 3에서 파일손상은 해소되었지만, 한컴 에디터 기준 2페이지 조직도 표 높이가 정답지보다
커져 다음 페이지로 넘어가는 회귀가 남았다.

Stage 4는 이 현상이 표 셀 `LIST_HEADER` tail 13바이트와 셀 내부 `PARA_HEADER` tail/flag
계약 중 어디에서 발생하는지 분리하기 위한 probe를 생성한다.

작업지시자 판정 후 이 단계의 해석은 정정되었다. 문제의 기준선은 HWP 저장 결과가 아니라,
`mel-001.hwpx`를 rhwp-studio에서 렌더링하는 시점부터 이미 틀어져 있었다.

## 2. 입력

```text
oracle:    samples/hwpx/hancom-hwp/mel-001.hwp
generated: output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp
```

조직도 표 식별 기준:

```text
TABLE rows=24, cols=47
```

Stage 3 추가 분석에서 확인한 반복 차이:

```text
셀 LIST_HEADER: 정답 size=47, 생성 size=34
셀 PARA_HEADER: 정답 size=24, 생성 size=22
```

## 3. 구현한 진단 명령

Stage 4 판정 파일 생성을 위해 다음 진단 명령을 추가했다.

```text
target/debug/rhwp hwp5-cell-header-probe
```

구현 위치:

```text
src/diagnostics/hwp5_cell_header_probe.rs
src/diagnostics/mod.rs
src/main.rs
```

명령 의미:

```text
generated HWP를 기준으로, oracle HWP의 표 셀 LIST_HEADER/PARA_HEADER record 일부를
축별로 graft한 판정용 HWP 파일을 만든다.
```

주의:

```text
이 산출물은 HWPX 저장기 구현 결과가 아니라 contract 분리 판정용 probe다.
```

## 4. 생성 명령

```text
target/debug/rhwp hwp5-cell-header-probe \
  samples/hwpx/hancom-hwp/mel-001.hwp \
  output/poc/hwpx2hwp/task993/stage3_table_common_attr_adapter/mel-001.hwp \
  --out-dir output/poc/hwpx2hwp/task993/stage4_cell_header_contract_probe \
  --section 0
```

## 5. 생성 결과

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage4_cell_header_contract_probe/
```

세부 생성 로그:

```text
output/poc/hwpx2hwp/task993/stage4_cell_header_contract_probe/stage4_generation.md
```

| file | bytes | hash | rhwp reload | list header tail | para header tail | org blocks | table blocks |
|---|---:|---|---|---:|---:|---:|---:|
| `01_stage3_current.hwp` | 160768 | `c8c092d5ddf17304` | ok, pages=20 | 0 | 0 | 0 | 0 |
| `02_cell_list_header_tail.hwp` | 161280 | `5d3a4d797e9dcbd1` | ok, pages=20 | 742 | 0 | 1 | 44 |
| `03_cell_para_header_tail.hwp` | 160768 | `257ea7c7b54022ae` | ok, pages=20 | 0 | 744 | 1 | 44 |
| `04_cell_list_para_header_tail.hwp` | 161280 | `5c69c5c08f656000` | ok, pages=20 | 742 | 744 | 1 | 44 |
| `05_all_table_cell_list_header_tail.hwp` | 161792 | `33f2a5222f87dcb6` | ok, pages=20 | 1272 | 0 | 1 | 44 |
| `06_all_table_cell_para_header_tail.hwp` | 161280 | `44547adc94443f3f` | ok, pages=20 | 0 | 1464 | 1 | 44 |
| `07_all_table_cell_list_para_header_tail.hwp` | 162304 | `fa672f575c1d1c62` | ok, pages=20 | 1272 | 1464 | 1 | 44 |

## 6. 판정표

작업지시자 판정 결과:

| variant | 한컴 판정 유형 | 2페이지 조직도 표 높이 | 1페이지 첫 부분 선 | 2페이지 첫 1x1 표 배경 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `01_stage3_current.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 기준 파일 |
| `02_cell_list_header_tail.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 셀 LIST_HEADER tail 13바이트 보강 |
| `03_cell_para_header_tail.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 셀 내부 PARA_HEADER tail/flag 보강 |
| `04_cell_list_para_header_tail.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 02+03 동시 보강 |
| `05_all_table_cell_list_header_tail.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 모든 표 셀 LIST_HEADER tail 보강 |
| `06_all_table_cell_para_header_tail.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 모든 표 셀 PARA_HEADER tail/flag 보강 |
| `07_all_table_cell_list_para_header_tail.hwp` | 정상 | 비정상 | 존재 | 실패 | 2페이지 조직도 셀내 텍스트 배치 실패 | 05+06 동시 보강 |

## 6.1 판정 해석 정정

Stage 4는 HWP5 cell header tail 보강으로 문제를 해결하지 못했다. 더 중요한 관찰은 다음이다.

```text
mel-001.hwpx를 rhwp-studio에서 직접 열어도 2페이지 조직도 셀 내부 텍스트 배치가 틀린다.
따라서 현재 HWP 저장 결과가 동일하게 틀려 보이는 것은, 잘못 조판된 IR 상태를 저장하고
다시 불러온 결과로 해석해야 한다.
```

즉, Stage 4의 probe 결과는 다음 결론을 준다.

```text
1. 조직도 표 높이/셀 내부 텍스트 배치 문제는 LIST_HEADER tail 13바이트 단독 문제가 아니다.
2. 조직도 표 높이/셀 내부 텍스트 배치 문제는 PARA_HEADER tail/flag 단독 문제가 아니다.
3. 모든 표 셀에 같은 tail 보강을 적용해도 개선되지 않는다.
4. HWPX -> IR 렌더링 기준선 자체가 틀려 있으므로, HWP 저장 계약을 더 만지기 전에
   rhwp-studio 렌더러를 먼저 고쳐야 한다.
```

## 6.2 추가 확인

`dump-pages` 기준으로는 `mel-001.hwpx`, 정답 HWP, Stage 3 생성 HWP의 2페이지 고수준 배치가
비슷하게 보인다. 하지만 시각 판정에서는 조직도 셀 내부 텍스트 배치가 실패한다.

작업지시자 추가 확인으로 더 직접적인 원인 후보가 확인되었다.

```text
- 한컴 에디터: 해당 조직도 셀 내부 글자 크기 8.0 pt
- rhwp-studio: 해당 조직도 셀 내부 글자 크기 10.0 pt
```

따라서 다음 단계는 문단/표의 외곽 y 좌표가 아니라, 표 셀 내부 텍스트의 글자모양 해석을
우선 비교해야 한다. line box, baseline, vertical align, lineSeg vpos는 글자 크기 해석이
정상화된 뒤 후속으로 확인한다.

현재 1차 원인 후보:

```text
HWPX 조직도 셀 텍스트가 참조하는 8pt CharShape 또는 스타일 상속값을 rhwp-studio가 적용하지 못하고,
기본 10pt 글자 크기로 fallback한다.
```

작업지시자 추가 관찰:

```text
셀 세로 방향 정렬에서 위쪽(top) 처리가 적용되지 않는 것으로 보인다.
```

따라서 Stage 5에서는 글자 크기 해석과 셀 vertical align 적용을 함께 추적한다. 두 문제는
독립일 수도 있고, 같은 셀 내부 layout 경로에서 동시에 발생할 수도 있다.

참고 관찰:

```text
- mel-001.hwpx rhwp reload pages=32
- 정답 HWP rhwp reload pages=20
- Stage 3 생성 HWP rhwp reload pages=20
- 2페이지 조직도 표는 rows=24, cols=47로 식별된다.
- HWPX dump에서는 해당 표 page_break가 CellBreak로 보이고, 정답 HWP에서는 RowBreak로 보인다.
```

단, 이 단계에서는 위 차이를 원인으로 확정하지 않는다. 현재 확정된 것은 rhwp-studio의
HWPX 렌더링 기준선이 먼저 틀렸다는 점이다.

## 7. 실행한 검증

```text
cargo fmt
cargo fmt --check
cargo build
target/debug/rhwp hwp5-cell-header-probe ...
```

검증 결과:

```text
- cargo fmt --check 통과
- cargo build 통과
- Stage 4 probe 7개 생성
- 7개 파일 모두 rhwp reload pages=20
```

## 8. 다음 단계

Stage 5는 HWP 저장기 probe가 아니라 rhwp-studio 렌더러 기준선 수정으로 전환한다.

목표:

```text
mel-001.hwpx를 rhwp-studio에서 열었을 때 2페이지 조직도 셀 내부 텍스트 배치가
정답 HWP/PDF와 같아지도록 한다.
```

Stage 5의 1차 추적 축:

```text
조직도 셀 내부 텍스트의 글자 크기가 한컴 기준 8.0pt인데 rhwp-studio에서 10.0pt로 처리되는
CharShape/Style 해석 경로를 찾고 수정한다.
```

Stage 5의 2차 추적 축:

```text
조직도 셀의 세로 방향 정렬값이 top인데 rhwp-studio 셀 내부 layout에서 위쪽 배치로 적용되지
않는 경로를 찾고 수정한다.
```

그 후에만 HWP 저장 결과의 표 배치, 1x1 표 배경, 파일손상 여부를 다시 판정한다.
