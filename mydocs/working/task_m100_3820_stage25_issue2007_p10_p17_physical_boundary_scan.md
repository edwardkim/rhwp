---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 25 — issue2007 p10–p17 물리 경계 재판정

## 목적

Stage 24(`e73caba73`)은 p9→p10의 terminal nested-table tail과 그 뒤의 소비된
empty spacer라는 한 원인만 보정했다. 사용자가 지적한 p10–p16의 차이를 이 보정 하나로
모두 해결됐다고 가정하지 않는다.

다음 범위를 동일 물리 쪽으로 직접 대조한다.

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 대상: PDF p10–p17 ↔ `rhwp export-svg --page 9..16` (0-based)

## 수행 계약

1. 최신 native `release-test` binary의 SVG와 기준 PDF를 192dpi의 같은 크기 PNG로
   raster한다.
2. 각 page pair와 앞뒤 경계를 각각 확인한다. 글꼴 raster 차이와 실제 text owner,
   table frame, continuation top/bottom의 이동을 구분한다.
3. 실제 구조 차이가 남은 쪽만 source→render tree→layout 원인을 기록하고 별도
   보정한다. 페이지 수 또는 자동 pixel score만으로 정상 판정하지 않는다.
4. 이번 scan에서 새 원인이 확인되면 현재 stage의 발견·분석까지만 기록하고, 코드 변경은
   다음 stage로 분리한다.

## 직접 대조 결과

`e73caba73`의 최신 native `release-test/rhwp export-svg --profile print`를 한 번 실행해
17개 SVG를 생성했다. 그 중 0-based SVG p9..p16을 선택해, 기준 PDF p10..p17을
`pdftoppm -r 192`로 raster한 1588×2246px 캔버스와 나란히 비교했다. 각 PNG의 왼쪽은
rhwp, 오른쪽은 Hancom 2020 기준 PDF다.

- p10: `< 조사기능 관련 타기관 입법례 >`와 첫 dotted table frame이 기준 PDF와 같은
  물리 top에 시작한다. Stage 24 이전의 terminal-tail false border 및 제목 하향은 없다.
- p11–p15: 앞쪽 continuation table의 끝과 다음 기관 section의 시작이 같은 물리 페이지에
  있고, source owner가 앞·뒤 페이지로 이동하거나 새 top/bottom frame이 생기지 않았다.
- p16–p17: RowBreak table 종료 뒤의 일반 문단과 terminal page 모두 같은 물리 page에
  남는다. p17의 끝 문단이 p16으로 흡수되거나 빈 tail page가 생기지 않는다.

본문 글꼴의 획, PUA glyph 및 폭은 Hancom PDF와 native SVG raster가 다르므로 pair에서
미세한 글자 모양 차이는 남는다. 그러나 이번 판정 범위의 page owner, continuation
frame, 표의 물리 top/bottom, section 시작은 같은 위치다. 이를 글꼴 차이를 상쇄하기 위한
layout 보정 사유로 삼지 않는다.

## 증적

PNG가 LFS 대상이 아님을 `git check-attr filter`로 먼저 확인했다.

- [p10–p17 contact](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p010_p017_contact.png)
- 페이지별 대조: [p10](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p010_pair.png), [p11](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p011_pair.png), [p12](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p012_pair.png), [p13](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p013_pair.png), [p14](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p014_pair.png), [p15](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p015_pair.png), [p16](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p016_pair.png), [p17](../pr/assets/task_m100_3820_stage25_issue2007_p10_p17_physical_boundary_scan/review_p017_pair.png)

## 범위 한계

이 단계는 최신 **native** renderer의 PDF 물리 페이지 직접 판정이다. 사용자가 수행하는
WASM build/브라우저 최종 확인을 대체하지 않으며, 그 환경에서 다시 재현되는 차이는
native 결과를 근거로 정상이라고 단정하지 않고 별도 browser paint 원인으로 조사한다.

이 scan에서는 새 코드 결함을 확인하지 못했으므로 renderer를 추가 변경하지 않았다.
