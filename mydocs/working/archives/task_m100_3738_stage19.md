---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 19 — 잔여 PDF 대조 결함 이월

## Stage 18에서 완료된 범위

Stage 18 commit `3511bb5d6`는 HWP p37에서 같은 TAC 그림 하나가 반복된 빈 guide 줄 둘에 재귀속되어
세 번째 그림처럼 보이던 결함만 고쳤다. 그림 37·38 두 개의 owner/위치는
[Stage 18 visual sweep](task_m100_3738_stage18_visual_sweep.md)으로 검증했다.

이 커밋은 아래 항목을 해결하거나 전체 pagination fidelity를 회복한 것이 아니다.

## 사용자 관측 기반 미해결 목록

| 우선 | rhwp 관측 페이지 | PDF 기준 결함/재확인 계약 | 현재 상태 |
| --- | --- | --- | --- |
| P0 | 43 | 본문과 각주 영역 overlap 없음 | **현재 Studio 웹 화면에서 재현됨**; 기존 native 자동 sweep false negative |
| P0 | 25 | PDF의 두 그림 중 첫 번째 그림(그림 25)이 rhwp에서 완전히 보이지 않음 | **Stage 19 해결**; native HWP5의 same-page stale picture offset을 cell top으로 reset하고 p23–25 visual evidence·focused 9/9로 검증 |
| P0 | 26–27 | 각주 26은 p26이 아니라 p27에 physical owner를 가져야 한다 | 유지보수자 직접 관측; footnote fragment/owner 미분석 |
| P0 | 44–45 | `표 20` 뒤 본문 마지막 줄의 physical page owner가 PDF와 일치한다 | **재현 완료**; rhwp p44에 본문을 과보존하고 PDF는 해당 마지막 줄부터 p45에서 재개 |
| P0 | 52 | 각주 58–60의 physical page owner와 내용이 PDF와 일치한다 | **재현 완료**; rhwp는 58·59만 보이고 PDF p52에는 URL 각주 60도 함께 있음 |
| P0 | 53 | p53 본문 tail과 각주 61–62의 physical page owner가 PDF와 일치한다 | **재현 완료**; rhwp는 60·61만 보이고 PDF p53은 본문 마지막 줄과 61·62를 함께 보존 |
| P0 | 54 | 본문 하단과 각주 영역 overlap 없음 | 최신 native selected sweep complete, visual overlap flag 0; 웹 직접 대조 보류 |
| P0 | 66 | 표 23 아래 각주 76·77의 physical page owner와 내용이 PDF와 일치한다 | **재현 완료**; rhwp p66에는 76만 있고 PDF p66은 76·77을 모두 보존 |
| P0 | 67 | 표 23 continuation 본문과 각주 78–85 영역이 PDF처럼 분리되어 가독성을 유지한다 | **재현 완료**; rhwp p67에서 본문/각주 블록이 겹쳐 텍스트가 중첩됨 |
| P1 | 58–59 | p58 마지막 문단이 PDF처럼 가능한 범위까지 남고 이른 절단이 없어야 한다 | 미분석 |
| P1 | 68–70 | 그림 49와 캡션 `그림 49. OPTN 생존 장기기증 원칙`의 physical page owner가 PDF와 일치하며 캡션이 단독 페이지가 되지 않아야 한다 | 미분석 |
| P1 | 76 | 표 24가 PDF의 다섯 행을 모두 보존한다 | native/WASM renderer별 재현 필요 |
| P0 | 83 | 각주 126–130의 physical page owner와 내용이 PDF와 일치한다 | **재현 완료**; rhwp p83에는 126–129만 있고 PDF p83에는 130도 있음 |
| P0 | 84–85 | p84 본문 tail 및 각주 130–133, p85 후속 문단의 physical owner가 PDF와 일치한다 | **재현 완료**; rhwp p84는 130–132만 두고 PDF p84의 본문 첫 줄·각주 131–133을 p85로 이월 |
| P0 | 85 | 본문 마지막 문단과 각주 134–136 영역이 PDF처럼 분리되어 가독성을 유지한다 | **재현 완료**; rhwp p85에서 본문과 각주가 중첩됨 |
| P0 | 90 | 표 27의 `이식대상자의 관계` 행과 `기타` continuation의 physical fragment가 PDF와 일치한다 | **재현 완료**; rhwp p90에서 관계 행이 사라지고 p91 fragment도 다름 |
| P0 | 94 | 표 28의 `불특정기증 (Unspecified Donation)` 행 physical owner가 PDF와 일치한다 | **재현 완료**; rhwp는 p94에 과보존하고 PDF는 p95 continuation으로 이월 |
| P0 | 106 | 표 29 fragment가 footer/page number와 겹치지 않고 PDF와 같은 행 경계에서 분할된다 | **재현 완료**; rhwp p106 표가 page number를 침범하고 p107 continuation도 다름 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52의 physical page owner가 PDF와 일치한다 | **재현 완료**; rhwp p107이 본문을 과보존하고 p108은 각주/그림부터 시작 |
| P1 | 87, 99–100 | PDF와 semantic flow/page owner 차이 원인 분류 | 미분석 |
| P2 | 전체 | native HWP output 220쪽과 한컴 기준 PDF 215쪽의 차이를 page-map으로 분해한다 | 미분석; 단일 숫자 목표로 축소 금지 |

