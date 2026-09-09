---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 27 — p26–27 각주 26 physical owner 보정

## 시작 근거

Stage 26 detector/evidence commits `8f84a5ecb0bfd4ee556239eaee3679946df10e02` /
`d502158bb910a35aedc462adb5bf3f46599c4fb4`는 p26–27의 결함을 자동 **후보화**했을 뿐 renderer를
고치지 않았다. 한컴 2020 기준 PDF에서는 p26에 marker `26)`만 있고 실제 `26) 11번 참고문헌 내 Adam et al
논문` 각주는 p27에 있다. 현재 rhwp는 이 각주를 p26 FootnoteArea에 너무 이르게 둔다.

`text-owner-shift-candidates.tsv`의 p26→p27 `rhwp_earlier_than_reference`, 21/21 character,
양쪽 coverage 1.000와 p26 `body_footnote_lines=1`은 PDF visual owner와 같은 결론을 낸다. 이 Stage는
native HWP5의 좁은 footnote registration 보정만 다루며, 219↔215 전체 page-map을 맞추려는 전역 수정은
금지한다.

## source·원인 경계

- source parent: native HWP `pi=373`, text length 223, inline Footnote control 1개
- body LineSeg: `vpos=62915/64915/66915/68915`; marker는 마지막 body line
- 다음 relevant body paragraph: `pi=374`의 첫 LineSeg `vpos=0` reset
- 현재 route: `src/renderer/typeset.rs` footnote target/registration `5819–5899`

현재 typeset은 pi=373의 body를 `cur_h=945.5px`까지 먼저 배치하고 각주를 현재 page에 등록한다. 그 뒤
footnote height를 반영하면 usable body boundary가 약 `916.2px`로 낮아져 이미 배치된 body/footnote가
충돌한다. 기존 `native_hwp5_first_footnote_overlap_break_line`은 같은 paragraph 안의 reset만 보고,
이 fixture처럼 다음 pi의 `vpos=0` boundary는 볼 수 없다.

## 보정 계약

다음 조건이 **모두** 성립하는 native HWP5 경우에만 marker/body는 현 page에 유지하고 footnote
registration만 다음 physical page로 defer한다.

1. 최초이며 단일 composed Body footnote line이다.
2. Footnote marker가 parent paragraph의 마지막 visible body line에 있다.
3. 다음 relevant Body paragraph의 첫 stored LineSeg가 `vpos=0` reset이다.
4. note reservation 뒤 actual body/FootnoteArea collision이 존재한다.
5. explicit page/section/column break, multi-line footnote fragment, existing footnote block은 이 경로에
   들어오지 않는다.

이는 p31 two-line fragment와 p43 existing-note reset-tail 경로를 바꾸지 않는다. 구현 전에 current
HWP source/control, render tree, existing focused test fixture를 다시 대조하고 code path가 위 조건을
직접 표현하는지 확인한다.

## 검증 계약

1. focused Rust regression: p26 marker/body는 p26에 남고, 각주 26/FootnoteArea는 p27에만 남는다.
2. p27의 following body/figure flow와 native 전체 page count가 의도치 않게 변하지 않는다.
3. p31 two-line fragment, p43 footnote reset-tail, p127 Square-wrap focused regressions을 함께 유지한다.
4. exact revision에서 `fidelity_compare --text-only --layout-ledger`를 p26–27에 실행해 owner shift와
   p26 body-footnote collision이 0이 되는지 확인한다.
5. 한컴 PDF visual sweep/PNG와 source/PDF provenance를 result document/asset으로 고정한다.

## 이월 원장

| 우선 | human 쪽 | 계약 | 상태 |
| --- | --- | --- | --- |
| P0 | 26–27 | 각주 26의 physical owner가 PDF와 같다. | **이 Stage에서 수정** |
| review | 42 | `question_marker_flow_drift`의 실제 결함 여부 | 확정 전 후보 |
| P0 | 52–53 | 각주 58–62와 본문 tail owner/내용 | 미해결 |
| P0 | 54 | 본문·각주 가독성 separation과 owner | 미해결 |
| P0 | 66–67 | 표 23·각주 76–85·body/footnote separation | 미해결 |
| P0 | 83–85 | 각주 126–136·본문 tail·page flow | 미해결 |
| review | 87 | semantic flow | 과거 보고, 미재검증 |
| P0 | 90 | 표 27 continuation row owner | 미해결 |
| P0 | 94 | 표 28 마지막 row owner | 미해결 |
| review | 99–100 | semantic flow | 과거 보고, 미재검증 |
| P0 | 106 | 표 29 footer/page number 및 row boundary | 미해결 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52 owner | 미해결 |
| P1 | 전체 | PDF 215 ↔ native HWP 219 page-map | 미해결, individual P0 뒤 분석 |

p23–24, p25, p30–32, p37, p43, p44–45, p58–59, p68–70, p76–80, p127은 최신 evidence에서
resolved이므로 재이월하지 않는다. 다음 Stage가 필요하면 위 잔여 항목 전부를 상태와 함께 옮긴다.

## 결과

코드 commit `c893d9889bca37f688e9195e8a02e7aa5ca951fb`는 본문을 되감지 않고 각주 registration만
다음 physical page로 넘겼다. focused fixture 15/15가 통과했고, 새 binary의 p26–27 direct
`fidelity_compare --text-only --layout-ledger`는 `reference_only=0`, `svg_only=0`, adjacent owner
shift 0건, `body_footnote_lines=0`을 기록했다.

한컴 2020 PDF와 direct review PNG도 p26의 marker/body와 p27의 footnote 26 physical owner가
같음을 확인했다. `task1274`의 p27 `endnote_separator_gap_drift` 1건은 PDF raster의 116개 수평
candidate 중 무관한 선을 선택한 review-only 오탐이다. 이 Stage의 자세한 visual/PDF 증적과
한계 판정은 [Stage 27 visual sweep](task_m100_3738_stage27_visual_sweep.md)에 고정했다.
