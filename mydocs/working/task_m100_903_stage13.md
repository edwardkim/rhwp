# Task m100 #903 Stage 13

## 1. 단계 목적

Stage 12 결론:

```text
첫 표 내부가 아니라 첫 표 종료 직후의 다음 record/control 경계가 현재 실패 후보
```

Stage 13은 Stage 11에서 한컴이 "첫 표 출력 후 파일손상"까지 진행한 기준선을 유지하고, 첫 표 다음 record 구간만 additive하게 이식한다.

이 단계의 목표는 다음 둘 중 어느 쪽이 한컴 파일손상을 유발하는지 분리하는 것이다.

```text
첫 표 직후 최상위 문단 header 불일치
두 번째 표 시작/child header 불일치
```

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

기준 산출물:

```text
output/poc/hwpx2hwp/task903/stage11_picture_structural_combo_probe/05_picture_full_plus_structural_bundle.hwp
```

비교 문서:

```text
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/first_table_boundary_report.md
output/poc/hwpx2hwp/task903/stage12_first_failure_boundary/boundary_diff.md
```

Stage 13 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/
```

작업지시자 시각 판정용 HWP는 반드시 `output/` 아래에 생성한다.

## 3. Stage 13 variant 계획

모든 variant는 Stage 11의 `05_picture_full_plus_structural_bundle` 상태를 기준으로 한다.

### 01_post_first_table_top_level_para_headers

정답 HWP에서 첫 표 직후 최상위 `PARA_HEADER` 3개를 이식한다.

대상:

```text
[37] PARA_HEADER lv=0 sz=24
[40] PARA_HEADER lv=0 sz=24
[43] PARA_HEADER lv=0 sz=24
```

목표:

- compact `PARA_HEADER sz=22`가 첫 표 뒤에서 한컴 파일손상을 유발하는지 확인한다.

### 02_next_table_ctrl_header

정답 HWP에서 두 번째 표 시작 `CTRL_HEADER(tbl )` payload를 이식한다.

대상:

```text
[47] CTRL_HEADER(tbl) lv=1 sz=46
```

목표:

- 두 번째 표 control header의 instance/field payload 불일치가 파일손상 원인인지 확인한다.

### 03_next_table_child_headers

정답 HWP에서 두 번째 표의 child header만 이식한다.

대상:

```text
[48] TABLE       lv=2 sz=24
[49] LIST_HEADER lv=2 sz=65
[50] PARA_HEADER lv=2 sz=24
```

목표:

- 표 내부 첫 셀의 `TABLE/LIST_HEADER/PARA_HEADER` compact record가 원인인지 확인한다.

### 04_next_table_record_span_47_53

정답 HWP에서 두 번째 표 시작부터 첫 셀 문단 line segment까지의 record span을 이식한다.

대상:

```text
[47] CTRL_HEADER(tbl)
[48] TABLE
[49] LIST_HEADER
[50] PARA_HEADER
[51] PARA_TEXT
[52] PARA_CHAR_SHAPE
[53] PARA_LINE_SEG
```

목표:

- 단일 header가 아니라 두 번째 표 첫 record span 전체 정합성이 필요한지 확인한다.

### 05_post_first_table_paras_plus_next_table_span

정답 HWP에서 첫 표 직후 문단 구간과 두 번째 표 첫 record span을 함께 이식한다.

대상:

```text
[37]..[46]
[47]..[53]
```

목표:

- 한컴 파일손상이 첫 표 종료 후 문단과 두 번째 표의 결합 경계에서 발생하는지 확인한다.

## 4. 판정 기준

한컴 판정은 다음처럼 구분한다.

```text
파일 읽기/저장 오류:
  "파일을 읽거나 저장하는데 오류가 있습니다."

파일손상:
  첫 표 등 일부 내용을 렌더링한 뒤 손상/복구 계열 메시지
```

Stage 13 성공 기준:

1. Stage 11보다 한컴 출력 위치가 뒤로 이동하는 variant 찾기
2. 파일손상 없이 열리는 variant가 있는지 확인
3. rhwp-studio 정상 렌더링 유지

## 5. 내부 검증

probe 생성 후 다음을 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter task903_stage13_generate_after_first_table_boundary_probe_variants -- --nocapture
```

가능하면 전체 adapter 테스트도 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
```

## 6. 작업지시자 판정 항목

판정 파일은 `output/` 아래에 생성한다.

판정 기록 형식:

```text
| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_post_first_table_top_level_para_headers |  |  |  |  |
| 02_next_table_ctrl_header |  |  |  |  |
| 03_next_table_child_headers |  |  |  |  |
| 04_next_table_record_span_47_53 |  |  |  |  |
| 05_post_first_table_paras_plus_next_table_span |  |  |  |  |
```

## 7. 승인 요청

Stage 13은 위 계획에 따라 첫 표 이후 boundary probe 5개를 생성한다.

## 8. 구현 결과

승인 후 `tests/hwpx_to_hwp_adapter.rs`에 Stage 13 probe 생성기를 추가했다.

생성 기준선:

```text
Stage 11 05_picture_full_plus_structural_bundle 재현
  - Stage 8 section core
  - 첫 표 첫 셀 LIST_HEADER 65B
  - 첫 표 그림 0 + 1 full payload
  - 첫 표 CTRL_HEADER/TABLE/LIST_HEADER/PARA_HEADER structural bundle
