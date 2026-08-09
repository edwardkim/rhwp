---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 62 — issue2007 p14 중간 block 경계 간격

## 문제

Stage 60·61의 clip 및 제목 뒤 저장 간격 보정 뒤에도
`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 물리 p14에서
`제51조(벌칙)` 표 조각과 `6 금융위원회` block 사이의 세로 배치가 한컴오피스
2020 기준 PDF와 다르다. 전체 PR 게이트는 이 차이를 닫을 때까지 보류한다.

기준 PDF는 `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`이며,
Stage 61의 144dpi render tree와 PDF text 좌표를 96dpi CSS 좌표로 정규화했다.

| 기준점 | rhwp y | PDF y | rhwp - PDF |
|---|---:|---:|---:|
| `제51조(벌칙)` text top | `343.6px` | `351.1px` | `-7.5px` |
| 마지막 `제27조제4항을 위반한 자는` text top | `447.6px` | `454.7px` | `-7.1px` |
| `6 금융위원회` text top | `524.7px` | `497.6px` | `+27.1px` |
| `자본시장과 금융투자업에 관한 법률` text top | `554.3px` | `535.9px` | `+18.4px` |

마지막 벌칙 줄 bottom과 다음 제목 top의 공백은 PDF 약 `29.1px`인 반면 rhwp는
약 `63.8px`다. 단순히 제목 뒤 `780HU`를 다시 조정할 문제가 아니라, 앞 재귀 표
조각의 끝과 다음 separator/title owner 사이에서 약 `34.7px`가 추가된 상태다.

## 기존 수정과의 경계

- Stage 60의 p14 항목 ⑦ ancestor clip 확장과 `한다.` 표시를 유지한다.
- Stage 61의 p12·p15 제목 뒤 `780HU` 저장 간격을 유지한다.
- pagination cursor나 source page owner를 임의로 이동하지 않는다.
- p14의 앞 조각, 빈 separator, 제목, 뒤 1×1 표 각각의 source `LINE_SEG`
  좌표와 현재 `CellUnit` 범위를 확인한 뒤 가장 좁은 계약만 수정한다.

## 원인

p14의 앞 조각은 p96의 빈 host 문단 안에 있는 1×1 중첩 표가 끝나는
`scalar terminal fragment`다. 이 mixed stream 자체는 끝났지만 같은 host cell에는
p97·p98의 separator와 `6 금융위원회` 제목이 후속 source owner로 남는다.

- terminal fragment의 `flow_visible`: `346.427px`
- 첫 가시 line box: `17.333px`, 실제 paint: `13.333px`, leading: `4.000px`
- 저장된 중첩 셀 top padding: `141HU = 1.880px`
- host 문단 line spacing: `840HU = 11.200px`

기존 `terminal_single_cell_tail`은 후속 owner 유무를 구분하지 않고
`flow + 2 × first visible unit + 4px`를 frame과 flow에 모두 넣었다. 이 때문에
실제 마지막 줄 아래에 약 `38.7px`의 빈 표 영역이 생겨 다음 제목을 밀었다.
p13은 신규 표, p15는 `recursive_cut=Some`인 일반 continuation이므로 같은 보정을
전역 적용하면 두 페이지가 다시 틀어진다.

## 수정

HWP5 원본에서 다음 조건을 모두 만족하는 경우만
`terminal_table_before_host_successor`로 분리했다.

- `recursive_cut`이 없고 offset continuation의 terminal fragment
- 단일 셀 중첩 continuation이며 현재 host 문단은 빈 문단
- 같은 host cell에 뒤 source owner가 존재

이 경우 frame은 실제 가시 unit과 첫 line box leading·저장 top padding까지만
소유하고, host line spacing은 다음 source owner로 넘어가는 flow에만 더한다.
paint offset에서는 leading·padding을 되감아 p14 첫 줄의 top inset도 복원한다.
후속 owner가 없는 기존 terminal fragment는 종전 tail 계약을 유지한다.

회귀는 p14 절대 위치만 고정하지 않고 다음 세 경계를 함께 고정했다.

- p13 신규 표 첫 줄 inset: `7.0..=8.1px`
- p14 scalar terminal continuation 첫 줄 inset: `3.8..=6.2px`
- p15 recursive continuation 첫 줄 inset: `1.5..=2.3px`

## 완료 조건

- p14 마지막 벌칙 줄→`6 금융위원회` 제목 간격이 PDF 좌표와 허용 오차 안에서 일치
- p14 제목→금융위원회 표 저장 간격과 항목 ⑦ 전체 clip 유지
- p15가 항목 ⑧부터 시작하고 전체 17쪽 유지
- p12·p14·p15 focused 회귀와 p14 PDF 직접 대조 통과
- 수정 결과와 before/after 증적을 이 문서에 기록하고 커밋한 뒤 다음 Stage에서
  전체 PR 게이트를 처음부터 실행

## 결과

96dpi CSS 좌표로 다시 측정한 결과 p14의 중간 block 경계는 다음과 같다.

| 기준점 | 수정 전 rhwp | 수정 후 rhwp | PDF | 수정 후 오차 |
|---|---:|---:|---:|---:|
| 감사원 표 마지막 줄 bottom | `460.9px` | `466.8px` | `468.5px` | `-1.7px` |
| 감사원 표 bottom | `505.2px` | `472.5px` | `473.6px` | `-1.1px` |
| `6 금융위원회` text top | `524.7px` | `497.2px` | `497.6px` | `-0.4px` |
| 금융위원회 표 top | `552.4px` | `525.0px` | `526.3px` | `-1.3px` |

마지막 줄과 감사원 표 하단의 빈 영역은 `44.3px`에서 `5.6px`로 줄었고 PDF의
`5.0px`와 일치한다. 표 하단→`6 금융위원회` 제목은 rhwp `24.8px`/PDF
`24.1px`, 제목 top→다음 표는 rhwp `27.8px`/PDF `28.7px`다. CSS 좌표 잔차는
각각 `+0.7px`, `-0.9px`이며 180dpi 래스터에서도 최대 `3px`다. p14 항목 ⑦
ancestor clip도 유지했다.

- issue2007 focused integration: `15/15` 통과
- #3637 관련 focused integration: `3/3` 통과
- visual sweep: requested/completed/missing `4/4/0`, SVG/render-tree `17/17`
- 자동 구조 후보: `0`쪽; 자동 무플래그만으로 판정하지 않고 p13·p14·p15를 PDF와 직접 대조

증적:

- [p13 신규 표 비회귀](../pr/assets/task_m100_3820_stage62_issue2007_p14_middle_spacing/review_p013_after.png)
- [p14 중간 block 수정 결과](../pr/assets/task_m100_3820_stage62_issue2007_p14_middle_spacing/review_p014_after.png)
- [p15 recursive continuation 비회귀](../pr/assets/task_m100_3820_stage62_issue2007_p14_middle_spacing/review_p015_after.png)
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`

review PNG SHA-256은 p13
`dfffc03134236a80f6a9eb9ef3e96bd292582e2a9d42cf16af10b11c29055ac3`, p14
`a231fe1bb91ec87b2aa08acab2441f609dab895989b6506eea31213230f4b127`, p15
`b2b248be02bbf7b2c3d0217ba8ccf48aad1e15c938cba6ea20c8172d0167480c`다.

Stage 62의 focused 수정과 증적을 커밋한 뒤, 새 Stage에서 p7–p17 전체 직접 대조와
전체 PR 게이트를 새 전용 target으로 처음부터 순차 실행한다.
