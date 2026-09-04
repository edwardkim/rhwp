---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 42 — overflow-cell gate 원인 정정

## 고정된 실패 입력

전체 integration gate `overflow_cell_lines_do_not_grow`가 현재값 4,267줄에서
다음 다섯 fixture의 증가를 보고했다. baseline 완화는 금지하며, 모두 실제 cell clip 밖
소실 줄을 줄이는 코드 수정으로 처리한다.

| fixture | baseline | 현재 | 증가 |
| --- | ---: | ---: | ---: |
| `76076_regulatory_analysis.hwp` | 0 | 10 | +10 |
| `86712_regulatory_analysis.hwp` | 66 | 91 | +25 |
| `issue1891/76076_regulatory_analysis.hwpx` | 0 | 10 | +10 |
| `issue1891/86712_regulatory_analysis.hwpx` | 66 | 91 | +25 |
| `issue3637/regulatory_impact_nested_table_escape.hwpx` | 19 | 23 | +4 |

## 조사 순서

1. `export-svg --json`으로 문서별 page-level overflow 분포를 고정한다.
2. 동일 HWP/HWPX 계열의 공통 page/fragment를 먼저 분리한다.
3. #3637 p26→p27 owner 보정이 만든 4줄은 HWP 2020 source-owner 증적을 유지하면서
   physical tail을 cell clip 안으로 돌리는 최소 변경으로 고친다.
4. focused overflow gate가 baseline 이하가 된 뒤에만 전체 integration을 한 번 재실행한다.

## 페이지별 분해

`export-svg --json`으로 5개 입력을 전수 렌더한 결과다. HWP와 `issue1891`의 확장자
`.hwpx` 입력은 같은 HWP5 계보로 같은 위치·개수를 보고하므로 하나의 renderer 경로로
처리해야 한다.

| fixture 계열 | overflow page (1-based) | 줄 수 | 첫 진단 source |
| --- | --- | ---: | --- |
| `76076` HWP/HWPX | p33 | 10 | `pi=10` line 4–10, 뒤이어 `pi=11..12` |
| `86712` HWP/HWPX | p6, p27 | 1, 90 | p6 `pi=102`; p27은 1×1/중첩 RowBreak tail |
| `issue3637` HWPX | p14, p28 | 8, 15 | mixed nested RowBreak tail |

따라서 source format만으로 갈라지는 다섯 독립 결함이 아니다. 현재 HWP5 wrapper
row-geometry 선택과 HWPX mixed-tail retry가 모두 physical cell bottom을 넘는 source line을
허용하는지 검증한다. p28의 #3637 4줄 증가를 줄이기 위해 p26→p27 PDF source-owner
계약을 되돌리지는 않는다.

## #3637 p28 근인과 정정

`#3637`의 증가분은 p26→p27 source-owner 보정 자체가 아니라, 그 뒤 p28에 놓이는
**12×3 손자 표**의 렌더 경로에 있었다.

| 항목 | 관측값 |
| --- | --- |
| 손자 표 호출 | `depth=2`, `y=1005.7px`, 전체 높이 `303.3px` |
| 부모가 남긴 실제 viewport | `29.6px` |
| 저장 source split | 없음 |
| 수정 전 | 12행 전체를 RenderTree에 생성하고 조상 `TableCell.clip`으로만 숨김 → p28 cell overflow 15줄 |
| HWP 2020 PDF | p28 하단에는 첫 조각만 있고 이후 행은 p29 continuation이 소유 |

이 표에는 `treat_as_char` 저장 비트가 있지만, 실제로는 부모 RowBreak 조각 안에서
물리적으로 잘리는 표다. 따라서 `layout_table`은 명시 source split이 없고 부모 viewport보다
큰 depth>0 표에 한해, 계산된 row-height로 현재 viewport에 들어오는 행 범위를 임시
`NestedTableSplit`으로 만들도록 보정했다. 명시 split은 항상 우선하며, 다음 쪽은 부모
fragment가 다시 호출해 소유한다.

이 방식은 콘텐츠를 drop하지 않는다. 기존에는 다음 쪽 소유 TextLine도 p28 RenderTree에
만든 뒤 SVG clip으로 감췄고, 수정 뒤에는 그 노드를 p28에 만들지 않아 p29이 정상 owner로
렌더한다.

## 검증

- `export-svg --page 27` (1-based p28): `overflowCellLines 15 → 0`.
- `export-svg --page 28` (1-based p29): `overflowCellLines 0` 유지.
- HWP 2020 PDF p28 raster 대조: 하단의 손자 표 첫 조각만 남고 다음 내용은 p29로 이어지는
  경계를 확인했다.
- `issue_3637_nested_table_starts_inside_parent_cell`: p26/p27 source owner 계약 통과.
- `overflow_cell_lines_do_not_grow`: 674 fixtures, skip 3, nonzero 21, total 723으로 통과.
  baseline은 변경하지 않았다.

회귀는 #3637 focused test에 p28의 `TableCell` 내부 TextLine이 page bottom 아래에서 시작하지
않는다는 직접 계약으로 추가했다. 따라서 향후 baseline의 여유(기존 19줄) 안에서 같은
숨은 p28 node가 다시 늘어나는 것도 통과할 수 없다.
