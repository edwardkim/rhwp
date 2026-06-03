# Task m100 #903 Stage 14

## 1. 단계 목적

Stage 13 판정 결과:

```text
01, 02: 첫 표 출력 후 파일손상
03, 04, 05: "분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %)" 출력 후 파일손상
```

따라서 Stage 13에서 효과가 있었던 최소 payload는 두 번째 표의 child header 계열이다.

```text
두 번째 표 TABLE/LIST_HEADER/PARA_HEADER
```

Stage 14는 그 다음 실패 경계인 문단 `0:10`의 4행×10열 표를 대상으로 한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 13 기준 산출물:

```text
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/03_next_table_child_headers.hwp
```

Stage 14 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/
```

작업지시자 시각 판정용 HWP는 반드시 `output/` 아래에 생성한다.

## 3. 현재 실패 경계

한컴 출력이 이동한 마지막 확인 문단:

```text
--- 문단 0.9 ---
텍스트: "< 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) >"
```

다음 문단:

```text
--- 문단 0.10 ---
controls=1
[0] 표: 4행×10열, 셀=33
```

정답 HWP record:

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46
[103] TABLE            lv=2 sz=30
[104] LIST_HEADER      lv=2 sz=47
[105] PARA_HEADER      lv=2 sz=24
```

Stage 13 `03_next_table_child_headers` record:

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46
[103] TABLE            lv=2 sz=28
[104] LIST_HEADER      lv=2 sz=34
[105] PARA_HEADER      lv=2 sz=22
```

## 4. Stage 14 variant 계획

모든 variant는 Stage 13의 `03_next_table_child_headers` 상태를 기준으로 한다.

### 01_chart_table_ctrl_header

문단 `0:10` 표의 `CTRL_HEADER(tbl )` raw payload만 정답 HWP에서 이식한다.

대상:

```text
[102] CTRL_HEADER(tbl)
```

목표:

- 4행×10열 표의 control header payload 불일치가 파일손상 원인인지 확인한다.

### 02_chart_table_record_only

문단 `0:10` 표의 `TABLE` record payload만 정답 HWP에서 이식한다.

대상:

```text
[103] TABLE
```

목표:

- `TABLE sz=28`과 정답 `TABLE sz=30` 차이가 원인인지 확인한다.

### 03_chart_table_first_cell_headers

문단 `0:10` 표의 첫 셀 child header만 정답 HWP에서 이식한다.

대상:

```text
[104] LIST_HEADER
[105] PARA_HEADER
```

목표:

- 첫 셀의 compact `LIST_HEADER/PARA_HEADER`가 원인인지 확인한다.

### 04_chart_table_all_cell_headers

문단 `0:10` 표의 모든 셀 `LIST_HEADER`와 셀 문단 `PARA_HEADER`를 정답 HWP에서 이식한다.

목표:

- 33개 셀 중 특정 셀의 header tail/field payload가 필요한지 확인한다.

### 05_chart_table_structural_bundle

문단 `0:10` 표의 `CTRL_HEADER`, `TABLE`, 전체 셀 `LIST_HEADER`, 전체 셀 문단 `PARA_HEADER`를 함께 이식한다.

목표:

- 4행×10열 표 structural bundle이 출력 위치를 다음 단계로 밀 수 있는지 확인한다.

### 06_chart_table_full_object

문단 `0:10` 표 object 전체를 정답 HWP에서 이식한다.

목표:

- structural header보다 더 넓은 record span 정합성이 필요한지 확인한다.

## 5. 판정 기준

한컴 판정은 다음처럼 구분한다.

```text
파일 읽기/저장 오류:
  "파일을 읽거나 저장하는데 오류가 있습니다."

파일손상:
  일부 내용을 렌더링한 뒤 손상/복구 계열 메시지
```

성공 기준:

1. Stage 13 `03`보다 한컴 출력 위치가 뒤로 이동하는 variant 찾기
2. 파일손상 없이 열리는 variant가 있는지 확인
3. rhwp-studio 정상 렌더링 유지

## 6. 내부 검증

probe 생성 후 다음을 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter task903_stage14_generate_chart_table_boundary_probe_variants -- --nocapture
```

가능하면 전체 adapter 테스트도 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
```

## 7. 작업지시자 판정 항목

판정 파일은 `output/` 아래에 생성한다.

판정 기록 형식:

```text
| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_chart_table_ctrl_header |  |  |  |  |
| 02_chart_table_record_only |  |  |  |  |
| 03_chart_table_first_cell_headers |  |  |  |  |
| 04_chart_table_all_cell_headers |  |  |  |  |
| 05_chart_table_structural_bundle |  |  |  |  |
| 06_chart_table_full_object |  |  |  |  |
```

## 8. 승인 요청

Stage 14는 위 계획에 따라 문단 `0:10` 4행×10열 표 boundary probe 6개를 생성한다.

## 9. 구현 결과

승인 후 `tests/hwpx_to_hwp_adapter.rs`에 Stage 14 probe 생성기를 추가했다.

생성 기준선:

```text
Stage 13 03_next_table_child_headers 재현
  - Stage 8 section core
  - 첫 표 첫 셀 LIST_HEADER 65B
  - 첫 표 그림 0 + 1 full payload
  - 첫 표 structural bundle
  - 두 번째 표 TABLE/LIST_HEADER/PARA_HEADER child header
