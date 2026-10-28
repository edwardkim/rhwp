---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 25 — p44–45 false-positive 정정과 자동 후보 경계

## 출발점

Stage 23의 p43 보정(`659e1efca6453ce8510f679da1e2b4ace7362f6f`) 뒤, 선택 sweep은
p44에 `column_text_flow_collapse`를 냈다. 이를 표 19·20의 잔여 조판 결함으로 해석해
Stage 23 결과 문서에 이월했지만, 이는 코드 보정을 시작하기 전에 PDF·source·현재 render tree를
다시 대조해야 하는 **후보**일 뿐 결함 확정이 아니었다.

이번 Stage는 다음 두 질문을 분리한다.

1. p44–45의 실제 본문/표 physical owner가 한컴 PDF와 다른가.
2. `fidelity_compare.py`와 visual sweep 중 어느 자동 신호가 이 유형을 구분하며, 표 raster가
   본문 column-flow 신호를 오염시키지 않게 하려면 무엇을 바꿔야 하는가.

HWP가 renderer 입력이고, 한컴 2020 PDF가 physical-layout 정답지다.

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 같은 개인정보 제거 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`

## p44–45 재판정 — renderer 결함 아님

현재 native binary(`target/review-p127-audit/release-test/rhwp`, SHA-256
`402d607d0690f4407bd8feb8e07eedcafd73ef33ee164abc418d7d86198e56ef`)에서 source `pi=516`의
stored `LINE_SEG`는 `vpos=[62711,64711,66711,68711,70711,0]`이다. 마지막 reset tail
(`textStart=296`, `되었으며, <표 20>과 같음.`)은 p45로 가야 한다.

| 비교 대상 | p44 | p45 |
| --- | --- | --- |
| 현재 rhwp render tree | table `pi=515/ci=0,1`, `pi=516`의 reset 전 5줄 | `pi=516` reset tail 뒤 `pi=517` |
| 한컴 PDF text | `…합병증이 인정`으로 끝남 | `되었으며, <표 20>과 같음.`으로 시작 |
| 판정 | **일치** | **일치** |

수정 전 binary(`0b1aad…`)는 p44에 `pi=516` 6줄 전부를 두고 p45를 `pi=517`로 시작했다.
따라서 original owner 결함은 `fidelity_compare` Stage 20 text ledger에서도 p44
`reference_only=8/svg_only=11`, p45 `14/0`으로 포착됐다. 현재 Stage 23 ledger는 p42–45가
모두 `0/0`이다. p43의 exact footnote-reset 보정이 p44–45의 same source reset tail도 올바르게
분할한 연쇄 효과다.

현재 p44의 table bbox는 `pi=515/ci=0: (170.4,324.4,433.3×208.8)`,
`pi=515/ci=1: (179.5,557.9,418.2×292.3)`이다. table 아래 `pi=516`의 다섯 본문 줄은
rhwp raster y=`1379–1558px`, PDF y=`1380–1559px`로 1px 이내에서 맞는다.

## 자동 도구의 역할과 수정 계약

`fidelity_compare.py`는 다음을 구분한다.

- **페이지 간 텍스트 owner 이동**: PDF text ↔ SVG text multiset으로 p44 원래 결함처럼 한 쪽의
  과잉/다음 쪽의 결손을 빠르게 후보화한다.
- **Square/Tight/Through 그림과 본문 physical 교차**: p127의 `square_wrap_text_overlap`으로,
  수정 전 1 → 수정 후 0을 재현했다.

그러나 같은 문자·같은 page owner를 유지한 표 row geometry나 raster stroke 차이는 이 도구만으로
확정할 수 없다. 그 경우 PDF/SVG visual sweep과 사람/PDF 대조가 필요하다.

p44의 false positive는 `visual_sweep.py`가 실제 1단 문서를 종이 좌·우 half로 자른 뒤,
중앙 table 19·20의 선과 셀 text를 본문 line band로 센 데서 생겼다. rhwp right-half 53 bands와
PDF 40 bands의 index가 표 내부에서부터 밀려, 이후 정상 본문까지 `column_text_flow_collapse`로
오인됐다.

따라서 다음의 좁은 도구 보정을 적용한다.

1. render tree의 Body `Table` bbox를 각각 rhwp/PDF raster 좌표로 투영한다.
2. **본문 column-flow 후보 계산에 한해** 이 table 영역을 mask한다. raw column band diagnostics와
   표 자체의 visual comparison은 보존한다.
3. mask 뒤 text band가 여전히 count/y-flow 기준을 넘을 때만 `column_text_flow_collapse`를 낸다.
   따라서 p127 같은 그림 wrap 붕괴는 계속 검출하고, p44 같은 table-stroke false positive만 제거한다.
4. synthetic positive/negative regression과 현재 p44–45 targeted sweep으로 전·후 결과를 고정한다.

이 변경은 표 fragment owner 자체를 합격 처리하지 않는다. 표 owner 결함은 `fidelity_compare` text
ledger, render tree table/footer/frame 후보 및 PDF 3-way review에서 별도 조사한다.

## 이월 원장

아래 상태는 앞 Stage의 `active` front matter가 아니라 가장 최근 source/PDF evidence를 우선해
정리했다. 해결된 항목을 다시 결함으로 되살리지 않으며, 해결 근거가 없는 과거 사용자 보고도 누락하지 않는다.

| 범위 | 계약 | 상태 |
| --- | --- | --- |
| p23–24, p25, p30–32, p37 | 그림/각주 reset/중복 paint의 이전 계약 | 해결 — Stage 4/19, Stage 13/16, Stage 18 evidence |
| p42 | `question_marker_flow_drift` | 결함 확정 전 review candidate |
| p43 | 본문 tail ↔ 각주 39–44 separator | 해결 — `659e1efca` + `3f77b6cdd` |
| p44–45 | 표 19·20 뒤 `pi=516` reset tail owner | **해결** — 이 Stage의 재판정으로 Stage 23 false-positive 이월을 정정 |
| p26–27 | 각주 26 physical owner | 미해결, source/PDF 재확인 대기 |
| p52–53 | 각주 58–62와 본문 tail owner | 미해결, PDF 직접 재현 완료 |
| p54 | 본문·각주 가독성 분리 | 미해결, 현 revision 재확인 대기 |
| p58–59, p68–70, p76–80 | reset-tail/그림 49/그림 51·표/각주 owner | 해결 — Stage 13/14/17 및 최신 shared regression |
| p66–67 | 표 23, 각주 76–85, 본문/각주 separation | 미해결 |
| p83–85 | 각주 126–136 및 본문 tail | 미해결, PDF 직접 재현 완료 |
| p87, p99–100 | semantic flow 차이 | 과거 사용자 보고, 미분석·미재검증 |
| p90, p94 | 표 27/28 row fragment owner | 미해결, PDF 직접 재현 완료 |
| p106 | 표 29 footer/page number와 fragment | 미해결, PDF 직접 재현 및 자동 후보 재포착 |
| p107–108 | 본문 tail·각주 1·그림 52 owner | 미해결, PDF 직접 재현 완료 |
| 전체 | native HWP 219쪽 ↔ PDF 215쪽 page-map | 미해결 — 개별 owner 확정 전 전역 보정 금지 |

## 완료 조건

- p44–45의 original owner 결함이 현재 revision에서 해결됐음을 source·PDF·render tree·text ledger로
  분리해 기록한다.
- p44 table stroke가 `column_text_flow_collapse` false positive를 만들지 않는다.
- p127의 Square 그림/text physical-overlap positive regression은 유지한다.
- 결과 visual evidence를 별도 문서와 asset으로 고정하고, 위 이월 원장은 다음 Stage에도 누락 없이 옮긴다.

## 결과

code commit `982e50dbf0142c5ce97ccd71650fb76ee447c47c`은 Body bbox를 flow-frame으로 사용하고
Body table raster만 column-text-flow 계산에서 mask했다. p44–45의 source/PDF owner 재판정, direct
`fidelity_compare` ledger, 선택 visual sweep과 p127 무회귀 근거는
[Stage 25 visual sweep](task_m100_3738_stage25_visual_sweep.md)에 고정한다.
