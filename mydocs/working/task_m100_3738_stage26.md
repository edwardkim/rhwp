---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 26 — p26–27 각주 26 owner 재현과 잔여 원장

## 시작 조건

Stage 25 code/evidence commit `982e50dbf0142c5ce97ccd71650fb76ee447c47c` /
`c2b687d968c4e720fb8a2f5db99c7eaaf695f7d3`는 p44–45를 새 결함으로 고치지 않았다. source
`pi=516` reset tail의 현재 physical owner가 PDF와 이미 일치함을 확인하고, centered table 19·20이
`column_text_flow_collapse`로 오인되는 detector false positive만 제거했다. 따라서 p44–45는 아래
미해결 원장에 넣지 않는다.

이번 Stage의 첫 항목은 유지보수자 직접 대조로 남은 HWP p26–27의 **각주 26 physical owner**다.
과거 관측은 rhwp p26에 보인 각주 26)이 한컴 PDF에서는 p27에 있어야 한다는 것이다. 같은 번호의
footnote anchor·source fragment·physical page owner를 먼저 확정하기 전에는, p52·p66 등 다른 각주
결함과 한 보정으로 묶지 않는다.

## 비교 기준과 재현 경계

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 같은 개인정보 제거 HWPX: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx`
- 한컴 2020 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- HWP SHA-256: `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`
- PDF SHA-256: `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`

HWP가 renderer 입력이고 위 PDF가 physical-layout 정답지다. user page number는 1-based이며,
CLI/render tree의 page index는 0-based로 병기한다. native HWP 219쪽 ↔ PDF 215쪽 차이를 맞추려는
전역 page-break 보정은 금지한다.

## p26–27 분석 계약

1. 현재 revision에서 PDF p26–27의 text/raster와 rhwp SVG/render tree를 함께 생성한다.
2. source의 각주 26 anchor, 관련 Body paragraph/LineSeg, FootnoteArea separator와 actual text-line
   bottom을 page별로 기록한다.
3. `fidelity_compare --text-only --layout-ledger`가 이 owner 이동을 어떤 signal로 후보화하는지
   확인한다. 0이라는 결과만으로 해결이라 하지 않고, PDF visual owner가 계약의 판정 기준이다.
4. 원인이 fragment/reservation/page-tail split 중 무엇인지 code path까지 좁힌 뒤, 해당 source shape에
   한정한 focused regression과 최소 Rust 수정만 한다.
5. code commit 뒤에는 정확한 binary로 p26–27 PDF visual evidence를 별도 문서·asset으로 보존한다.
   잔여가 하나라도 있으면 결과를 과장하지 않고 commit한 뒤 다음 Stage 원장에 아래 항목 전체를 이월한다.

## 현재 결함과 detector 원인 확정

현재 binary `target/review-p127-audit/release-test/rhwp` (SHA-256
`402d607d0690f4407bd8feb8e07eedcafd73ef33ee164abc418d7d86198e56ef`)에서 direct pair를
zero-based `25 26`으로 실행했다. requested/completed는 p26–27 모두이고 run state는 complete다.

| human 쪽 | PDF | rhwp | automatic signal |
| --- | --- | --- | --- |
| p26 | 본문 끝의 marker `26)`만 남고 각주 내용은 p27 | `26) 11번 참고문헌 내 Adam et al 논문` 전체가 FootnoteArea에 너무 이르게 있음 | `svg_only=21`, `body_footnote_lines=1` |
| p27 | 위 각주 26의 physical owner | 해당 각주 없음 | `reference_only=21` |

두 쪽의 문자 Counter 교집합은 정확히 21/21(양쪽 coverage 1.000)이다. 따라서 이 경우는
문자 소실이 아니라 **rhwp가 PDF보다 한 쪽 이르게 배치한 page-owner 이동**이며, PDF visual review와
render-tree physical location까지 같은 결론을 낸다. `visual_sweep.py`의 p26–27 targeted
run은 `flags=[]`였으므로, line-band raster 신호만으로 이 owner 결함을 자동 승격하지 못했다.

source parent는 native HWP `pi=373`(text length 223, inline Footnote control 1개)이다. 마지막 body
line에 marker가 있고 네 stored line은 `vpos=62915/64915/66915/68915`; 다음 `pi=374`의 첫 line만
`vpos=0` reset이다. typeset은 pi=373을 `cur_h=945.5px`까지 먼저 배치한 뒤 각주를 등록해 usable
height를 약 `916.2px`로 줄인다. 즉 이미 배치된 body를 reflow하지 않아 각주만 p26에 붙는다.

관련 route는 `src/renderer/typeset.rs`의 footnote target/registration (`5819–5899`)이며, 기존
`native_hwp5_first_footnote_overlap_break_line` (`1919–2008`, 호출 `5285–5308`)은 **같은 paragraph**의
`vpos=0` reset만 찾는다. 이 fixture의 page boundary는 다음 pi=374에 있으므로 기존 p31 two-line
fragment/p43 reset-tail 경로로 일반화하면 안 된다.

## Stage 26 detector 보정 계약

`fidelity_compare`는 raw `text-report.tsv`에 page별 Counter만 남기던 상태였다. 이번 보정은 renderer
결함을 해결하는 척하지 않고 다음 two candidate-only ledgers를 추가한다.

1. `text-owner-shift-candidates.tsv`: pN의 SVG-only와 pN+1의 PDF-only(또는 그 역방향)가 8자 이상,
   양쪽 75% 이상 일치하면 `rhwp_earlier_than_reference` 또는 `rhwp_later_than_reference`로 묶는다.
   p26–27의 21/21처럼 physical owner review의 우선 후보를 바로 만든다.
2. `page-count-ledger.tsv`: 기준 PDF, full SVG export, full render tree 페이지 수를 scope와 함께
   기록한다. `219↔215` drift는 개별 owner 조사 후보이며, 자동 global page-break 보정 근거가 아니다.

Square/Tight/Through 그림의 same-page body 침범은 이미 `square_wrap_text_overlap`이 담당한다. 반면
표 row split, 같은 문자 수의 same-page overlap, image-caption 관계는 이 Counter/geometry 범위 밖이므로
visual sweep/PDF review로 남긴다.

## 전수 잔여 원장

`해결` 항목은 다시 미해결로 되돌리지 않는다. `확정 전`과 `과거 보고`는 실제 결함 수로 세지 않되,
재검증 전까지 누락하지 않는다.

| 우선 | human 쪽 | 계약 | 상태·최신 근거 |
| --- | --- | --- | --- |
| P0 | 26–27 | 각주 26의 physical owner가 PDF와 같다. | **이 Stage의 first item**. Stage 19 직접 관측, source/PDF/render tree 재확인 필요. |
| review | 42 | `question_marker_flow_drift`가 실제 body-flow 결함인지 구분한다. | 확정 전 후보. p127 그림 내부 glyph 오탐과 혼동하지 않게 별도 review한다. |
| P0 | 52–53 | 각주 58–62와 본문 tail의 page owner/내용이 PDF와 같다. | 미해결, Stage 19 PDF 직접 재현. |
| P0 | 54 | 본문과 각주가 읽을 수 있게 분리되고 owner가 PDF와 같다. | 미해결, 현 revision source/PDF 재확인 대기. |
| P0 | 66–67 | 표 23, 각주 76–85, body/footnote separation이 PDF와 같다. | 미해결, p66 각주 77 누락 및 p67 overlap 직접 재현. |
| P0 | 83–85 | 각주 126–136, 본문 tail, 84–85 page flow가 PDF와 같다. | 미해결, p83 각주 130 누락·p85 overlap 포함, PDF 직접 재현. |
| review | 87 | semantic flow가 PDF와 같은지 확인한다. | 과거 사용자 보고, 아직 source/PDF 재분석 전. |
| P0 | 90 | 표 27의 관계 행·기타 continuation row owner가 PDF와 같다. | 미해결, PDF 직접 재현. |
| P0 | 94 | 표 28 마지막 행의 physical owner가 PDF와 같다. | 미해결, PDF 직접 재현. |
| review | 99–100 | semantic flow가 PDF와 같은지 확인한다. | 과거 사용자 보고, 아직 source/PDF 재분석 전. |
| P0 | 106 | 표 29가 footer/page number를 침범하지 않고 PDF row boundary에서 분할된다. | 미해결, PDF 직접 재현 및 automatic candidate 재포착. |
| P0 | 107–108 | 본문 tail·각주 1·그림 52의 physical owner가 PDF와 같다. | 미해결, PDF 직접 재현. |
| P1 | 전체 | PDF 215쪽과 native HWP 219쪽의 page-map 차이를 개별 owner 이후 설명한다. | 미해결. 개별 P0 해결 전 전역 보정 금지. |

다음은 이미 해결되어 이 원장에는 넣지 않는다: p23–24, p25, p30–32, p37, p43,
p44–45, p58–59, p68–70, p76–80, p127. 특히 p44–45는 Stage 25 direct PDF/source/render-tree
재판정으로 resolved이며, Stage 23의 stale false-positive 이월보다 최신 evidence가 우선한다.

## 완료 조건

- p26–27의 actual footnote 26 source anchor와 PDF physical owner를 재현 가능한 자료로 확정한다.
- 수정이 필요하면 regression·최소 code change·PDF visual evidence를 분리된 commits로 남긴다.
- 수정이 불필요하면 그 이유와 false-positive/이미 해결 근거를 명시하고 다음 잔여 항목을 진행한다.
- 다음 Stage가 생기면 위 잔여 원장을 상태 변화와 함께 전부 옮긴다.

## 결과

`8f84a5ecb0bfd4ee556239eaee3679946df10e02`에서 p26–27처럼 인접 쪽으로 이동한 text owner와
PDF↔renderer total page-count drift를 candidate ledger로 자동 분류했다. p26 renderer 결함 자체는
남아 있으므로 다음 Stage에서 별도 native HWP5 보정으로 진행한다. exact PDF visual/sweep evidence는
[Stage 26 visual sweep](task_m100_3738_stage26_visual_sweep.md)에 고정한다.
