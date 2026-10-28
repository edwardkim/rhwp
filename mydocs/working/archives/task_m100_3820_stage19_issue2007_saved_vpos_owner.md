---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-05
---

# Task #3820 Stage 19 — issue2007 저장 페이지 경계 보존

## 문제와 기준

`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 한컴 기준은
`pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`이다. Stage 18의 현재
macOS SVG/PDF 직접 대조에서 p8–p17은 모두 불일치로 기록돼 있었으나, 이 단계에서는
제목의 **물리 쪽 소유**부터 PDF 원본과 다시 확인한다.

## 재현·원인

`export-hwpx`로 원본을 임시 해석한 결과, 큰 1×1 nested RowBreak 셀 안에서 제목은
`hp:p id="239"`, `paraPrIDRef="97"`, `LINE_SEG vertpos="0"`이다. 바로 앞 문단의
마지막 `vertpos`는 64960이므로 저장된 셀 내부의 명시적 다음 페이지 시작이다.
`keepWithNext=0`이므로 문단 속성의 keep 규칙으로 추정해 이동할 수 없다.

처음에는 이 reset을 p8 시작으로 해석했다. 그러나 기준 PDF를 실제 물리 7–9쪽으로 다시 raster
대조한 결과는 다음과 같다.

| PDF 물리 쪽 | 기준 내용 | rhwp 현재 내용 |
| --- | --- | --- |
| 7 | `<해외 반부패 전담기구 조사기능 현황>` 제목 | 같은 제목 |
| 8 | 해외 조사기능 표 | 같은 표 |
| 9 | `<국내 유사입법례 분석>` 제목과 국내 표 | 같은 제목과 표 |

기존 p8/p9 판정은 0-based export page와 PDF의 1-based 물리 쪽을 혼동한 오류였다. 이 지점에서
hard break를 추가하면 제목과 표 사이를 인위적으로 갈라 실제 PDF보다 나쁜 출력이 된다.

## p10–p17 Canvas/PDF 직접 재검증과 보정

독립 Studio 인스턴스에서 원본 HWP를 다시 열고 Canvas를 794×1123으로 raster한 뒤, 같은
해상도의 PDF 물리 p10–p17과 짝지어 비교했다. p10–p16의 표 frame·쪽 소유는 현재 tree와
Canvas에서 유지된다. 반면 p17의 terminal 1×1 continuation은 첫 제목 `3) 선호된 대안의
기대효과` baseline이 163.63px로 PDF보다 정확히 한 첫 가시 unit(32px) 아래였다.

원인은 `mixed_nested_split_from_cut`가 첫 visible unit을 이미 fragment의 물리 reservation에
반영했는데, `terminal` 조각만 `offset_within_start`에서 그 unit을 다시 남긴 분기였다. terminal
tail의 clip 높이는 별도 `terminal_single_cell_tail` 경로가 보존하므로, content origin에도
비종료 조각과 같은 보정을 적용해야 한다. 보정 후 p17 baseline은 131.63px로 PDF 시작과
일치한다.

회귀는 p10의 첫 가시 행뿐 아니라 p17 terminal heading도 nested table viewport 상단 14px
이내에 있어야 한다고 고정했다. focused suite는 8/8 통과했다.

## 결론

페이지 소유 자체에 hard break를 추가하지 않는다. PDF 물리 쪽 기준 검증을 유지하면서,
terminal 여부로 달라졌던 1×1 nested continuation의 content-origin 보정을 제거해 p17 마지막
조각의 수직 drift를 해소했다.
