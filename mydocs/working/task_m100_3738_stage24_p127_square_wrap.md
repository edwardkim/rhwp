---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 24 — p127 Square 그림 56 본문 침범 보정

## 문제와 정답지

유지보수자 직접 비교에서 native HWP p127은 그림 56을 기준 PDF와 같은 physical page로 옮겼지만,
그림 왼쪽으로 흘러야 할 본문이 그림 안쪽까지 침범했다. 기준은 다음의 개인정보 제거 한컴 2020 PDF다.

- HWP: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- physical page: human p127 / zero-based index `126`

HWP `pi=1355 / ci=0`은 non-TAC `Picture`, `Square`, `Para`, `Column`, `Top/Left`,
`horizontalOffset=23057HU`인 그림 56이다. 바로 다음 `pi=1356`의 15개 LineSeg는 모두
`cs=0`, `sw=23057HU`, `vpos=0…28000`이다. 이 `sw`는 그림의 왼쪽 physical boundary까지의
저장된 line box이며, 문단 hanging indent를 포함한다.

## 원인

Stage 22는 그림 PageItem을 다음 physical page로 이월했지만, 그 그림이 소유한 `pi=1356` narrow
wrap anchor를 함께 이월하지 않았다. 그 결과 layout은 `pi=1356`을 body 전체 폭으로 그려 그림 위로
13개 행을 교차시켰다.

초기 anchor 전달 뒤에도 `wrap_anchor` 경로가 `LineSeg.sw`를 TextLine usable width로 그대로 사용해,
hanging indent를 한 번 더 더했다. p127은 그림 시작 `x=401.92px`에 대해 본문 오른쪽 끝이
`419.48px`까지 들어갔다. 동일한 source contract의 그림 64/p156도 같은 폭 의미를 공유한다.

## 보정 계약

1. page-tail Square 그림이 다음 physical page를 소유한다고 좁게 확정될 때, 다음 문단 인덱스와
   source `WrapAnchorRef`를 deferred control에 함께 보관한다.
2. 새 physical page를 열어 기존 column anchor를 flush/reset한 뒤에만 그 anchor를 등록한다. 따라서
   p126/p155의 이전 page에는 누수되지 않는다.
3. `wrap_anchor` layout은 `sw - image-margin-right - paragraph-left/right-margin`을 usable text 폭으로
   쓴다. `sw`를 포함 폭으로 해석하는 일반 stored-segment path와 같은 계약이다.
4. Figure 56/p127과 Figure 64/p156에서 그림과 세로로 겹치는 모든 TextLine이 가로로 교차하지 않는
   focused regression을 둔다.

명시적 page/section/column break가 next paragraph 앞에 있는 복합 source와 여러 deferred Square 그림이
한 target을 공유하는 source는 이번 HWP fixture에 없으며, 이 contract의 일반화 전 별도 source로 검토한다.

## 검증 결과

- focused source regression 13/13: 그림 56/p127 `pi=1356`, 그림 64/p156 `pi=1693` bbox non-overlap
- Stage 9–22 regression을 유지했고, Python detector regression도 27/27 통과했다.
- 정확한 code revision `775370b2f48339f84ee627bb058b94444c9ed933`에서 p126–127 및 p155–156
  Hancom PDF visual sweep과 `fidelity_compare --layout-ledger`를 재실행했다.

완료 근거, PNG/JSON/hash, 자동 후보의 사람 판정은
[Stage 24 visual sweep](task_m100_3738_stage24_p127_visual_sweep.md)에 고정한다.
