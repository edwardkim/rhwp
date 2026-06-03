# Task m100 #903 Stage 47 계획

## 1. 목적

Stage46 판정으로 이미지 출력 원인은 확정되었다.

```text
DocInfo BIN_DATA 적용 있음:
  이미지 출력 성공

DocInfo BIN_DATA 적용 없음:
  이미지 출력 실패
```

그러나 표/셀 배치는 Stage46의 모든 후보에서 실패했다.

특히 다음 후보도 실패했다.

```text
06_docinfo_bindata_plus_shape_full_payload_plus_table_required
```

이 후보는 Stage37/40에서 성공 조건으로 보였던 `DocInfo BIN_DATA + SHAPE_FULL + TABLE_REQUIRED`를
Stage44 baseline 위에 재현하려는 상한 후보였다.
따라서 Stage47의 목적은 다음 질문에 답하는 것이다.

```text
왜 Stage40의 성공 조합은 성공하고,
Stage46의 동일해 보이는 조합은 표 배치에 실패하는가?
```

## 2. 현재 해석

Stage40 성공 후보는 다음 baseline 위에서 만들어졌다.

```text
output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp
```

이 baseline에는 이미 다음 payload graft가 들어 있다.

```text
CTRL_HEADER
LIST_HEADER
PARA_HEADER
```

Stage37에서는 이 세 payload가 단독으로는 한컴 오류를 해결하지 못했기 때문에
직접 원인에서 제외했다.

하지만 Stage46 결과는 이 payload들이 다음 조합의 전제 조건일 가능성을 다시 연다.

```text
CTRL/LIST/PARA header payload
+ SHAPE_FULL
+ TABLE_REQUIRED
```

즉 Stage47에서는 `단독 원인`이 아니라 `조합 전제`로 다시 검증한다.

## 3. 기준 파일

성공 기준:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

이 파일은 Stage40에서 다음 조건으로 성공했다.

```text
SHAPE_FULL:
  21, 22, 35, 36, 807, 808, 809, 810, 811, 812, 813

TABLE_REQUIRED:
  48, 103, 286, 433, 563, 742, 1619, 2944, 6466
```

실패 기준:

```text
output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp
```

이 파일은 이미지 출력은 성공했지만 표/셀 배치는 실패했다.

## 4. 작업 범위

### 4.1 먼저 할 것

두 파일의 `BodyText/Section0` record를 비교한다.

출력:

```text
output/poc/hwpx2hwp/task903/stage47_success_vs_stage46_diff/stage47_section0_diff.md
```

보고 항목:

```text
1. record count 일치 여부
2. tag/level 일치 여부
3. data size가 다른 record 목록
4. data bytes가 다른 record 목록
5. 특히 CTRL_HEADER/LIST_HEADER/PARA_HEADER 차이 여부
6. SHAPE/TABLE index가 정말 Stage40 성공 파일과 같아졌는지
```

### 4.2 후보 HWP 생성

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/
```

후보:

```text
01_stage46_06_plus_ctrl_header.hwp
02_stage46_06_plus_list_header.hwp
03_stage46_06_plus_para_header.hwp
04_stage46_06_plus_ctrl_list_headers.hwp
05_stage46_06_plus_ctrl_para_headers.hwp
06_stage46_06_plus_ctrl_list_para_headers.hwp
```

각 후보는 Stage46 `06` 실패 파일을 baseline으로 삼고,
positive에서 해당 tag payload만 graft한다.

```text
CTRL_HEADER: tag 71
LIST_HEADER: tag 72
PARA_HEADER: tag 66
```

### 4.3 하지 않을 것

```text
- 새 production 구현
- Stage45 raw_rendering 추가 세분화
- Section0 전체 raw stream graft
- Stage40에서 이미 확인한 TABLE index leave-one-out 반복
```

## 5. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage47_generate_header_prereq_probe -- --nocapture
```

## 6. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_stage46_06_plus_ctrl_header |  |  |  |  |  |
| 02_stage46_06_plus_list_header |  |  |  |  |  |
| 03_stage46_06_plus_para_header |  |  |  |  |  |
| 04_stage46_06_plus_ctrl_list_headers |  |  |  |  |  |
| 05_stage46_06_plus_ctrl_para_headers |  |  |  |  |  |
| 06_stage46_06_plus_ctrl_list_para_headers |  |  |  |  |  |

## 7. 판정 해석

```text
06이 성공:
  Stage40 성공 조건에는 CTRL/LIST/PARA header payload가 전제 조건으로 필요했다.
  이후 clean 구현은 header payload의 누락 필드를 해석해야 한다.

CTRL 또는 PARA 포함 후보만 개선:
  이미지/개체 anchor 계열 payload가 해당 header에 들어 있을 가능성이 높다.

LIST 포함 후보만 개선:
  table/list geometry 또는 cell list attr 계열 payload가 핵심일 가능성이 높다.

모두 실패:
  Stage46 06과 Stage40 성공 파일의 diff report를 기준으로,
  header tag 외의 남은 record 차이를 다음 stage에서 분리한다.
```

## 8. 성공 기준

```text
1. Stage40 성공과 Stage46 실패의 차이를 명시적인 record payload 차이로 설명한다.
2. CTRL/LIST/PARA header payload가 조합 전제인지 확인한다.
3. 다음 clean 구현 범위를 TABLE payload가 아니라 header payload까지 확장해야 하는지 결정한다.
```