## 진행 순서

1. p25 empty RowBreak picture offset 원인군을 먼저 끝낸다. selected sweep으로 PDF의 그림 25·26 두 그림과
   p23–24 무회귀를 확인한다.
2. p26–27, p43–45, p52–53, p54, p66의 footnote/body owner 원인군은 PDF raster, selected visual sweep,
   render tree footnote separator/bottom 및 source fragment로 각각 재현한다. 동일 reservation 결함이라는 근거가
   있을 때만 하나의 수정으로 묶는다.
3. 각 원인군마다 분석 문서 → focused regression → 최소 코드 수정 → selected visual sweep/PNG 결과 문서 → commit의
   순서를 지킨다.
4. 모든 개별 owner가 판정된 뒤에만 220/215 page-map을 분석한다. 전체 페이지 수만 맞추는 보정은 하지 않는다.

HWP/HWPX/PDF 원본 해시와 보관 경로는 [Stage 18 visual sweep](task_m100_3738_stage18_visual_sweep.md)에 기록되어 있다.

## 최신 native 경계 재검증 — p30–31, p43, p54, p66

`target/review-planet6897-20260802/release-test/rhwp`로 다음 complete selected sweep을 이미 실행했다.

- p30–31: `/private/tmp/rhwp-stage19-p030-031-sweep/issue3738-stage19-hwp-p030-031/`
  - requested/completed `30,31`, 누락 없음, visual overlap flag 0.
  - p31 review PNG에서 각주 30은 문단/표와 분리되어 있다.
- p43·54·66: `/private/tmp/rhwp-stage19-p043-p054-p066-sweep/issue3738-stage19-hwp-p043-p054-p066/`
  - requested/completed `43,54,66`, 누락 없음, visual overlap flag 0.
  - render tree상 p43 FootnoteArea y=741.6, p54 footnote y=851.0, p66 표 bottom=952.7과 footnote y=992.3 사이에 간격이 있다.
    이 좌표는 **현재 보이는 노드끼리의 충돌 부재**만 뜻하며 p66 각주 77의 존재/owner는 판정하지 않는다.

유지보수자가 p30–31을 현재 PDF와 직접 대조해 정상임을 확인했다. 따라서 이 둘은 잔여 목록에서 제외한다.
PDF와 native의 서체 raster 차이로 ink metric을 완전 정합 지표로 쓰지 않으며, p43·54·66은 실제 Studio 웹/WASM
대조 전에는 완료로 닫지 않는다. 특히 p43은 이 문서 작성 후 Studio 웹 화면에서 본문 마지막 줄과 각주
separator/첫 각주가 충돌하는 것으로 재현됐다. 따라서 `visual overlap flag 0`은 이 결함의 부재 근거가 아니라
현재 detector의 false negative이며, p43을 즉시 P0 분석 대상으로 되돌린다. p66은 사용자 비교에서 각주 77 누락도
확인됐으므로 visual overlap flag로 완료를 주장할 수 없다.

## 2026-08-02 현재 웹/WASM 재대조 — p77–80은 잔여 목록에서 제외

동일 HWP를 현재 `localhost:7700` Studio에서 다시 열어 220쪽으로 로드한 뒤, 기준 PDF와 직접 대조했다.
웹 화면에서 p77에는 그림 51과 캡션 및 각주 103·104가 함께 있었고, p78에는 표 25의 첫 fragment와
각주 105·106만, p79에는 이어지는 표 25 fragment와 각주 107–111만 나타났다. p80은 각주 112부터
시작했으며, PDF와의 직접 화면 대조에서 본문·각주·표의 owner가 정상으로 판정됐다.

