# Task m100 #903 Stage 47 작업 기록

## 1. 목적

Stage46 판정에서 이미지 출력 원인은 `DocInfo BIN_DATA` metadata로 확정되었다.

반면 표/셀 배치는 Stage46의 모든 후보에서 실패했다.
특히 다음 후보도 실패했다.

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp
```

Stage47은 Stage40 성공 파일과 Stage46 실패 파일을 직접 비교하고,
Stage46 `06` 실패 baseline에 `CTRL_HEADER/LIST_HEADER/PARA_HEADER` payload를 다시 조합한다.

## 2. 기준 파일

성공 기준:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

실패 기준:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp
```

## 3. Section0 diff 리포트

출력:

```text
output/poc/hwpx2hwp/task903/stage47_success_vs_stage46_diff/stage47_section0_diff.md
```

요약:

| item | success | failing |
|---|---:|---:|
| section0 bytes | 225288 | 217165 |
| record count | 7879 | 7879 |
| differing comparable records | 1307 | 1307 |

tag별 차이:

| tag | name | count |
|---:|---|---:|
| 66 | PARA_HEADER | 568 |
| 67 | PARA_TEXT | 55 |
| 68 | PARA_CHAR_SHAPE | 77 |
| 69 | PARA_LINE_SEG | 42 |
| 71 | CTRL_HEADER | 29 |
| 72 | LIST_HEADER | 524 |
| 77 | TABLE | 12 |

관찰:

```text
Stage40 성공 파일과 Stage46 실패 파일의 차이는 header-class만이 아니다.
PARA_TEXT, PARA_CHAR_SHAPE, PARA_LINE_SEG, TABLE payload 차이도 남아 있다.
```

따라서 Stage47 후보가 실패하면 다음 단계에서는 header-class 외 잔여 tag를 다시 분리해야 한다.

## 4. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/
```

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/01_stage46_06_plus_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/02_stage46_06_plus_list_header.hwp
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/03_stage46_06_plus_para_header.hwp
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/04_stage46_06_plus_ctrl_list_headers.hwp
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/05_stage46_06_plus_ctrl_para_headers.hwp
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/06_stage46_06_plus_ctrl_list_para_headers.hwp
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/header_prereq_detail.md
```

## 5. 실행한 검증

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage47_generate_header_prereq_probe -- --nocapture
```

결과:

```text
test task903_stage47_generate_header_prereq_probe ... ok
```

내부 재로드:

| variant | bytes | rhwp reload |
|---|---:|---|
| `01_stage46_06_plus_ctrl_header.hwp` | 374784 | ok, pages=9 |
| `02_stage46_06_plus_list_header.hwp` | 375296 | ok, pages=9 |
| `03_stage46_06_plus_para_header.hwp` | 374784 | ok, pages=9 |
| `04_stage46_06_plus_ctrl_list_headers.hwp` | 375296 | ok, pages=9 |
| `05_stage46_06_plus_ctrl_para_headers.hwp` | 375296 | ok, pages=9 |
| `06_stage46_06_plus_ctrl_list_para_headers.hwp` | 375808 | ok, pages=9 |

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_stage46_06_plus_ctrl_header | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 02_stage46_06_plus_list_header | 성공 | 성공 | 표 배치 엉망 | 표 배치 엉망 |  |
| 03_stage46_06_plus_para_header | 성공 | 성공 | 표 배치 엉망 | 표 배치 엉망 |  |
| 04_stage46_06_plus_ctrl_list_headers | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 05_stage46_06_plus_ctrl_para_headers | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |
| 06_stage46_06_plus_ctrl_list_para_headers | 성공 | 성공 | 일부 셀 텍스트 배치 틀림 | 성공 |  |

## 7. 판정 해석 기준

```text
06이 성공:
  Stage40 성공 조건에는 CTRL/LIST/PARA header payload가 조합 전제 조건으로 필요했다.

일부 후보만 개선:
  개선 후보에 포함된 tag를 중심으로 clean 구현 후보를 세분화한다.

모두 실패:
  Stage47 diff에서 남은 PARA_TEXT/PARA_CHAR_SHAPE/PARA_LINE_SEG/TABLE 차이를 다음 stage에서 분리한다.
```

## 8. 판정 해석

Stage47 판정으로 `CTRL_HEADER`의 역할이 확정되었다.

```text
CTRL_HEADER 포함 후보:
  01, 04, 05, 06
  => 한컴 성공, 이미지 성공, rhwp-studio 성공, 큰 표 배치 회복

CTRL_HEADER 미포함 후보:
  02, 03
  => 한컴 열기는 성공하지만 표 배치 엉망, rhwp-studio도 표 배치 엉망
```

따라서 표 배치 붕괴를 정상권으로 끌어올리는 직접 전제는 `CTRL_HEADER` payload다.

반면 `LIST_HEADER`와 `PARA_HEADER`는 단독으로는 표 배치를 회복하지 못했다.

```text
02 LIST_HEADER only:
  표 배치 엉망

03 PARA_HEADER only:
  표 배치 엉망
```

또한 `CTRL_HEADER`에 `LIST_HEADER` 또는 `PARA_HEADER`를 더해도 남은 결함은 사라지지 않았다.

```text
01 CTRL_HEADER only:
  일부 셀 텍스트 배치 틀림

04/05/06:
  일부 셀 텍스트 배치 틀림
```

따라서 남은 문제는 `LIST_HEADER/PARA_HEADER` 자체가 아니라,
Stage47 diff에 남아 있던 다음 residual tag 중 하나 또는 조합으로 보는 것이 타당하다.

```text
PARA_TEXT
PARA_CHAR_SHAPE
PARA_LINE_SEG
TABLE
```

## 9. 결론

현재까지 구현 후보는 다음처럼 분리된다.

```text
이미지 출력:
  DocInfo BIN_DATA metadata

큰 표/개체 배치:
  CTRL_HEADER payload

일부 셀 텍스트 배치:
  residual PARA_TEXT / PARA_CHAR_SHAPE / PARA_LINE_SEG / TABLE payload
```

Stage48에서는 Stage47 `06`을 baseline으로 삼고,
Stage40 성공 파일에서 residual tag를 조합 graft하여 남은 셀 텍스트 배치 원인을 분리한다.
