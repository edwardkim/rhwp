---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 28 — p52–53 각주 58–62 physical owner 분석

## 시작 근거

Stage 27 code/evidence commits `c893d9889bca37f688e9195e8a02e7aa5ca951fb` /
`1df0d828444abf46e34714a9ee621c706a1e0f2b`는 p26–27 각주 26 owner를 해결했다. 이 Stage는
p52–53에서 사용자가 직접 확인한 footnote quantity/owner 불일치를 다음 P0 항목으로 다룬다.

한컴 2020 PDF에서 p52는 각주 58–60, p53은 이어지는 본문과 각주 61–62가 물리적으로 읽혀야 한다.
이전 rhwp 관측은 p52에 58–59만 남고 60이 p53 쪽으로 이동했으며, p53의 본문/각주 흐름도 PDF와
달랐다는 것이다. 이는 기존 p26의 단일 marker의 한 쪽 이른 owner와 같은 원인이라고 가정하지 않는다.

## 분석·수정 계약

1. source control, stored `LINE_SEG`, physical render-tree FootnoteArea, PDF p52–53 text/bbox를 먼저
   함께 기록한다.
2. `fidelity_compare --text-only --layout-ledger`의 per-page text, adjacent owner-shift, body/footnote
   geometry가 실제 결함을 어느 범위까지 후보화하는지 확인한다.
3. root cause가 확정되기 전에는 p26 helper를 넓히거나 global page count를 맞추지 않는다.
4. 수정이 필요하면 실제 HWP fixture focused regression과 최소 renderer 변경을 먼저 commit하고,
   그 exact binary의 visual sweep/PDF evidence를 결과 문서로 분리한다.
5. residual이 있으면 결과를 과장하지 않고 commit한 뒤 다음 Stage에 아래 원장 전부를 이월한다.

## 확정 원인과 수정 범위

`pi=602`는 p52 `lines=0..4` / p53 `lines=4..9`, `pi=605`는 p53 `lines=0..13` /
p54 `lines=13..17`로 이미 split되어 있다. 각 Footnote marker는 앞 fragment에 있지만,
`typeset.rs`는 paragraph 배치 뒤 inline control을 처리할 때 current tail page를 가리킨다.

기존 `find_inline_control_target_page`는 marker의 stored line이 든 완료 page를 찾을 수 있으나,
`native_hwp5_first_footnote_overlap_break_line`의 **첫 각주 collision** 경로에서만 호출됐다.
p52/p53은 이미 notes 58–59/61이 있어 해당 guard가 `None`이고, note 60/62가 각각 tail p53/p54에
등록됐다.

따라서 native HWP5의 다음 좁은 경우에만 full Body footnote를 completed marker page로 소급 등록한다.

1. first-footnote collision 경로가 결정을 내리지 않았다.
2. marker line owner는 completed `PartialParagraph` page다.
3. 그 completed page는 이미 Body Footnote를 하나 이상 보유한다.

처음에는 2번만으로 넓혔으나, p62 note 74의 **첫** 각주까지 current reservation에서 빼 219→220쪽으로
변화시키는 회귀를 확인했다. 현재 조건 3과 기존 collision route의 `Some(None)` 결정을 보존해 p30
note 29/p31 two-line note 30의 계약을 되살렸다. 이는 모든 split-footnote 문서의 재조판 해법이 아니라,
이번 multi-note HWP5 physical-owner class의 최소 보정이다.

## 자동 후보 보강

수정 전 p52–54 direct text 비교는 `p52 reference_only=83`, `p53 reference_only=73/svg_only=21`,
`p54 svg_only=135`였지만 Counter의 상호 차분은 다른 본문 문자와 상쇄되어 owner-shift TSV를 비웠다.
`fidelity_compare.py`에 16자 이상 whitespace-normalized ordered sequence ledger를 추가해 다음을
candidate로 잡는다.

| PDF owner → rhwp owner | sequence | chars |
| --- | --- | ---: |
| p52 → p53 | note 60 ConsensusStatementShort URL | 83 |
| p53 → p54 | note 62 KDIGO citation | 135 |

`text-owner-sequence-candidates.tsv`는 후보 전용이다. NFC/공백 정규화, multiline target과 chain,
same-page reorder false positive를 unit test로 고정했고, 이 신호만으로 visual fidelity를 확정하지 않는다.

## 결과

코드 commit `543c5b3988512945c1273368a25a1edfd97d15c3`는 first-footnote collision route를 보존한 채,
native HWP5의 completed multi-note marker page에만 note 60/62를 소급 등록한다. fixture regression은
16/16, fidelity comparator unit test는 25/25 통과했다.

새 binary의 p52–54 direct `fidelity_compare --text-only --layout-ledger`는 세 쪽 모두
`reference_only=0`, `svg_only=0`, Counter owner shift 0건, ordered owner sequence 0건,
`body_footnote_lines=0`을 기록했다. selected visual sweep도 p52–54 모두 complete, flag 0건이다.
PDF review PNG를 직접 확인해 p52 note 60, p53 note 62, p54 후속 각주·본문이 원래 physical owner와
같음을 확인했다. 전역 page count는 native 219 / PDF 215로 기존 `+4` candidate가 남아 있으며 이
Stage에서 해결됐다고 주장하지 않는다. 상세 증적은
[Stage 28 visual sweep](task_m100_3738_stage28_visual_sweep.md)에 고정했다.

## 이월 원장

| 우선 | human 쪽 | 계약 | 상태 |
| --- | --- | --- | --- |
| review | 42 | `question_marker_flow_drift`의 실제 body-flow 결함 여부 | 확정 전 후보 |
| P0 | 52–54 | 각주 58–62와 본문 tail owner/내용·separation이 PDF와 같다. | **해결** |
| P0 | 66–67 | 표 23·각주 76–85·body/footnote separation | 미해결 |
| P0 | 83–85 | 각주 126–136·본문 tail·84–85 page flow | 미해결 |
| review | 87 | semantic flow | 과거 보고, 미재검증 |
| P0 | 90 | 표 27 continuation row owner | 미해결 |
| P0 | 94 | 표 28 마지막 row owner | 미해결 |
| review | 99–100 | semantic flow | 과거 보고, 미재검증 |
| P0 | 106 | 표 29 footer/page number 및 row boundary | 미해결 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52 owner | 미해결 |
| P1 | 전체 | PDF 215 ↔ native HWP 219 page-map | 미해결, individual P0 뒤 분석 |

p23–24, p25, p26–27, p30–32, p37, p43, p44–45, p58–59, p68–70, p76–80, p127은 최신
evidence에서 resolved이므로 재이월하지 않는다. 다음 Stage는 p52–54도 재이월하지 않고, 표 23/각주
76–85가 시작되는 p66–67부터 위의 나머지 원장을 전부 옮긴다.
