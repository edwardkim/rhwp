---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-05
---

# Task #3820 Stage 13 — issue2007 중첩 셀의 native/WASM 전수 정합

## 목적

`samples/basic/issue2007_nested_cell_pagination_42065.hwp`는 이전 보정 뒤 기준 PDF와 같은
17쪽으로 회복됐지만, 최신 WASM 산출물을 Studio에서 열었을 때도 각 페이지의 중첩 표와 본문이
기준 PDF와 같은지 전수로 확정하지 않았다. 이 Stage는 17쪽 전부를 한컴 2020 기준 PDF와 다시
대조하고, 실제 남은 결함만 source → layout → paint 경로로 좁혀 보정한다.

## 기준과 범위

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf` (17쪽, A4)
- 출발 commit: `f6e3dfb1b` (`fix: 다중 줄 부동 표 오프셋 기준을 구분한다`)
- 대상: 저장소의 `issue2007*.hwp` glob에 현재 일치하는 위 단일 fixture.

이전 Stage 11의 `17쪽` page-count 및 raw PUA 0은 필요한 안전 조건일 뿐, 각 페이지의
행 분할·중첩 표 clip·본문 흐름이 기준 PDF와 같다는 증거가 아니다. page count만 맞고 paint가
다른 상태를 정상으로 판정하지 않는다.

## 조사 계약

1. 현재 native release-test binary로 0–16쪽 전수 `fidelity_compare --text-only --export-all-svg
   --layout-ledger`를 실행해 page-count, text owner, glyph, table fragment, overflow 후보를 원장화한다.
2. 같은 binary로 1–17쪽 PDF/SVG raster visual sweep을 실행하고, 모든 review sheet를 확인한다.
3. 특히 p4와 자동 순위 상위·구조 후보 쪽은 source `(pi, ci)`, `CellUnit`, nested split 및
   render tree를 PDF와 대조한다. 자동 지표와 page count만으로 결함을 확정하지 않는다.
4. native 보정 뒤 focused regression과 같은 전수 native 비교를 다시 수행하고,
   `wasm-pack build --target web --out-dir pkg` 후 WASM/Studio 출력도 같은 fixture에서 확인한다.

## 수용 기준

- 기준 PDF와 rhwp의 페이지 수가 모두 17이고, 각 페이지의 중첩 표·본문 owner·caption이
  기준과 모순되지 않는다.
- 발견 결함은 재현 명령, 기준 PDF 대조 이미지, source와 renderer 경로를 기록한 뒤에만 수정한다.
- 수정은 p4를 포함한 실제 발동 source 계약으로 제한하고, `issue_2007_nested_cell_content_paginates`
  및 직접 시각 증적을 함께 갱신한다.
- WASM package는 수정된 Rust source로 재생성하고 Studio에서 같은 fixture가 실제로 읽히는 것을
  확인한다. 단, 사람이 보는 기준 PDF의 최종 시각 판정을 자동 지표로 대체하지 않는다.

## 원인과 보정

전수 초기 sweep에서 p8은 17쪽 page count와 text owner가 맞아도, p7의 마지막 줄
`및 정치부문에 존재하는 것으로 인식되는 부패의 정도를 측정`을 p8에 다시 paint하고 표 전체를
기준 PDF보다 한 줄 아래로 밀었다. render tree에서는 원본 바깥 표(`pi=7`, `ci=1`)의 p8 조각이
cell clip 밖의 후속 흐름까지 포섭해 높이 `6591px`로 확장되어 있었다.

원인은 mixed nested 1×1 RowBreak continuation에서 서로 다른 두 값을 같은 offset으로 다룬 데 있다.

- `offset`은 이미 앞쪽 페이지에서 소비한 **전체 콘텐츠 origin**이다.
- `visible_height`와 `flow_height`는 현 페이지 border/후속 문단에 필요한 **물리 reservation**이며,
  첫 visible unit을 추가로 포함해야 한다.

기존에는 origin에서도 첫 visible unit을 빼서 직전 페이지 마지막 줄을 다시 paint했다. 보정은
origin에는 전체 `offset`을 쓰고, 기존 reservation 계산은 유지한다. 동시에 `clip=true`인
`TableCell`의 **일반 흐름 자손**은 partial-table bbox와 body clip 확장 근거에서 제외했다. 이 자손들은
PageRenderTree에는 남아도 해당 페이지에 paint되지 않는 continuation tail이므로 Canvas/WASM의 현재
페이지 replay 범위를 넓혀서는 안 된다. 단, 직접 배치된 사각형·이미지·텍스트박스 등 도형 subtree는
현재 물리 쪽에서 실제로 paint될 수 있으므로 계속 표 bbox에 포함한다. 이 구분은
`rowbreak-problem-pages` p17의 visible textbox가 표 clip 밖으로 잘리지 않게 하는 기존 회귀로 고정했다.

이후 p4 재감사에서, 이 clip 범위 제한이 다른 축의 결함을 남긴 것을 확인했다. 12×5 nested table의
우측 outer vertical border(`x=729.4`, stroke `1.9px`)는 SVG에 생성됐지만, 상위 1×1 cell/body clip의
우측 끝이 `x=725.7`이어서 paint 전에 전부 제거됐다. 이는 child flow tail을 다시 허용할 사유가 아니다.
`layout_table`은 이제 clip된 cell에서 **직접 중첩 Table의 실제 outer vertical border stroke만** 가로 clip
범위에 포함한다. 그 외 자손, 특히 다음 물리 쪽 continuation tail은 계속 clip 밖에 남는다.

같은 원인을 빠른 전수 후보화에서도 놓치지 않도록 `tools/fidelity_compare/fidelity_compare.py`의
`--layout-ledger`에 `svg-table-border-clip-candidates.tsv`를 추가했다. render tree의 Table direct border와
SVG `<line>`을 연결하고 Body/TableCell ancestor clip 뒤 가시 폭이 20% 이하인 outer vertical border를
candidate로 기록한다. 수정 전 p4에는 3건(큰 12×5 표 우측선 포함), 수정 후에는 0건이다.

회귀는 page count만 보지 않고 p8 `pi=7/ci=1` fragment의 physical bottom이 페이지 안에 남는지와,
TableCell clip을 적용한 가시 텍스트에 p7 마지막 줄이 다시 나타나지 않는지를 고정한다.

## p10–p16 재발견과 추가 보정

사용자 직접 검토로 p10–p16 전체가 비정상임을 확인했다. 종전의 page-count·frame 후보 해석은 이
결함을 놓친 잘못된 판정이므로 취소한다. 일곱 쪽은 모두 원본 `pi=7`, `ci=1`의 같은 1×1 RowBreak
continuation 조각이며, 깊은 손자 1×1 셀의 `LINE_SEG` 중간 줄이 `vertical_pos=0`으로 되돌아간다.
그 reset을 새 셀의 시작으로 오인해 p10–p16 각각에서 뒤 문단이 셀 상단으로 재배치되었고, 수정 전
`table-cell-text-overlap-candidates.tsv`에는 매 쪽 28쌍의 실제 TextLine 겹침이 기록됐다.

보정은 continuation 전용 상태를 새로 전파하는 방식이 아니라, **모든 TableCell**에서 문단 또는 문단
내부 줄의 첫 번째가 아닌 `vpos=0` reset을 찾은 뒤에는 저장 좌표 anchor를 다시 쓰지 않고 누적 flow를
유지하는 방식이다. 따라서 p2의 일반 9×2 표 우측 셀(수정 전 5쌍 겹침)도 p10–p16과 같은 코드 경로로
해소된다. 한편 p8의 콘텐츠 origin은 첫 visible unit을 다시 빼지 않는 기존 `offset` 계약을 유지했다.
둘은 서로 다른 현상이다.

보정 후 native release-test binary로 p10–p16을 다시 추출한 결과, 같은 ledger의 TextLine overlap은
**7쪽 모두 0쌍**이다. p10 및 p16의 PDF 직접 raster 대조와 p10–p16 전체 pair sheet에서 이전의 본문
겹침이 사라진 것을 확인했다. 회귀는 `issue_2007_nested_cell_continuation_does_not_rebase_descendant_vpos`
가 `pi=7/ci=1`의 p10–p16 모든 nested cell을 대상으로 같은 기하 조건을 검사해 고정한다.

## 현재 상태와 이월

- p4 nested table 우측 outer border clip 보정과 p8 continuation duplicate-line 보정은 유지한다.
- p2의 표 셀 내부 문단 겹침과 p10–p16의 대규모 본문 겹침은 위 보정으로 해소했지만, Stage 13은 아직
  **active**다. 나머지 페이지의 PDF 정합은 별도 직접 검토·보정 대상이며, page count 17만으로 합격
  처리하지 않는다.
- 최신 native binary의 17쪽 전체 `--text-only --layout-ledger`도 실행했다. `table-cell-text-overlap`
  원장은 header만 남아 p2·p10–p16 외 다른 셀 내부 본문 겹침도 0건이다. table footer/outside-frame의
  보조 후보는 p5–p17 continuation frame에서 남아 있으므로, 이 값만으로 시각 합격으로 승격하지 않는다.
- 이번 source 변경 뒤 WASM package는 재생성하지 않았다. 이전 WASM build는 사용자가 수동으로 확인한
  산출물이며, native 보정과 동일하다는 최종 근거로 사용하지 않는다.

전수 비교와 원본/정답지/생성 증적은 [Stage 13 visual sweep](task_m100_3820_stage13_visual_sweep.md)에
보관한다. HWP와 한컴 PDF는 이미 저장소의 `samples/basic/`와 `pdf/basic/` canonical 경로에 보관되어
있으므로 중복 복사하지 않았다.
