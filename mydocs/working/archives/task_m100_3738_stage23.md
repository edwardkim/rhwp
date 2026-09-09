---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 23 — HWP p43 본문·각주 physical collision 재현과 보정

## 출발 근거

Stage 22는 그림 56·64의 next-page owner만 해소했고, code/evidence commit
`a5612fd2534dbe5ccf4b85e330769213ce30f93c` / `7cd98ce5b` 뒤 native HWP는 219쪽이다. 그러나
유지보수자가 한컴 PDF와 rhwp Studio 화면을 직접 대조한 HWP p43에는 본문 마지막 줄이 각주 separator·첫
각주와 겹쳐 보이는 결함이 남아 있다. 기준 PDF p43은 본문을 끝낸 뒤 separator 아래의 39)–44) 각주가
분리되어 있다.

이 결함은 Stage 19 selected native sweep에서 `FootnoteArea y=741.6`와 보이는 TextLine 사이 bbox overlap이
없다는 자동 결과와 모순된다. 따라서 그 flag 0은 해결 근거가 아니라, 실제 paint/line-height/첫 각주
baseline 또는 WASM draw path를 보지 못한 **false negative 후보**로 취급한다. 사용자 UI와 한컴 PDF가
독립 기준이며, SVG의 단순 node bbox만으로 완료를 선언하지 않는다.

## 입력·정답지·재현 범위

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 동일 개인정보 제거 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- human p43 = zero-based page index 42

이번 Stage의 첫 산출물은 p43 source para/control·각주 39–44와 PDF/SVG/WASM의 physical y-coordinate를
한 표에 기록하는 것이다. 그 뒤에만 actual footnote reservation, composed line height, separator paint,
page-tail split 중 하나로 원인을 좁힌다. 전체 215↔219 page count를 맞추기 위한 전역 보정은 금지한다.

## 이월 원장

아래는 Stage 19·22와 유지보수자 직접 대조에서 남은 항목을 누락 없이 옮긴 것이다. `현재 재현`으로
표시되기 전까지는 과거 관측을 현재 결함이라고 과장하지 않으며, p43 해결을 이 표 전체의 해결로
간주하지 않는다.

| 우선순위 | human 쪽 | 계약 | 현재 상태 |
| --- | --- | --- | --- |
| P0 | 43 | 본문 tail과 각주 39–44가 separator를 경계로 분리된다. | **해결 — Stage 23 p43 visual evidence** |
| P0 | 26–27 | 각주 26)의 physical owner가 PDF와 같다. | 이월, source/PDF 재확인 대기 |
| P0 | 44–45 | 표 20 뒤 본문 마지막 줄의 physical owner가 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P0 | 52–53 | 각주 60–62의 page owner/내용이 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P0 | 54 | 본문과 각주 영역이 가독성 있게 분리된다. | 이월, source/PDF 재확인 대기 |
| P0 | 66–67 | 표 23·각주 76–85의 fragment와 본문/각주 분리가 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P0 | 83–85 | 각주 130–136, 본문 tail의 owner와 separation이 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P0 | 90 | 표 27 관계 행/기타 continuation의 row owner가 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P0 | 94 | 표 28 마지막 행의 physical owner가 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P0 | 106 | 표 29가 footer/page number를 침범하지 않고 PDF row boundary에서 분할된다. | Stage 19 PDF 직접 재현 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52의 physical owner가 PDF와 같다. | Stage 19 PDF 직접 재현 |
| P1 | 전체 | PDF 215쪽과 native HWP 219쪽의 page-map 차이를 개별 owner 확정 뒤 설명한다. | 개별 P0 뒤에만 분석 |

p23–24, p30–31, p77–80은 유지보수자 최신 직접 대조에서 정상으로 판정됐고, p25와 p37은 각각 Stage 19/20
선행 commit에서 다룬 항목이므로 이 Stage의 미해결 원장에 되넣지 않는다.

p127은 Stage 22가 그림의 physical owner만 고정했을 때의 follow-up 결함이었다. Stage 24 code
`775370b2f48339f84ee627bb058b94444c9ed933`은 다음 page에 `pi=1356` wrap anchor를 함께 전달하고,
stored `sw`에서 paragraph margin을 제외해 그림 56의 좌측 boundary에서 본문을 끝낸다. Stage 24 visual
evidence `916a67869`은 p127의 `column_text_flow_collapse=0`, image/TextLine physical non-overlap,
`fidelity_compare`의 `square_wrap_text_overlap=0`을 고정했다. 분홍 그림 내부 raster 때문에 남는
`question_marker_flow_drift`는 review를 요구하는 false-positive 후보로 남기되, p127은 이 Stage의
미해결 원장에서 제거한다.

