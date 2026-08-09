---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 26 — issue2007 p8→p9 중첩 표 종료 경계

## 범위

Stage 24는 p9→p10 terminal 8×4 table tail, Stage 25는 p10–p17의 physical page
owner를 확인했다. p8→p9는 다른 source table 종료와 다음 source block 시작의 경계이므로
앞선 결과로 정상이라고 추정하지 않는다.

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 비교: PDF p8→p9 ↔ `rhwp export-svg --page 7→8`

## 판정 계약

같은 192dpi physical raster에서 p8 하단과 p9 상단을 확대 대조한다. false border,
blank spacer 중복, source owner 이동, table frame 끊김이 실제로 보일 때만 renderer
원인을 분석한다. 글꼴 raster 차이는 layout 결함으로 취급하지 않는다.

## 직접 대조 결과

Stage 25에서 생성한 최신 native SVG p8·p9와 Hancom 2020 기준 PDF p8·p9를 각각
192dpi, 1588×2246px로 raster했다. 경계 확대와 페이지별 pair에서 다음을 확인했다.

- p8의 표 마지막 row·외곽 frame은 PDF와 같은 위치에서 끝난다.
- p9는 `<국내 유사입법례 분석>`으로 시작하며, 제목·첫 문단·`국내기관 조사기능 현황` 표의
  physical top이 기준 PDF와 같다.
- p8 bottom의 표 border가 p9 top에 false frame으로 남지 않고, p9 source block도 p8로
  앞당겨지거나 p10으로 밀리지 않는다.

글꼴 glyph 및 표 안의 문자 폭에는 native SVG/PDF raster 차이가 있지만, 이 경계의
pagination·table frame·source owner 차이는 아니다. 따라서 renderer 코드는 변경하지 않았다.

## 증적

PNG는 `git check-attr filter`에서 LFS 대상이 아님을 먼저 확인했다. 왼쪽은 rhwp,
오른쪽은 Hancom 2020 기준 PDF다.

- [p8→p9 경계 확대](../pr/assets/task_m100_3820_stage26_issue2007_p8_p9_physical_seam/review_p008_p009_boundary.png)
- [p8 전체 대조](../pr/assets/task_m100_3820_stage26_issue2007_p8_p9_physical_seam/review_p008_pair.png)
- [p9 전체 대조](../pr/assets/task_m100_3820_stage26_issue2007_p8_p9_physical_seam/review_p009_pair.png)
