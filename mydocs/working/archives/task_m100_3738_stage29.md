---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-03
---

# Task #3738 Stage 29 — p66–67 표 23·각주 76–85 physical flow

## 시작 근거

Stage 28 code/evidence commits `543c5b3988512945c1273368a25a1edfd97d15c3` /
`10a916421e9dc62c2bd4138ca40cbd0981c3da9f4`는 p52–54의 multi-note owner를 해결했다.
한컴 2020 기준 PDF와 동일 HWP의 p66–67은 다음 독립 계약을 아직 만족하지 않는다.

- p66은 표 23의 첫 RowBreak fragment 뒤에 각주 76과 77을 모두 물리적으로 보존한다.
- p67은 표 23 continuation, 후속 본문, 각주 78–85를 PDF와 같은 owner로 분리해 가독성 있게
  배치한다. Body `TextLine`과 `FootnoteArea`는 서로 침범하지 않는다.
- 이 Stage는 p66–67만의 증거로 전역 PDF 215 ↔ native 219 page-map을 해결됐다고 주장하지 않는다.

과거 Stage 9/11은 같은 표의 선예약과 FootnoteArea paint reservation을 각각 보정했지만, 이후
실제 PDF 대조에서 p66 note 77 owner 누락과 p67 body/footnote overlap이 다시 확인됐다. 따라서
이전 완료 표기를 재사용하지 않고 현재 Stage 28 head의 source control·stored line·render-tree·PDF
text/bbox를 다시 함께 판정한다.

## 분석 계약

1. p66–67 direct `fidelity_compare --text-only --layout-ledger`로 PDF/SVG text owner,
   `body_footnote_lines`, table/footer/frame, ordered owner sequence 후보를 먼저 남긴다.
2. p728 RowBreak 표 23의 fragment row 범위, table-cell footnote 76–82와 후속 body footnote
   83–85의 marker line owner·registration 순서를 분리해 기록한다.
3. 기준 PDF의 p66 note 76–77 및 p67 note 78–85 physical owner와 rhwp render-tree의
   FootnoteArea separator/bottom 및 Body bottom을 대조한다. 단순 native raster의 무충돌만으로
   내용/owner 보존을 주장하지 않는다.
4. 원인이 확정되기 전에는 Stage 9 table-footnote queue나 Stage 28 completed-page routing을
   일반화하지 않는다. 코드 수정은 실제 HWP focused regression으로 먼저 고정한다.
5. code commit 뒤에는 exact binary로 p66–67 visual sweep과 direct fidelity 증적을 별도 결과
   문서에 저장한다. 잔여가 있으면 해결로 가장하지 않고 다음 Stage로 원장을 전부 이월한다.

## 현재 재현 기준

```bash
RHWP_BIN=target/review-pr3740-stage28/release-test/rhwp \
python3 tools/fidelity_compare/fidelity_compare.py 65 66 \
  --source 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --reference-pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --label issue3738-stage29-p66-p67 --reference-grade '한컴 2020 기준 PDF' \
  --text-only --layout-ledger \
  --out-dir /private/tmp/rhwp-stage29-fidelity-p66-p67-20260802
```

이 실행과 source/PDF 경계 분석이 완료되기 전에는 현재 원인을 `p52–54`의 multi-note owner class나
일반 FootnoteArea 높이 계산과 동일하다고 가정하지 않는다.

## 확정 원인과 code 범위

현재 Stage 28 baseline은 p66 `reference_only=153`, p67 `svg_only=153`, p66→67
`rhwp_later_than_reference` owner shift 153자, p67 `body_footnote_lines=2`를 기록한다.
`table-fragment-candidates.tsv`도 같은 source `(pi=728, ci=0)` 표가 p66→p67에 이어짐을
기록한다. 이는 검토 후보일 뿐 PDF 행 owner의 판정은 아니다.

source p728은 7×2 RowBreak 표 23이며 p66에는 rows `0..5`, p67에는 `5..7`이 배치된다.
표 첫 data row의 cell footnote 77은 composer line과 정확히 일치하는 stored `LINE_SEG`의
두 번째 줄 뒤 `vpos=0` reset을 가진다. 한컴 PDF는 p66에 note 76과 **번호가 있는 note 77의
앞 fragment**를, p67에 note 77의 **번호 없는 tail** 뒤 note 78–85를 둔다.