```

그 위에 문단 `0:10`의 4행×10열 표 payload만 additive하게 적용했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/01_chart_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/02_chart_table_record_only.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/03_chart_table_first_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/04_chart_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/05_chart_table_structural_bundle.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/06_chart_table_full_object.hwp
```

적용 payload:

| variant | 추가 payload |
|---|---|
| 01 | 문단 `0:10` 표 `CTRL_HEADER(tbl )` raw payload |
| 02 | 문단 `0:10` 표 `TABLE` record payload |
| 03 | 문단 `0:10` 표 첫 셀 `LIST_HEADER` + 첫 셀 문단 `PARA_HEADER` |
| 04 | 문단 `0:10` 표 전체 셀 `LIST_HEADER` + 전체 셀 문단 `PARA_HEADER` |
| 05 | 문단 `0:10` 표 `CTRL_HEADER` + `TABLE` + 전체 셀 header structural bundle |
| 06 | 문단 `0:10` 표 object 전체 |

## 10. 내부 검증 결과

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage14_generate_chart_table_boundary_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 43 filtered out
```

전체 adapter test:

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
```

결과:

```text
test result: ok. 44 passed; 0 failed
```

모든 Stage 14 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 11. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/01_chart_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/02_chart_table_record_only.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/03_chart_table_first_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/04_chart_table_all_cell_headers.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/05_chart_table_structural_bundle.hwp
output/poc/hwpx2hwp/task903/stage14_chart_table_boundary_probe/06_chart_table_full_object.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_chart_table_ctrl_header | 파일손상 | 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) 출력 후 | 정상 |  |
| 02_chart_table_record_only | 파일손상 | 업종별 동향(억 달러, %) 까지 더 출력됨 | 정상 | 셀내 텍스트 세로 배치가 위로 올라감 |
| 03_chart_table_first_cell_headers | 파일손상 | 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) 출력 후 | 정상 |  |
| 04_chart_table_all_cell_headers | 파일손상 | 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) 출력 후 | 정상 |  |
| 05_chart_table_structural_bundle | 파일손상 | 업종별 동향(억 달러, %) 까지 더 출력됨 | 정상 | 셀내 텍스트 세로 배치가 위로 올라감 |
| 06_chart_table_full_object | 파일손상 | 업종별 동향(억 달러, %) 까지 더 출력됨 | 정상 | 셀내 텍스트 세로 배치가 위로 올라감 |

## 12. 판정 해석

Stage 14의 목적은 정답지와 동일한 파일을 만드는 것이 아니라, 한컴 에디터의 파일손상 경계가 어느 payload에서 이동하는지 확인하는 것이었다.

판정 결과, 출력 경계를 이동시킨 것은 다음 계열이다.

```text
02_chart_table_record_only
05_chart_table_structural_bundle
06_chart_table_full_object
```

반면 다음 계열은 Stage 13의 출력 경계를 넘기지 못했다.

```text
01_chart_table_ctrl_header
03_chart_table_first_cell_headers
04_chart_table_all_cell_headers
```

따라서 문단 `0:10` 차트 표에서는 `TABLE` 계열 payload가 한컴 파서의 다음 진행 경계에 영향을 준다.

하지만 `02`, `05`, `06`은 모두 셀 안 텍스트 세로 배치가 위로 올라가는 부작용을 보였다. 따라서 이 산출물을 다음 단계의 정상 baseline으로 취급하면 안 된다.

중요한 관찰:

```text
02는 한컴 출력 경계를 "업종별 동향" 문단까지 밀었지만,
그 다음 표 진입 시점에서 파일손상이 발생한다.
```

이는 다음 표 자체가 첫 원인이라는 뜻이 아니라, 앞선 차트 표의 문단/표 record tuple이 불완전한 상태로 한컴 파서에 해석되어 다음 record 진입 시점에서 깨졌을 가능성이 있다.

## 13. Stage 15 전환

Stage 15는 새 HWP variant를 바로 만들지 않는다.

먼저 정답 HWP와 Stage 14 `02_chart_table_record_only` 산출물을 문단 `0:10`부터 `0:14`까지 비교한다.

비교 대상:

```text
문단 0:10  차트 표 host paragraph + table object
문단 0:11  차트 표 뒤 일반 문단
문단 0:12  빈 문단
문단 0:13  "업종별 동향" 제목 문단
문단 0:14  다음 4행×6열 표
```

확인할 항목:

```text
PARA_HEADER size
PARA_TEXT text/cc/control code 위치
PARA_CHAR_SHAPE pos
PARA_LINE_SEG text_start/vpos
CTRL_HEADER(tbl) raw payload
TABLE record size/raw payload
cell LIST_HEADER/PARA_HEADER size
cell PARA_LINE_SEG
```

Stage 15의 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage15_chart_tuple_diff/
```