유지보수자가 웹 화면과 기준 PDF를 직접 재확인해 p78–80 모두 정상임을 확인했다. 따라서 과거 localhost
산출물에서 관측된 p76–80의 결함은 현재 native/WASM head에서 재현되지 않는다.
스크롤 중간 화면만으로 p80 본문 이월을 판정할 수 없으므로 이를 새 결함으로 등록하거나 보정하지 않는다.
단, 전체 220/215쪽 차이는 별도의 page-map 분석 대상으로 유지한다.

## 2026-08-02 p23–25 현재 판정

현재 HWP p23–24 선택 sweep은 `/private/tmp/rhwp-stage19-p023-p024-current/`에 complete 상태로 보관했다.
유지보수자가 기준 PDF와 직접 대조해 p23–24가 정상임을 확인했으므로, 과거 그림 21·22 관측은 현재 잔여
목록에서 제외한다. p25의 비정상은 별도 결함이며, p23–24의 과거 그림 23 이월 문제로 일반화하지 않는다.

## p25 그림 25 누락 — 현재 재현 근거

`/private/tmp/rhwp-stage19-p025-current/issue3738-stage19-hwp-p025-current/review/review_025.png`의
PDF pane에는 그림 25와 그림 26 두 그림이 보이지만, rhwp pane에는 첫 그림 25의 자리만 비어 있고 그림 26만
보인다. render tree에서 원본 HWP 문단 `pi=357`의 빈 1×1 `RowBreak` 표는
`Table y=243.2px`, 내부 첫 `Image y=-88.3px, h=261.7px`로 방출된다. 따라서 첫 그림은 존재하되
표의 page-local origin 위로 88.3px 이탈해 clip되어 보이지 않는다.

### 원인 범위와 최소 보정 계약

원본 `dump --para 357`은 host paragraph stored vpos가 `12000 HU`, 표의 `vOffset`이 `0 HU`,
그림의 `vOffset`이 signed `-50000 HU`임을 보인다. 현재
`stored_layout_relocated_empty_rowbreak_picture_resets_offset()`은 빈 1×1 RowBreak 표의 엄격한
형상을 이미 판정하지만, `table_offset > 0` 그리고
`host_vpos + table_offset + picture_offset ≈ 0`인 **다음 페이지로 이월된** ladder만 허용한다.
따라서 이 문단은 같은 형상인데도 `12000 + 0 - 50000 = -38000 HU`라서 보정에서 제외되고,
stale page-scale 음수 그림 offset이 그대로 적용된다.

같은 HWP의 다른 1×1 빈 RowBreak 그림 표 `pi=344`는 `host=52230`, `table=560`,
`picture=-52790 HU`로 합계가 0이어서 기존 이월 보정에 이미 해당한다. 전체 fixture에서
`table=0`이면서 이 엄격한 형상과 page-scale 음수 picture offset을 함께 갖는 항목은 `pi=357`뿐이다.
그러므로 기존 이월 ladder의 허용 범위를 넓히지 않고, native HWP5에서만 다음의 별도 계약을 추가한다.

- 이미 검증된 빈 1×1 RowBreak/자리차지/문단 기준/flow-with-text 그림 형상은 그대로 유지한다.
- 표 offset이 정확히 `0`, 그림 offset이 `-40000 HU` 이하인 page-scale 음수여야 한다.
- 이 경우 그림의 physical top은 stale absolute offset이 아니라 현재 표 cell의 page-local content top이다.

이 경계는 일반 음수 offset 그림, 다중 셀/행 표, TAC 그림, HWPX stored-layout 경로에는 적용하지 않는다.
수정 후에는 `pi=357` Image가 p25 table frame 내부에 한 번만 나타나고 그림 26 및 p23–24는 변하지 않는지를
focused regression과 selected visual sweep으로 확인한다.

## p44–45 본문 owner 경계 — 신규 사용자 재현

