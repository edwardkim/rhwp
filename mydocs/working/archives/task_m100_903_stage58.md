# Task m100 #903 Stage 58: HWPX-H 샘플 table attr 회귀 판정 기록

## 1. 목적

Stage 58은 `hwpx-h-01`, `hwpx-h-02`, `hwpx-h-03` 샘플을 대상으로 table attr 계열 변경이
이미지 출력, 표/셀 배치, 페이지네이션, 한컴 판정에 어떤 영향을 주는지 비교한 시각 판정 기록이다.

이 문서는 `hwpx2hwp-rule.md`의 table attr 관련 관찰을 저장소 안에서 참조 가능하게 남기기 위해
대화 기록에 남아 있던 작업지시자 판정표를 복원한 것이다.

## 2. 판정 표

| variant | sample | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|---|
| 01_current_baseline | hwpx-h-01 | 성공 | 성공 | 실패 | 성공 | 성공 | 이미지 출력 성공, 표 배치 실패 | 현 production |
| 01_current_baseline | hwpx-h-02 | 성공 | 실패 | 실패 | 성공 | 성공 | 이미지 출력 실패, 표 배치 실패 | 현 production |
| 01_current_baseline | hwpx-h-03 | 파일 손상 | 실패 | 실패 | - | 2페이지까지만 | 이미지 출력 실패, 표 배치 실패 | 현 production |
| 02_all_table_attr | hwpx-h-01 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | Stage57 후보 |
| 02_all_table_attr | hwpx-h-02 | 성공 | 실패 | 성공 | 성공 | 성공 | 이미지 출력 실패, 표 배치 성공 | Stage57 후보 |
| 02_all_table_attr | hwpx-h-03 | 파일 손상 | 실패 | 실패 | - | 2페이지까지만 | 이미지 출력 실패, 1페이지 페이지네이션 실패 | Stage57 후보 |
| 03_all_table_margin | hwpx-h-01 | 성공 | 성공 | 실패 | 성공 | 성공 | 이미지 출력 실패, 표 배치 실패 | margin only |
| 03_all_table_margin | hwpx-h-02 | 성공 | 실패 | 실패 | 성공 | 성공 | 이미지 출력 실패, 표 배치 실패 | margin only |
| 03_all_table_margin | hwpx-h-03 | 파일 손상 | 실패 | 실패 | - | 2페이지까지만 | 이미지 출력 실패, 표 배치 실패 | margin only |
| 04_all_table_attr_margin | hwpx-h-01 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | attr + margin |
| 04_all_table_attr_margin | hwpx-h-02 | 성공 | 실패 | 성공 | 성공 | 성공 | 이미지 출력 실패, 표 배치 성공 | attr + margin |
| 04_all_table_attr_margin | hwpx-h-03 | 파일 손상 | 실패 | 2페이지까지만 성공 | 2페이지까지만 성공 | 실패 (2페이지 이미지 객체 묶기 전까지만) | 이미지 출력 실패, 1페이지 페이지네이션 실패 | attr + margin |
| 05_non_tac_attr_margin | hwpx-h-01 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | treat_as_char=false |
| 05_non_tac_attr_margin | hwpx-h-02 | 성공 | 실패 | 성공 | 성공 | 성공 | 이미지 출력 실패, 표 배치 성공 | treat_as_char=false |
| 05_non_tac_attr_margin | hwpx-h-03 | 파일 손상 | 실패 | 2페이지까지만 성공 | 2페이지까지만 성공 | 실패 (2페이지 이미지 객체 묶기 전까지만) | 이미지 출력 실패, 1페이지 페이지네이션 실패 | treat_as_char=false |
| 06_flow_table_attr_margin | hwpx-h-01 | 성공 | 성공 | 실패 | 성공 | 성공 | 이미지 출력 성공, 표 배치 실패 | non-TAC + TopAndBottom |
| 06_flow_table_attr_margin | hwpx-h-02 | 성공 | 실패 | 3페이지에서 페이지네이션 실패 | 성공 | 10페이지로 증가 | 이미지 출력 실패, 페이지네이션 실패 | non-TAC + TopAndBottom |
| 06_flow_table_attr_margin | hwpx-h-03 | 파일 손상 | 실패 | 2페이지까지만 성공 | 2페이지까지만 성공 | 실패 (2페이지 이미지 객체 묶기 전까지만) | 이미지 출력 실패, 1페이지 페이지네이션 실패 | non-TAC + TopAndBottom |
| 07_repeat_header_attr_margin | hwpx-h-01 | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | repeat_header=true |
| 07_repeat_header_attr_margin | hwpx-h-02 | 성공 | 실패 | 성공 | 성공 | 성공 | 이미지 출력 실패, 3페이지 페이지네이션 실패, 10페이지로 증가 | repeat_header=true |
| 07_repeat_header_attr_margin | hwpx-h-03 | 파일 손상 | 실패 | 2페이지까지만 성공 | 2페이지까지만 성공 | 실패 (2페이지 이미지 객체 묶기 전까지만) | 이미지 출력 실패, 1페이지 페이지네이션 실패 | repeat_header=true |
| 08_page_break_attr_margin | hwpx-h-01 | 성공 | 성공 | 실패 | 성공 | 성공 | 이미지 출력 실패, 표 배치 실패 | page_break != None |
| 08_page_break_attr_margin | hwpx-h-02 | 성공 | 성공 | 실패 | 성공 | 성공 | 이미지 출력 실패, 표 배치 실패 | page_break != None |
| 08_page_break_attr_margin | hwpx-h-03 | 파일 손상 | 실패 | 실패 | 성공 | 실패 (2페이지 이미지 객체 묶기 전까지만) | 이미지 출력 실패, 표 배치 실패 | page_break != None |

## 3. 관찰

Stage 58 판정은 다음을 보여준다.

```text
1. table attr 계열은 hwpx-h-01의 표 배치 성공에 영향을 준다.
2. 같은 변경이 hwpx-h-02의 이미지 출력 문제를 해결하지는 못한다.
3. hwpx-h-03은 여러 variant에서 파일 손상과 2페이지 이후 실패를 유지한다.
4. page_break 계열은 hwpx-h-01/02에서 표 배치 실패를 유발하거나 유지한다.
5. repeat_header / all_table_attr 계열은 일부 샘플에서 표 배치 개선과 회귀를 동시에 만든다.
```

## 4. 해석 한계

이 표는 작업지시자 시각 판정 기록이다. 구현 규칙으로 승격하려면 다음이 추가로 필요하다.

```text
한컴 HWP oracle의 TABLE / CTRL_HEADER / LIST_HEADER / PARA_HEADER tuple inventory
rhwp generated HWP의 같은 tuple inventory
누락/추가/값 다름/순서 다름 diff
샘플별 회귀 여부
HWPX construct -> HWP5 table/control tuple lowering contract
```

따라서 Stage 58은 table attr 관련 "관찰 근거"이지만, 그 자체만으로 production lowering rule은 아니다.
