---
kind: investigation
status: active
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-03
---

# Task #3738 Stage 31 — p90 표 27 continuation row owner

## 시작 근거

[Stage 30](task_m100_3738_stage30.md)은 p83–85을 current exact native/PDF pair에서 재현하지
못한 사실을 증적으로 고정했다. 사용자 PDF 대조의 p90은 다르다. rhwp는 표 27의
`이식대상자와 관계` row를 p91로 넘기는 반면, 한컴 2020 PDF는 그 row까지 p90에 두고 다음
`기타` row만 p91로 시작한다. 이 Stage는 table fragment candidate를 결함 판정으로 오용하지 않고
source row identity·PDF row owner·render-tree fragment를 함께 대조한다.

## 분석 계약

1. p90–91 direct `fidelity_compare --text-only --layout-ledger`에서 PDF/SVG text owner와
   table fragment candidate를 고정한다.
2. 표 27의 source `(pi, ci)`, row range, cell height/padding/rowspan, p90/p91 fragment boundary를
   dump/render-tree/PDF raster로 대조한다.
3. PDF가 p90에 유지한 row가 rhwp의 body/footer reservation 또는 RowBreak fit tolerance 중 어느
   쪽에서 밀리는지 계량한다. whole-table/next-page heuristic으로 일반화하지 않는다.
4. 현재 HWP actual fixture의 focused regression으로 p90/p91 row owner와 frame/footer non-overlap을
   고정한 뒤, exact binary visual sweep과 result evidence를 별도 commit으로 보관한다.
5. p66–67의 table-cell footnote fragment와 p90의 ordinary table row fit은 독립 계약으로 취급한다.

## 초기 재현 명령

```bash
RHWP_BIN=target/review-pr3740-stage29/release-test/rhwp \
python3 tools/fidelity_compare/fidelity_compare.py 89 90 \
  --source 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --reference-pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --label issue3738-stage31-p90-p91 --reference-grade '한컴 2020 기준 PDF' \
  --text-only --layout-ledger \
  --out-dir /private/tmp/rhwp-stage31-fidelity-p90-p91-20260803
```

## 원인 분석

baseline direct fidelity에서는 p90 `reference_only=61`, p91 `svg_only=54`이고, pi=962의
table fragment ledger가 p90→p91 row owner 후보를 냈다. `RHWP_TABLE_DRIFT=1` 진단은 첫
fragment의 `current_height=613.3`, 기존 note 141을 포함한 `base=956.2`, `footnote=31.4`,
일반 safety margin `40.0`을 보였다. 따라서 row scan의 가용 공간은 249.2px에 그쳐 rows
0–4(243.6px)까지만 소비했다.

실제 FootnoteArea 경계는 924.8px(`956.2 - 31.4`)이며, 표의 첫 조각 오버헤드를 뺀
가용 공간은 289.2px다. rows 0–5는 274.3px로 그 경계 안에 들어가고 마지막 row 6은
83.0px라 들어가지 않는다. 즉, 한컴 PDF의 p90은 `이식대상자와 관계` row까지, p91은
`기타` row부터 시작하는 정확한 physical boundary다.

## 수정 계약

`typeset_block_table_inner`는 native HWP5·비-TAC TopAndBottom·RowBreak·rowspan 없음·표
자체 각주 없음·기존 Body 각주 있음·본문 하반부·다음 stored paragraph의 vpos rewind라는
교집합에서만 첫 fragment row scan을 실제 FootnoteArea 경계로 계산한다. 일반 safety
margin, table-cell footnote, rowspan, internal-reset 표는 기존 계약을 유지한다. scan 자체가
actual boundary를 상한으로 사용하므로 본문/각주 overlap을 허용하지 않는다.

focused regression은 p90의 relationship row owner, p91의 `기타` 재개, native page count
219 유지, pi=962 bottom ≤ note 141 separator를 고정한다.

## 이월 원장

| 우선 | human 쪽 | 계약 | 상태 |
| --- | --- | --- | --- |
| review | 42 | `question_marker_flow_drift`의 실제 body-flow 결함 여부 | 확정 전 후보 |
| P0 | 156 | 그림 64 Square wrap 본문 침범 | 전수 fidelity preflight의 `pi=1692/ci=1`, 12행 교차 후보 — PDF review 전 확정하지 않음 |
| P0 | 90 | 표 27 continuation row owner | **현재 Stage** |
| P0 | 94 | 표 28 마지막 row owner | 미해결 |
| review | 87 | semantic flow | 과거 보고, 미재검증 |
| review | 99–100 | semantic flow | 과거 보고, 미재검증 |
| P0 | 106 | 표 29 footer/page number 및 row boundary | 미해결 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52 owner | 미해결 |
| P1 | 전체 | PDF 215 ↔ native HWP 219 page-map | 미해결, individual P0 뒤 분석 |

p23–24, p25, p26–27, p30–32, p37, p43, p44–45, p52–54, p58–59, p66–67, p68–70,
p76–80, p83–85, p127은 최신 evidence에서 resolved 또는 current exact revision에서 재현되지
않아 재이월하지 않는다. 단, p156은 p127과 같은 과거 Stage 24 범주였지만 현재 full fidelity
preflight에서 다시 후보가 되었으므로 위 P0 원장에 독립적으로 남긴다.