유지보수자가 현재 rhwp와 기준 PDF를 직접 비교했다. rhwp p44는 `표 20. 일본 생존 간 기증자의 이식대상자와의 관계`
아래의 문단 전체를 p44에 과도하게 남긴 뒤 p45를 `- 이식된 간의 종류별 합병증은 ...`으로 시작한다. 기준 PDF는
같은 문단의 마지막 줄 `되었으며, <표 20>과 같음.`부터 p45에서 재개한다. 이는 단순 raster 차이가 아니라 physical
page owner 경계 결함이다. p25의 empty RowBreak picture offset과는 source control 형상이 달라, p25 결과를 commit한
다음 Stage에서 stored vpos/paragraph fit 및 p44–45 PDF raster를 별도 분석한다.

## p52 각주 60 누락 — 신규 사용자 재현

유지보수자가 현재 rhwp와 기준 PDF를 직접 대조했다. rhwp p52의 FootnoteArea는 `58)`과 `59)` 두 항목에서 끝나지만,
기준 PDF p52에는 `60) http://www.who.int/transplantation/publications/ConsensusStatementShort.pdf?ua=1` URL 각주가
같은 physical page에 추가로 있다. 이 항목은 다음 쪽으로 단순 이월된 것으로 가정하지 않는다. 이후 Stage에서 p52와
후속 페이지의 footnote owner/fragment source를 render tree와 page text로 함께 확인한 뒤, 기존 26–27·43·54·66과 같은
reservation/fragment 원인인지 분류한다.

## p53 본문·각주 61–62 owner 불일치 — 신규 사용자 재현

같은 비교에서 rhwp p53은 `60)` URL과 `61)` URL만 표시하고 본문은 보이지 않는다. 기준 PDF p53은
`증후보자를 평가하거나 승인을 결정과정에 관여하는 자 최소 1명을 포함하여 이해상충을 최소화` 본문 tail을 먼저 두고,
`61)` URL과 `62) Lentine, Krista L., et al. ...`을 함께 둔다. 따라서 p52의 각주 60 누락, p53의 60 앞당김 및
각주 62 누락/본문 owner 불일치는 하나의 page-fragment 연쇄일 수 있으나, 아직 같은 보정을 적용하지 않는다. 후속 분석은
p52–53과 PDF p52–53을 쌍으로 고정해 각주 60–62의 source/physical fragment를 먼저 확정한다.

## p66 각주 77 누락 — 사용자 대조로 상태 정정

기존 native selected sweep의 `visual overlap flag 0`은 각주 충돌만 탐지했으므로, 내용/개수 보존을 검증하지 못했다.
유지보수자 대조 화면에서 rhwp p66은 표 23 뒤 `76)`만 보이고, 기준 PDF p66은 `76)` 및
`77) CFR → Title 42(Public Health) → Chapter IV(CMS, CENTERS FOR MEDICARE & MEDICAID SERVICES, ...`를 함께
보인다. 따라서 p66의 완료 보류 항목을 단순 overlap 검토에서 **각주 77 physical fragment 누락** 결함으로 바꾼다.
후속 분석은 표 23 첫 fragment와 table-cell footnote reservation이 77을 p67로 잘못 넘기는지 확인한다.

## p67 본문·각주 overlap — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p67은 표 23 continuation 뒤 본문(`42 CFR Part 482`, `42 CFR Part 121`,
`National Organ Transplant Act`)과 각주 78–85 블록이 같은 세로 영역을 점유해 서로 중첩된다. 기준 PDF p67은
동일 본문과 78–85를 분리된 영역에 명확히 배치한다. 이는 p66의 각주 77 누락과 같은 표 23 fragment 뒤에서 발생하지만,
현재 단계에서는 p67의 overlap을 독립적인 P0 contract로 고정한다. 후속 render-tree 분석은 body 마지막 line bottom,
FootnoteArea separator top, 각주 78–85의 physical owner를 함께 측정한다.

## p85 본문·각주 134–136 overlap — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p85는 `기증후보자에게 충분히 설명하고...`로 시작하는 본문 마지막 문단의 하단과
각주 `134)`, `135)`, `136)` URL 줄을 같은 세로 영역에 겹쳐 그린다. 기준 PDF p85는 본문을 완결한 뒤
구분선 아래에 134–136을 분리한다. 이는 p67과 동일하게 overlap detector가 반드시 잡아야 할 P0 가독성 결함이지만,
표 continuation이 아닌 일반 문단/각주 조합일 수 있으므로 p67과 보정 경로를 공유한다고 가정하지 않는다.

