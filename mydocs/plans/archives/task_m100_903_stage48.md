# Task m100 #903 Stage 48 계획

## 1. 목적

Stage47 판정으로 표 배치 붕괴를 정상권으로 끌어올리는 직접 전제는 `CTRL_HEADER` payload로 좁혀졌다.

```text
CTRL_HEADER 포함 후보:
  한컴 성공
  이미지 성공
  rhwp-studio 성공
  큰 표 배치 회복

CTRL_HEADER 미포함 후보:
  표 배치 엉망
```

하지만 `CTRL_HEADER`를 포함한 모든 후보에서 일부 셀 텍스트 배치가 아직 틀렸다.

Stage48의 목적은 Stage47 `06`을 baseline으로 삼아,
남은 셀 텍스트 배치 오류의 직접 원인을 residual tag별로 분리하는 것이다.

## 2. 현재 확정 사항

```text
이미지 출력:
  DocInfo BIN_DATA metadata

큰 표/개체 배치:
  CTRL_HEADER payload

남은 결함:
  일부 셀 텍스트 배치 틀림
```

Stage47 diff에서 성공 파일과 실패 파일 사이에 남은 tag 차이는 다음이다.

```text
PARA_TEXT       55 records
PARA_CHAR_SHAPE 77 records
PARA_LINE_SEG   42 records
TABLE           12 records
```

## 3. 기준 파일

baseline:

```text
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/06_stage46_06_plus_ctrl_list_para_headers.hwp
```

이 파일은 다음 상태다.

```text
한컴 성공
이미지 성공
큰 표 배치 회복
일부 셀 텍스트 배치 틀림
```

success source:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

이 파일은 Stage40 판정에서 정상 기준으로 사용했다.

## 4. 작업 범위

### 4.1 먼저 할 것

Stage47 `06`과 Stage40 success의 `BodyText/Section0` residual diff를 다시 생성한다.

출력:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/stage48_residual_diff.md
```

확인 항목:

```text
1. CTRL/LIST/PARA header 차이가 제거되었는지
2. 남은 tag가 PARA_TEXT/PARA_CHAR_SHAPE/PARA_LINE_SEG/TABLE로 좁혀졌는지
3. TABLE tag 차이가 Stage46에서 이미 graft한 required table 외의 residual인지
```

### 4.2 후보 HWP 생성

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/
```

후보:

```text
01_plus_para_text.hwp
02_plus_para_char_shape.hwp
03_plus_para_line_seg.hwp
04_plus_table_all.hwp
05_plus_text_char_shape.hwp
06_plus_text_char_line_seg.hwp
07_plus_line_seg_table.hwp
08_plus_text_char_line_seg_table.hwp
```

각 후보는 Stage47 `06` baseline에 Stage40 success의 해당 tag payload를 graft한다.

tag:

```text
PARA_TEXT:       67
PARA_CHAR_SHAPE: 68
PARA_LINE_SEG:   69
TABLE:           77
```

### 4.3 하지 않을 것

```text
- Stage47 header 후보 반복
- DocInfo BIN_DATA 재검증
- SHAPE payload 추가 세분화
- production 코드 구현
```

Stage48은 남은 셀 텍스트 배치 오류 원인 분리 stage다.

## 5. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage48_generate_residual_text_layout_probe -- --nocapture
```

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_plus_para_text |  |  |  |  |  |
| 02_plus_para_char_shape |  |  |  |  |  |
| 03_plus_para_line_seg |  |  |  |  |  |
| 04_plus_table_all |  |  |  |  |  |
| 05_plus_text_char_shape |  |  |  |  |  |
| 06_plus_text_char_line_seg |  |  |  |  |  |
| 07_plus_line_seg_table |  |  |  |  |  |
| 08_plus_text_char_line_seg_table |  |  |  |  |  |

## 7. 판정 해석

```text
03에서 셀 텍스트 배치가 회복:
  line_seg 계열이 직접 원인이다.

02 또는 05에서 회복:
  char shape run payload가 텍스트 배치/폭 계산에 영향을 준다.

04 또는 07에서 회복:
  TABLE residual payload가 셀 내부 텍스트 배치에도 필요하다.

08만 정상:
  텍스트 run, char shape, line seg, table residual이 조합으로 필요하다.

모두 실패:
  Stage40 success와 Stage48 08의 diff를 다시 계산해 남은 record를 확인한다.
```

## 8. 성공 기준

```text
1. 일부 셀 텍스트 배치 오류를 residual tag 단위로 좁힌다.
2. 이미지/큰 표 배치 문제와 남은 셀 텍스트 문제를 분리된 구현 축으로 유지한다.
3. 다음 production 구현 후보를 DocInfo BIN_DATA, CTRL_HEADER, residual text/table payload로 정리한다.
```