기존 fragment queue는 table-cell note를 원자적인 `content_height`로만 등록한다. 그래서 p66의
표/기존 note 76 뒤에는 note 77 전체가 fit하지 않아 통째로 p67에 밀렸고, p67의 tail body와
FootnoteArea가 겹쳤다.

수정은 native HWP5의 다음 좁은 형상만 line fragment queue로 처리한다.

1. footnote source의 stored lines와 composed lines가 1:1이고 정확히 하나의 positive→zero reset이 있다.
2. marker cell row가 지금 확정한 single-column intermediate table fragment 안에 있다.
3. 전체 note는 fit하지 않지만 reset 앞 prefix는 table/footer guard 안에 fit한다.

이때 p66에는 separator·number를 가진 prefix를, 다음 table fragment에는 source 순서를 보존한
number 없는 tail을 등록한다. tail 뒤 새 note가 이어지는 fresh page에는 separator를 한 번 다시
예약한다. 현재 page에 이미 separator가 예약되었는지도 별도로 추적하므로, separator 없는 tail만
먼저 배치된 뒤 일반 note가 이어지는 경우에도 reserve/paint contract가 갈라지지 않는다. 행 내부에서
다시 잘린 fragment, 단순 capacity 부족·terminal fragment·HWPX·다단·marker가 다른 fragment인 경우는
기존 원자 queue를 유지한다.

동시에 `fidelity_compare --layout-ledger`는 `table-fragment-candidates.tsv`를 추가한다. 같은
`(pi, ci)` Body table의 인접 page fragment, table/footer·frame, 또는 page 하단 table과 24자 이상
PDF↔SVG text delta를 candidate로 남긴다. PDF text layer가 표 행을 추출하지 못하는 p94 유형을 포함해
자동 triage를 넓히되, visual/PDF review 전에는 결함이나 올바른 row owner로 승격하지 않는다.

## 이월 원장

| 우선 | human 쪽 | 계약 | 상태 |
| --- | --- | --- | --- |
| review | 42 | `question_marker_flow_drift`의 실제 body-flow 결함 여부 | 확정 전 후보 |
| P0 | 66–67 | 표 23·각주 76–85·body/footnote separation이 PDF와 같다. | **해결 — Stage 29 evidence** |
| P0 | 83–85 | 각주 126–136·본문 tail·84–85 page flow | 미해결 |
| review | 87 | semantic flow | 과거 보고, 미재검증 |
| P0 | 90 | 표 27 continuation row owner | 미해결 |
| P0 | 94 | 표 28 마지막 row owner | 미해결 |
| review | 99–100 | semantic flow | 과거 보고, 미재검증 |
| P0 | 106 | 표 29 footer/page number 및 row boundary | 미해결 |
| P0 | 107–108 | 본문 tail·각주 1·그림 52 owner | 미해결 |
| P1 | 전체 | PDF 215 ↔ native HWP 219 page-map | 미해결, individual P0 뒤 분석 |

p23–24, p25, p26–27, p30–32, p37, p43, p44–45, p52–54, p58–59, p68–70, p76–80,
p127은 최신 evidence에서 resolved이므로 재이월하지 않는다.

## 결과

code commits `e9ff9fb7e31df9c9e33ba6fafbdb129bb559f524`와
`41a5af904ed6a3c53d86a5e8afd2fd00630ad98f`는 native HWP5 p728의 저장 reset을 검증된
table-cell footnote fragment로만 분리하고, 번호 없는 tail 뒤 일반 note의 separator 예약도
rendered FootnoteArea와 동일하게 만든다. focused fixture는 **16/16 통과**했고, 기준 PDF direct
대조에서 p66·p67 text owner와 layout candidate는 모두 0이다. p728의 same `(pi, ci)` 인접 table
fragment는 candidate ledger에 남지만, PDF/SVG text delta가 0이므로 row owner 결함으로 승격하지
않는다.

PDF/SVG review PNG, source/PDF/binary provenance, 수정 전·후 ledger와 사람 판정은
[Stage 29 visual sweep](task_m100_3738_stage29_visual_sweep.md)에 고정했다. full native render
tree 219쪽과 기준 PDF 215쪽의 전역 page-map 차이(+4)는 이 두 physical page 해결과 별개로 다음
Stage에 계속 이월한다.