## p83 각주 130 누락 — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p83의 FootnoteArea는 `126)`–`129)` 네 항목만 보존한다. 기준 PDF p83에는
동일 네 항목 뒤 `130) http://www.odequs.eu/`가 같은 physical page에 추가로 있다. 따라서 기존 p83–84의
일반 page-owner 조사 항목을 p83의 **각주 130 fragment 누락** P0 contract로 구체화한다. p52–53 및 p66과 마찬가지로
후속 페이지에 단순 이월됐다고 추정하지 않고, source와 physical owner를 함께 추적한다.

## p84–85 본문·각주 fragment 흐름 불일치 — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p84는 각주 `130)`–`132)`까지 보인 뒤 p85를
`(Charter of Fundamental Rights of the European Union)...` 본문으로 시작한다. 기준 PDF p84는 이 본문 첫 줄을
이미 p84에 포함하고 각주도 `131)`–`133)`으로 끝낸다. 기준 PDF p85는 그 후속 `위한...` 본문부터 재개한다.
따라서 p83의 각주 130 owner 불일치와 p84의 130–133 분할은 연쇄될 수 있으나, p85의 본문/각주 overlap과 섞어
단일 보정으로 가정하지 않는다. 후속 단계에서 p83–85의 para/footnote source와 stored vpos reset을 함께 page-map한다.

## p90 표 27 fragment 불일치 — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p90의 표 27은 `장기 유형` 뒤에 바로 각주 141로 끝나며, 기준 PDF p90에 있는
`이식대상자의 관계` 행이 빠져 있다. rhwp p91의 후속 표는 이 관계 행과 `기타` 행을 PDF와 다른 방식으로 분할한다.
기준 PDF는 p90에 관계 행까지 포함하고 p91에는 `기타` continuation만 둔다. 따라서 p90은 단순 table raster 차이가
아닌 RowBreak table row fragment owner 결함이며, p66–67·p76–80과 같은 표 조판 경로인지 별도 분석한다.

## p94 표 28 행 owner 불일치 — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p94의 표 28은 `지정 기증`, `간접기증`, `불특정기증` 세 행을 모두 보인다.
기준 PDF p94는 `지정 기증`과 `간접기증` 두 행에서 끝나며, `불특정기증 (Unspecified Donation)` 행은 p95의
표 continuation으로 시작한다. 따라서 p94의 차이는 표 자체의 raster 차이가 아니라 RowBreak 표의 마지막 행을
현재 페이지에 과보존하는 physical fragment owner 결함이다.

## p106 표 29 footer overlap 및 fragment 불일치 — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p106의 표 29 마지막 행이 footer의 `- 106 -` 페이지 번호 영역까지 내려와 겹친다.
기준 PDF p106은 표의 첫 세 행만 안전하게 보존하고 footer 위에서 끝나며, 남은 행은 p107에서 이어진다. rhwp p107은
표 continuation의 행 구성도 PDF와 달라진다. 따라서 p106은 RowBreak 표의 actual footnote/footer boundary와 row
fragment owner를 함께 검증해야 하는 P0이며, 단순 page number 위치 보정으로 해결해서는 안 된다.

## p107–108 본문·각주·그림 52 owner 불일치 — 신규 사용자 재현

유지보수자 대조 화면에서 rhwp p107은 `모든 의뢰 사례를 평가할 법적 책임은 HTA에 있음...` 본문을 끝까지
과보존하고, p108은 각주 `1)`과 그림 52로 바로 시작한다. 기준 PDF p107은 해당 본문의 앞부분에서 끝나고,
나머지 본문 tail이 p108 상단에서 재개된 뒤 각주 1)과 그림 52가 이어진다. 따라서 이는 그림 52만의 anchor
문제가 아니라 본문 fit/physical owner가 먼저 달라진 연쇄이며, 이후 source para와 그림 anchor를 분리해 분석한다.

## Stage 19 종료

p25의 그림 25 누락만 이 단계에서 수정·검증했다. 결과와 3-way PNG는
[Stage 19 visual sweep](task_m100_3738_stage19_visual_sweep.md)에 고정했다. 나머지 목록은 해결됐다고
표시하지 않으며, 다음 Stage는 215쪽을 사람 눈으로 전수 순회하지 않도록 기준 PDF 대비 자동 후보 추출기를
범용 파일 입력·분할 실행으로 확장하는 분석부터 시작한다. 후보 순위는 코드 수정의 근거가 아니라 고해상도
selected visual sweep과 source/render-tree 분석의 우선순위를 정하는 용도로만 사용한다.