```

그 위에 첫 표 종료 직후 boundary payload만 additive하게 적용했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/01_post_first_table_top_level_para_headers.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/02_next_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/03_next_table_child_headers.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/04_next_table_record_span_47_53.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/05_post_first_table_paras_plus_next_table_span.hwp
```

적용 payload:

| variant | 추가 payload |
|---|---|
| 01 | 첫 표 직후 최상위 `PARA_HEADER` 3개: 문단 `0:1`, `0:2`, `0:3` |
| 02 | 두 번째 표 `CTRL_HEADER(tbl )` raw payload |
| 03 | 두 번째 표의 `TABLE`, 첫 셀 `LIST_HEADER`, 첫 셀 문단 `PARA_HEADER` payload |
| 04 | 두 번째 표 전체 record span `47..53`에 해당하는 table object |
| 05 | 첫 표 직후 문단 `0:1..0:3`의 record payload + 두 번째 표 record span |

## 9. 내부 검증 결과

Targeted test:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage13_generate_after_first_table_boundary_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed; 0 failed; 42 filtered out
```

전체 adapter test:

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
```

결과:

```text
test result: ok. 43 passed; 0 failed
```

모든 Stage 13 variant는 rhwp 재로드 성공, 페이지 수 9를 유지했다.

## 10. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/01_post_first_table_top_level_para_headers.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/02_next_table_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/03_next_table_child_headers.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/04_next_table_record_span_47_53.hwp
output/poc/hwpx2hwp/task903/stage13_after_first_table_boundary_probe/05_post_first_table_paras_plus_next_table_span.hwp
```

판정 기록:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|
| 01_post_first_table_top_level_para_headers | 파일손상 | 첫 표 출력 후 | 정상 | Stage 11과 동일 |
| 02_next_table_ctrl_header | 파일손상 | 첫 표 출력 후 | 정상 | `CTRL_HEADER` 단독은 효과 없음 |
| 03_next_table_child_headers | 파일손상 | `분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %)` 출력 후 | 정상 | 출력 위치 이동 |
| 04_next_table_record_span_47_53 | 파일손상 | `분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %)` 출력 후 | 정상 | 03과 같은 수준 |
| 05_post_first_table_paras_plus_next_table_span | 파일손상 | `분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %)` 출력 후 | 정상 | 03과 같은 수준 |

## 11. 판정 해석

확인된 점:

- `01_post_first_table_top_level_para_headers`는 한컴 출력 위치를 뒤로 밀지 못했다.
- `02_next_table_ctrl_header`도 한컴 출력 위치를 뒤로 밀지 못했다.
- `03_next_table_child_headers`부터 출력 위치가 `분기별 해외직접투자액 추이...` 문단 이후로 이동했다.
- `04`, `05`는 `03`과 같은 수준이므로 두 번째 표 전체 span 또는 첫 표 직후 문단 전체 이식은 추가 효과가 없었다.

따라서 Stage 13 기준에서 효과가 있었던 최소 payload는 다음이다.

```text
두 번째 표의 TABLE/LIST_HEADER/PARA_HEADER child header 계열
```

`CTRL_HEADER(tbl )` 단독이 아니라 표 내부 child header가 한컴 reader의 다음 경계를 통과시키는 핵심 신호다.

## 12. 다음 실패 경계

`dump` 기준으로 한컴이 이동한 출력 위치는 문단 `0:9`다.

```text
--- 문단 0.9 ---
텍스트: "< 분기별 해외직접투자액 추이(억 달러, 전년동기 대비, %) >"
```

그 다음 문단 `0:10`에는 4행×10열 표가 있다.

```text
--- 문단 0.10 ---
controls=1
[0] 표: 4행×10열, 셀=33
```

record dump 기준으로 이 표는 정답 HWP에서 다음 구간으로 시작한다.

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46
[103] TABLE            lv=2 sz=30
[104] LIST_HEADER      lv=2 sz=47
[105] PARA_HEADER      lv=2 sz=24
...
```

Stage 13 `03_next_table_child_headers`에서는 같은 위치가 다음처럼 남아 있다.

```text
[102] CTRL_HEADER(tbl) lv=1 sz=46
[103] TABLE            lv=2 sz=28
[104] LIST_HEADER      lv=2 sz=34
[105] PARA_HEADER      lv=2 sz=22
...
```

따라서 Stage 14는 문단 `0:10`의 4행×10열 표 boundary를 대상으로 한다.
