# Task #362 Stage 1 — 결함 origin 정량 분석

## 진단 방법

`src/renderer/height_measurer.rs` 의 `measure_table_impl` 에 임시 디버그 출력 추가:
- 표 측정 결과 (raw_table_height, common_h, scale 발동, row_heights)
- 셀 측정 결과 (text_h, nested 여부, content_h, pad_top, pad_bot, req_h, prev_row_h)

main 과 v0.7.3 worktree 양쪽에 동일 디버그 추가 후 `samples/kps-ai.hwp` p56 (pi=535) 측정값 비교.

## Task 1.1 — 외부 셀 (pi=535, r=0, c=0) 측정 비교

| 항목 | v0.7.3 | main |
|---|---|---|
| text_h (cell paragraphs) | 566.89 | 566.89 |
| nested | true | true |
| content_h (last_seg_end max text_h) | 861.29 | 861.29 |
| pad_top, pad_bot | 1.88, 1.88 | 1.88, 1.88 |
| req_h | 865.05 | 865.05 |
| prev_row_h | 834.81 | 834.81 |

→ **height_measurer 단계의 측정값은 v0.7.3 와 main 이 완전히 일치**.

## Task 1.2 — 외부 표 측정

| 항목 | 값 (v0.7.3 == main) |
|---|---|
| raw_table_height | 865.05 |
| common_h | 865.05 |
| tac | true |
| scale 발동 | NO_SCALE (raw == common) |
| 결과 table_height | 865.05 |

→ TAC 클램프 발동 안 함 (raw == common).

## Task 1.3 — 내부 7x6 표 측정

| 표 | row_heights |
|---|---|
| depth=1, rows=7 (첫번째) | [51.20, 35.47×6] = 264.00 |
| depth=1, rows=7 (두번째) | [28.64, 38.63×4, 7.76, 38.63] = 229.53 |
| depth=1, rows=7 (세번째) | [32.19, 38.20, 59.99, 49.52, 54.00, 17.75, 45.97] = 297.61 |

(외부 셀 안에 7x6 표가 3개 — 표 헤더, 데이터, 평가 결과 3개 구조)

→ 측정값 동일 (v0.7.3 와 main).

## Task 1.4 — 19.5 px 차이의 진짜 origin

### SVG 비교 결과 (재확인)

kps-ai p56 의 마지막 텍스트 "(평가점수)" y 좌표:
| 항목 | v0.7.3 | main |
|---|---|---|
| 마지막 텍스트 y | 1001.18 | 1020.65 |
| 외부 셀 clipPath y_end | 1020.53 | 1020.53 |

→ main 의 텍스트 y 가 0.12 px 초과로 클립 발생, 차이 19.5 px.

### 결론 — 결함 origin 은 layout 단계

- height_measurer 의 측정값 (text_h, content_h, req_h) 은 v0.7.3 와 main 이 동일
- 그러나 실제 렌더 시 셀 안 paragraph y 좌표가 main 에서 19.5 px 더 아래
- → **셀 안의 paragraph 배치 (layout 단계) 에서 누적 y 가 다르게 계산됨**

`git diff v0.7.3..local/devel --stat -- src/renderer/layout/`:
```
paragraph_layout.rs   | 447 +++--
table_cell_content.rs |  19 +-
table_layout.rs       | 363 +++--
text_measurement.rs   | 344 +++-
... (총 1455 라인 변경)
```

→ layout 단계에 광범위한 변경이 있음. 본 task 의 origin 은 **height_measurer (측정) 가 아니라 layout (렌더 좌표 산출)** 입니다.

## Stage 2 의 진단 항목 (재정의)

본 결함의 origin 은 **height_measurer 에서 layout 으로 이동**:

1. **셀 안 paragraph 의 누적 y 계산 비교** (v0.7.3 vs main)
   - `table_layout.rs::calc_cell_paragraphs_content_height` 또는 paragraph 배치 함수
   - 셀 안 11 paragraphs 각각의 y 좌표 trace
2. **measure_cell 의 text_h (566.89)** 와 layout 단계의 누적 y 의 차이
3. **content_height (861.29) 가 어떻게 셀 안 paragraph 좌표로 변환되는지**

## 작업지시자 추가 단서

작업지시자 시각 확인: 표 안에 표 + 문단 + 이미지가 포함된 케이스 전반에서 조판 틀어짐.
→ 본 결함이 단일 origin 이고 영향이 광범위. Stage 2 에서 광범위한 영향을 일으킬 수 있는 layout 변경 (Task #279 cell padding, paragraph_layout 변경) 식별 우선.

## 다음 단계 (Stage 2)

1. 셀 안 paragraph 의 layout 단계 좌표 산출 함수 식별
2. 19.5 px 차이의 origin 을 layout 코드에서 확정
3. 수정 방안 (origin 정정 or 시멘틱 환원)
4. Stage 2 보고서 작성 → 승인 요청
