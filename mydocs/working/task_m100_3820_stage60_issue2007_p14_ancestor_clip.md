---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 60 — issue2007 p14 재귀 셀 clip 소실

## 문제

`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 rhwp 물리 p14 하단에서
금융위원회 항목 ⑦의 마지막 두 줄이 잘린다. 한컴오피스 2020 PDF는
`관계자에게 내보여야`와 `한다.`를 모두 p14에 표시하고 p15는 항목 ⑧로
시작한다.

수정 전 증적:

- [p14 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage60_issue2007_p14_ancestor_clip/review_p014_before.png)
- 원본: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 재현: Stage 59 release-test binary로 `scripts/visual_sweep.py --pages 14-15 --dpi 180`

## 정확한 소실 범위

- p14 outer `PartialTable(pi=7,ci=1)` cursor: start `[282]`, end `[331]`
- p15 cursor: start `[331]`; 항목 ⑧이 소유
- p14 leaf 항목 ⑦ 첫 줄 bbox: `969.7..983.0px`
- p14 leaf `한다.` bbox: `987.0..1000.3px`
- leaf 금융위원회 cell clip bottom: `1006.213px`
- 중간 wrapper cell clip bottom: `979.067px`
- outer wrapper cell clip bottom: `974.947px`
- body clip bottom: `1009.120px`

따라서 page owner나 body 용량은 정상이고, 재귀 wrapper clip만 현재 자식 조각보다
약 31px 짧다. 첫 줄은 약 8px이 잘리고 `한다.`는 전부 잘리며, p15에서
재방출되지 않으므로 실제 콘텐츠 소실이다.

## 원인

Stage 48의 recursive child viewport 보정은 자식 `RowCut`에 속한 마지막 줄을
render tree에 복원했지만, 부모 flow height는 기존 `flow_visible`을 유지했다.
`expand_terminal_cell_clip_to_nested_table_descendants()`는 host가 terminal일 때만 셀 clip을
늘리므로, p14처럼 nonterminal host 안에 명시적 `recursive_cut`으로 소스 범위가
제한된 현재 자식 조각도 조상 clip에서 잘린다.

기존 `contains_painted_text()`는 text bbox와 clip이 조금이라도 교차하면 성공하고,
아예 clip 밖인 `한다.`를 검사하지 않아 이 결함을 놓쳤다. 기존
`LAYOUT_OVERFLOW_CELL`도 page bottom 초과를 보므로 page 안의 ancestor clip 소실은
검출하지 못한다.

## 수정 원칙

1. pagination cursor와 `flow_height`는 바꾸지 않는다.
2. nonterminal cell 전체를 무조건 확장하지 않는다.
3. `recursive_cut`이 source start/end를 명시적으로 제한한 현재 호출이 새로
   붙인 child subtree만 측정해 해당 조상 cell clip에 포섭한다.
4. scalar nonterminal child와 다음 쪽 source tail은 계속 숨긴다.
5. p14 항목 ⑦의 두 줄이 모든 조상 clip 안에 **전체** 포함되는지 회귀로
   고정하고, p15가 여전히 항목 ⑧로 시작함을 함께 검사한다.

## 추가 발견 — p98 제목 뒤 저장 간격 소실

clip 보정 후 사용자 PDF 직접 대조에서 p14의 `6 금융위원회` 제목과 뒤의
`자본시장과 금융투자업에 관한 법률` 표가 서로 붙는 추가 결함을 확인했다.

- p98 제목 LINE_SEG: `vertical_pos=28421`, `line_height=1300`,
  `line_spacing=780` HWPUNIT
- p99 중첩 표 host LINE_SEG: `vertical_pos=30501` HWPUNIT
- 원본 시작점 차이: `2080 = 1300 + 780` HWPUNIT
- 현재 rhwp 제목 top→표 top: `17.3px` = 1300 HWPUNIT만 반영
- 누락: `10.4px` = 780 HWPUNIT의 후행 줄간격

p99는 현재 cut이 선택한 mixed nested fragment이지만 line range가 `(1,1)`인
zero-width control 문단이다. `last_rendered_para_idx`가 `start < end`인 문단만 세어
p98을 셀의 마지막 문단으로 오판하고, 공통 문단 렌더러가 마지막 줄 후행 간격을
제외한 것이 직접 원인이다. 선택되지 않은 `(0,0)` 문단과 TAC 표는 계속 제외하고,
현재 cut이 선택한 `start == end && start > 0` block-table 문단만 마지막 렌더
owner 계산에 포함한다.

## 완료 조건

- 수정 전 회귀는 `한다.` bottom `1000.3 > 974.947` 로 실패
- 수정 후 p14 항목 ⑦ 두 줄과 하단 frame이 모든 ancestor clip 안에 존재
- p98 제목 top→p99 표 top이 저장 차이 2080 HWPUNIT(약 27.73px)와 일치
- p15 항목 ⑧ 소유권과 17쪽 수 유지
- issue2007 focused 회귀와 p14-p15 PDF 재대조 통과
- 수정 커밋 후 새 PR gate stage에서 전체 회귀를 처음부터 순차 실행

## 수정 결과

`recursive_cut`이 현재 쪽 source 범위를 명시한 호출에서, 그 호출이 새로 추가한
직계 table root만 현재 `TableCell` clip에 포섭하도록 보정했다. child subtree를
무조건 재귀 순회하지 않고 일반 nonterminal scalar 경로도 건드리지 않아 다음 쪽
tail을 노출하지 않는다. `flow_height`와 pagination cursor 역시 그대로 유지했다.

회귀는 p14 금융위원회 하위 표의 exact TextLine `한다.`를 특정한 뒤, 페이지부터
해당 줄까지 경로상의 모든 `clip=true TableCell` 교집합이 줄 bbox 전체를
포함하는지 검사한다.

- 수정 전 실패: `line_bottom=1000.333`, `clip_bottom=974.947`
- 수정 후: 유효 clip bottom 약 `1006.2px`, 마지막 줄 bottom `1000.333px`
- 새 helper 단위 회귀: 1건 통과
- `issue_2007_nested_cell_pagination`: 15/15 통과, 17쪽 유지
- 간격 회귀 red: `heading_top=524.693`, `table_top=542.027`,
  `delta=17.333px`
- 수정 후: 제목 top `524.7px`, 표 top `552.4px`, delta `27.7px`
  (`2080 HWPUNIT = line_height 1300 + line_spacing 780`)
- 최종 visual sweep: requested/completed/missing `2/2/0`, SVG/render-tree `17/17`
- p14: 항목 ⑦ 마지막 두 줄과 하단 점선 frame 표시
- p15: 항목 ⑧부터 시작하며 p14 소유 제목을 반복하지 않음

수정 후 증적:

- [p14 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage60_issue2007_p14_ancestor_clip/review_p014_after.png)
- [p15 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage60_issue2007_p14_ancestor_clip/review_p015_after.png)

사용자 PDF 직접 대조에서 별도로 확인된 p12 `중앙선거관리위원회`와 p15 `조달청`
블록 간격 차이는 Stage 61로 이월한다. p14의 clip·후행 간격 보정과 p15 source
소유권 회귀는 이 Stage에서 닫고, 새 결함은 분석 문서를 먼저 커밋한 뒤 진행한다.