## 분석·수정 계약

1. 현재 revision으로 p43 HWP→SVG/render tree와 기준 PDF p43을 다시 얻어, body tail·separator·각주
   첫/마지막 baseline, footer의 실제 paint geometry를 대조한다. native와 이미 사용자가 확인한 WASM 빌드의
   경로가 다르면 결과를 분리해 기록한다. WASM은 다시 빌드하지 않는다.
2. source의 para/control/footnote anchor가 같은 physical page에 있어야 하는지 PDF text layer와 HWP
   record를 함께 확인한다. 단순 bbox가 비겹침이어도 glyph ascent/descent 또는 paint order가 겹치면
   결함으로 인정한다.
3. 원인이 확정되면 native HWP5의 해당 source contract에 한정한 최소 보정과 focused regression을 추가한다.
   p54·p67·p85가 같은 코드 경로인지 확인 전에는 함께 바꾸지 않는다.
4. 코드 변경은 먼저 commit하고, 그 정확한 revision에서 p43 PDF direct visual sweep 및 evidence를 별도
   document/asset으로 고정한다. 잔여가 있으면 evidence commit 뒤 다음 Stage로 다시 이월한다.

## 원인 확정 — source reset과 full-fit 우회

source `pi=512`에는 inline control이 없다. 문제의 각주 39–44는 앞 본문 `pi=505`(39), `pi=506`(40–42),
`pi=507`(43), `pi=511`(44)에서 이미 p43의 Body footnote로 등록된다. `pi=512`의 stored `LINE_SEG`는
`vpos=[44000,46000,48000,0]`, `textStart=[0,58,131,190]`이며, 마지막 `vpos=0` 줄
`(47.7%)이었음.`이 한컴 PDF p44로 넘어가는 reset tail이다.

실제 p43 body 높이는 `956.2px`, 이미 등록된 각주 39–44의 composed FootnoteArea는 `297.8px`이고 본문
좌표의 경계는 `658.4px`다. pi=512의 앞 세 줄 bottom은 `600.0/626.7/653.3px`로 경계 안에 있지만,
넷째 줄 top은 `666.7px`로 경계를 넘는다. 기존 `first_vpos >= body_height * 0.7` 추정 guard는 첫 줄
`586.7px`(61.4%)를 탈락시켜 이 exact geometry를 보지 못했다.

또한 exact body-footnote boundary helper가 split loop에서만 호출되면, `pi=512` 전체가 현재 공간에 든다는
full-fit early return을 우회하지 못한다. 보정은 전역 각주 예약값을 바꾸지 않는다. native HWP5·단일 단·
control 없는 visible 본문·Body/non-fragment 각주만 대상으로, source/flow 오차 2px 이내와 reset 직전/직후의
실제 FootnoteArea crossing을 모두 만족할 때 얻은 break line을 기존 `forced_page_break_line` chain에 넣는다.
따라서 full-fit과 line split이 같은 경계를 적용하며, p43에서는 1–3줄과 각주 39–44를 유지하고 4줄만 p44로
보낸다.

## 완료 기준

- p43에서 본문 glyph와 separator/각주 glyph가 겹치지 않고, 39)–44)의 physical owner가 기준 PDF와 같다.
- source contract를 고정한 focused regression이 통과한다.
- code revision과 분리된 visual evidence가 HWP/HWPX/PDF provenance·PNG·자동 후보/사람 판정을 남긴다.
- 위 이월 원장 중 미처리 항목은 다음 Stage 문서에 다시 전부 옮긴다.

## 결과

code commit `659e1efca6453ce8510f679da1e2b4ace7362f6f`에서 p43의 physical separator collision을 해소했고,
focused regression 14/14와 p42–45 한컴 PDF visual sweep을 완료했다. p43 완료 근거와 p44–45 residual은
[Stage 23 visual sweep](task_m100_3738_stage23_p43_visual_sweep.md)에 고정한다.
