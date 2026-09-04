---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820 Stage 10 — p171→p172 native HWP5 text tail / TopAndBottom 표 경계 가설 기각

## 최신 전수 원장과 재현

Stage 9 후 최신 `fidelity_compare --text-only --export-all-svg --layout-ledger`는 기준 PDF 215쪽,
rhwp SVG/render tree 218쪽으로 **+3**을 기록한다. p168 표 44의 first fragment와 p118/p127은
이미 해소됐고, 다음 의미 있는 owner shift는 p171→p172다.

`pi=1796`에는 두 번째 stored line 뒤 `vpos=0` reset이 있고, 직후 `pi=1797`에는 1×1 non-TAC
`TopAndBottom` 표(`<의학적 논거>`)가 이어진다. rhwp 물리 p171에는 `pi=1796` 첫 line만, p172에는
두 번째 line만, p173에는 reset tail과 표가 나타나므로 한 줄짜리 p172가 후속 내용을 연쇄 이월하는
형상이다.

그러나 기준 PDF와 rhwp는 이 경계보다 앞에서 이미 물리 page owner가 어긋나 있다. PDF p170의
`BMI 지표 … 그러나` tail은 rhwp p171→p172에, PDF p171의 `BMI와 영상 검사 …`는 rhwp p173에
있다. 따라서 같은 물리 번호 p170/p171을 직접 짝지은 초기 수용 기준은 유효하지 않으며, 이 경계를
전역 `+3`의 첫 원인으로 단정할 수 없다.

`dump-pages` source 형상은 다음과 같다.

- rhwp p171: `PartialParagraph pi=1796 lines=0..1`, first vpos `67848 HU`
- rhwp p172: `PartialParagraph pi=1796 lines=1..2`
- rhwp p173: `PartialParagraph pi=1796 lines=2..8`, line 2에서 `vpos=0` reset
- successor: `pi=1797`, 1×1, non-TAC, `TopAndBottom`, 높이 약 `310.4px`

## 기각한 보정 가설

줄 단위 split의 기존 `hwp_authoritative`는 다음 line의 `vpos=0`을 보고도 현재 generic base
available height를 초과하면 reset 직전 line을 보수적으로 이월한다. 이 fixture의 source visible
bottom은 body height 안이지만 generic safety budget 밖이라 p170의 저장 owner를 잃는다.

native HWP5 단일 컬럼 text paragraph에서 다음 조건을 만족하면 reset 직전 line을 body height까지
허용하는 좁은 보정안을 구현해 focused diagnostic으로 확인했다.

1. visible text only paragraph의 내부 stored reset이 있고 reset 직전 line bottom이 body height 안이다.
2. 현 paragraph가 body tail(60% 이후)에서 시작하며 현재 page에는 앞선 content가 있다.
3. direct successor가 visible text 없는 단일 non-TAC 1×1 `TopAndBottom` table이다.

진단상 후보 조건 자체(`native=true`, single column, footnote 없음, tail height `900.9/956.2px`,
successor가 해당 table, reset index 2)는 성립했다. 하지만 기준과의 물리 page mapping이 이미
앞에서 달라 p170/p171을 직접 assert한 회귀는 실패했다. 이 구현은 범위가 옳다는 증거 없이
커밋하지 않고 철회했다.

## 다음 단계

1. `upstream/devel` 동기화·리베이스 뒤 전수 owner ledger를 다시 만든다.
2. physical page 번호가 아니라 PDF↔rhwp의 독립적인 동일 문자열 anchor를 따라 **첫 실제 여분 page**를
   찾는다.
3. 그 anchor의 source `pi`/LINE_SEG·각주·표/그림 owner를 기준 PDF와 대조해 다음 보정 가설을 세운다.
4. 가설이 실제 첫 분기를 제거하고 page count를 줄이는 focused regression을 만든 뒤에만 구현을 커밋한다.
