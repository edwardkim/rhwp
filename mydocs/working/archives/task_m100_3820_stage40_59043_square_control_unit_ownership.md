---
kind: analysis
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 40 — #1921 p98 Square picture의 fragment 소유권

## 목적

Stage 39에서 확정한 p12의 PDF page-owner 이탈을, RowBreak 표 전체의 페이지 수나 baseline을
바꾸지 않고 p98 row 2의 Square picture control과 `CellUnit` cut의 대응으로 설명한다. 이 문서는
분석이 끝나기 전의 구현 변경을 금지하는 작업 기준이다.

## 기준 자료와 직접 대조

- 입력: `samples/issue1921/59043_regulatory_analysis.hwp`
- 정답지: `pdf/issue1921/59043_regulatory_analysis-2022.pdf`
- 정답 PDF p11은 동영상/소개 영역 뒤에 product image와 설명을 같은 row 2의 첫 fragment로 둔다.
- 정답 PDF p12는 `JOUZ` 고지와 두 개의 SNS screenshot으로 row 2를 끝내며, p13은 이미
  `나. 정부개입 필요성`으로 진행한다.
- current p11은 같은 row의 Square control 중 일부를 화면상 빈 영역으로 남기며, current p12는
  product image와 큰 빈 영역을 차지한다. current p13은 PDF p12가 끝낸 SNS image 영역을 다시
  배치한다. 따라서 문제는 단순 글꼴·raster 차이가 아니라 control-to-page owner의 오류다.

## 원본 구조

`rhwp dump --para 98`으로 확인한 `pi=98` outer 표는 6×1 `RowBreak`다. 문제 cell은 row 2,
32 paragraph, `pad_top=9968 HU`이며 control anchor는 다음과 같다.

| cell paragraph | control | wrap | 저장 vpos |
| --- | --- | --- | --- |
| 0 | bin 8, 9 | Square | 0 |
| 6 | bin 7 | Square | 8592 |
| 19 | bin 10 | Square | 27208 |
| 31 | bin 11 | Square | 49277 |

각 control은 독립 paragraph의 Square floating picture다. 그러나 현재 `cell_units_uncached`는
Square/Tight/Through flow 높이를 16px짜리 무명 fragment unit으로 분해한다. unit에는
`para_idx`만 있고 **어느 control의 흐름 높이인지**는 없다.

## 관측한 cut와 renderer 동작

current page fragment는 `p11: [0,29)`, `p12: [29,82)`, `p13: [82,end)`를 사용한다.
`RHWP_DIAG_CELLPARA`에서 p12는 cp6와 cp19의 `nonline=true`를 확인했다. renderer의
`cell_cut_contains_non_inline_control_units`는 "현재 paragraph의 generic non-inline unit이 한 개라도
cut에 있는가"만 묻고, 그 unit이 어떤 picture의 height에서 왔는지는 알지 못한다.

그 결과 p12는 cp6/cp19 content를 셀 fragment의 상단에서 다시 paint하고, p13은 cp31 picture를
독립 페이지로 늦게 paint한다. 이는 각 picture의 저장된 vpos/height 범위와 cut의 교집합으로
owner를 결정해야 하는 HWP5 계약과 다르다.

## 수정 전 검증 조건

1. CellUnit 또는 동등한 cut 메타데이터가 Square control의 개별 owner를 구별할 수 있어야 한다.
2. p11--p13에서 각 control은 정확히 한 page fragment에만 paint되어야 한다. p12가 끝난 뒤 p13은
   `나. 정부개입 필요성`으로 진행해야 한다.
3. #1921 p8 containment regression, `issue_2430_cell_rewrap_threshold`, `issue_2007`, `issue_1939`,
   그리고 overflow baseline debt를 다시 확인한다. 기존 baseline debt를 성공으로 가장하지 않는다.
4. PDF p11--p13 page-level fidelity와 render tree의 image bbox를 모두 남긴다.

## 다음 구현 범위

Square picture flow unit에 control identity/range를 보존하고, partial-table renderer가 현재 cut의
control unit만 paint하도록 최소 변경을 검토한다. generic 16px fragment를 일괄 제거하거나
stored `vpos`를 모든 RowBreak 표에 강제 적용하는 전역 변경은 하지 않는다. 그러한 변경은 #2007,
#2430 및 기존 native HWP5 pagination을 동시에 회귀시킬 범위이기 때문이다.

## 반증된 1차 실험 — 채택하지 않음

"intact vpos 사다리면 Square flow 높이를 전부 생략하고, visible line이 있는 fragment가 picture를
소유한다"는 좁은 실험을 수행했다. p11--p13의 총 page count는 39에서 38로 줄었지만, direct SVG
p11은 정답 PDF의 동영상·product sequence 대신 product image 하나와 빈 cell만 남겼다. render tree도
p11 row 2의 두 기존 image가 여전히 cell top보다 위에 있고, 새로 보인 image 하나만 cell 안에 있었다.

따라서 저장 vpos는 **picture owner를 고르는 데는 필요하지만**, 이 표에서 Square flow 높이를 전부
대체하는 완전한 row-height source는 아니다. 해당 실험 코드는 즉시 되돌렸으며, 다음 단계는
control별 flow unit의 identity와 fragment clip을 보존하는 방식으로 한정한다.
