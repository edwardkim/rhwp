---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-03
---

# Task #3738 Stage 30 — p83–85 각주 126–136 physical flow

## 시작 근거

[Stage 29](task_m100_3738_stage29.md)의 code/evidence commits
`e9ff9fb7e31df9c9e33ba6fafbdb129bb559f524` /
`41a5af904ed6a3c53d86a5e8afd2fd00630ad98f` /
`4c6b92d49`는 p66–67 표 23의 note 77 fragment owner와 body/footnote overlap을 해소했다.
사용자 PDF 대조에서는 p83의 각주 수량, p84–85의 본문·각주 흐름, p85의 본문/각주 overlap이 여전히
지적됐다. 이 Stage는 이전 보고의 완료 표기를 재사용하지 않고 현재 exact binary와 한컴 2020 PDF로
p83–85을 다시 판정한다.

## 분석 계약

1. p83–85 direct `fidelity_compare --text-only --layout-ledger`로 text Counter, ordered owner,
   table fragment candidate, body/footnote geometry를 먼저 고정한다.
2. 기준 PDF의 p83 notes 126–130, p84 notes 131–133, p85 notes 134–136과 native HWP render
   tree의 physical owner·separator·body bottom을 각각 대조한다. text extraction의 line wrapping만으로
   source owner를 단정하지 않는다.
3. p84→85 body flow는 paragraph/line-seg reset, table/shape가 있으면 그 source identity까지 분리한다.
4. code change는 p83–85 actual HWP focused regression으로 고정하고, exact binary PDF visual sweep과
   evidence는 별도 commit으로 보관한다.
5. p66–67의 table-cell fragment queue를 일반 footnote/다른 페이지에 확장하지 않는다. shared cause가
   evidence로 입증될 때만 가장 좁은 범위로 재사용한다.

## 초기 재현 명령

```bash
RHWP_BIN=target/review-pr3740-stage29/release-test/rhwp \
python3 tools/fidelity_compare/fidelity_compare.py 82 84 \
  --source 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --reference-pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --label issue3738-stage30-p83-p85 --reference-grade '한컴 2020 기준 PDF' \
  --text-only --layout-ledger \
  --out-dir /private/tmp/rhwp-stage30-fidelity-p83-p85-20260803
```

## 이월 원장

| 우선 | human 쪽 | 계약 | 상태 |
| --- | --- | --- | --- |
| review | 42 | `question_marker_flow_drift`의 실제 body-flow 결함 여부 | 확정 전 후보 |
| P0 | 83–85 | notes 126–136·본문 tail·p84–85 physical owner가 PDF와 같다. | **현재 exact revision에서 재현 안 됨 — Stage 30 evidence** |
| review | 87 | semantic flow | 과거 보고, 미재검증 |
| P0 | 90 | 표 27 continuation row owner | 미해결 |
| P0 | 94 | 표 28 마지막 row owner | 미해결 |
| review | 99–100 | semantic flow | 과거 보고, 미재검증 |
| P0 | 106 | 표 29 footer/page number 및 row boundary | 미해결 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52 owner | 미해결 |
| P1 | 전체 | PDF 215 ↔ native HWP 219 page-map | 미해결, individual P0 뒤 분석 |

p23–24, p25, p26–27, p30–32, p37, p43, p44–45, p52–54, p58–59, p66–67, p68–70,
p76–80, p127은 최신 evidence에서 resolved이므로 재이월하지 않는다.

## 결과

현재 Stage 29 exact binary에서는 사용자 화면에서 지적된 p83–85 결함을 재현하지 못했다. PDF와 SVG
모두 p83 notes 126–130, p84 notes 131–133, p85 notes 134–136을 같은 physical page에 두며,
direct text owner/ordered owner/layout/table candidate는 전부 0이다. p84→85의 reset tail과 p85
본문/각주는 PNG 직접 대조에서도 서로 침범하지 않는다. 따라서 이 Stage는 새 code change를 만들지
않고 현재 revision의 반증·provenance를 고정한다. 과거 화면이 다른 binary/pkg를 사용했는지는 이
native evidence만으로 단정하지 않는다.

상세 PDF/SVG review와 ledger는 [Stage 30 visual sweep](task_m100_3738_stage30_visual_sweep.md)에
고정했다. 다음 Stage는 p90 표 27 continuation row owner를 독립적으로 다룬다.
