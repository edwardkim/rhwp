---
kind: verification
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 20 candidate ledger — 한컴 PDF 215쪽 자동 후보 수집

## 목적과 비판정 범위

이 결과는 215쪽을 사람이 순서대로 화면 대조하지 않고, 한컴 기준 PDF와 현재 rhwp 산출 사이의
**검토 후보**를 한 번에 추출한 기록이다. 문자 multiset·render-tree bbox는 결함 판정 또는 해결 증명이
아니다. 폰트 추출 차이, 의도된 겹침, PDF text layer 매핑 차이도 후보가 될 수 있으므로 이후에는
candidate page만 high-DPI visual sweep/PDF source 대조로 확정한다.

`render-diff`는 rhwp 자기 roundtrip의 기하 회귀 게이트이므로 한컴 기준 PDF와의 physical page owner
차이를 찾는 이 용도로 사용하지 않았다.

## 입력·실행·완료 상태

```bash
RHWP_BIN=target/review-planet6897-20260802/release-test/rhwp \
python3 tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source 'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  --reference-pdf 'pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf' \
  --label issue3738-hwp --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /private/tmp/rhwp-stage20-fidelity-ledger
```

- 기준 PDF 요청 범위: 0–214 (1-based p1–215), 완료 215/215, 누락 0.
- `--export-all-svg`는 rhwp SVG 220쪽을 한 번에 export했다. 따라서 기준 PDF 215쪽과 native 220쪽의
  전체 page count 불일치는 숨기지 않았다.
- `--text-only`여서 Chrome/PNG comparison sheet를 215개 만들지 않았다. 현재 Mac에 설치된 `pypdf`로
  PDF text만 추출했으며, 사용자가 이미 수행한 WASM build와 `pypdfium2` raster는 재실행하지 않았다.

## 보존된 증적

- [provenance.tsv](../pr/assets/pr_3740_issue3738_stage20/provenance.tsv) — direct pair 경로·등급
- [run-state.tsv](../pr/assets/pr_3740_issue3738_stage20/run-state.tsv) — requested/completed/missing=215/215/0
- [text-report.tsv](../pr/assets/pr_3740_issue3738_stage20/text-report.tsv) — PDF만 존재하는 문자와 SVG만 존재하는 문자
- [layout-candidates.tsv](../pr/assets/pr_3740_issue3738_stage20/layout-candidates.tsv) — body/각주·표/footer·frame 후보

입력 HWP/HWPX/PDF는 각각 `samples/`, `samples/`, `pdf/pr3740/hwp/`에 보관돼 있으며 SHA-256은
HWP `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`, HWPX
`8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c`, PDF
`7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`다. TSV 4개는 모두 LFS filter
비대상이며 일반 Git으로 보관했다. TSV SHA-256은 provenance
`4e4d686e778fa9a12072d26ad9990f3c3cbc41b056fcf3c07c5ad94c3f671fda`, run state
`3f1c96db93cf6a8f82787d80e9b6b331dbd25c3eeaacf6fba8a4208585418b9e`, text report
`924010ef6848c4f0cfbd96834a0d1ff7681ab557c49f60c8cdac572bcd0c7e22`, layout ledger
`0c3787b81e5928ad3caad8b1778645f25279a68180a05f401f2ce320cbd47769`다.

## 후보 결과

| signal | 결과 | 해석 |
| --- | --- | --- |
| PDF text↔SVG text nonzero | 117/215쪽 (0 차이 98쪽) | physical owner 이동·누락·과잉의 빠른 후보. page 155 이후에는 220/215 page-map 분기로 큰 값이 연쇄되어 개별 결함 수로 세지 않는다. |
| text 차이 합계 ≥25 | 100쪽 | high-DPI sweep의 전수 대상이 아니라, page-map cluster의 시작점과 기존 P0 목록을 고르는 입력이다. |
| `overflowCellLines` | p157에서 26 | 표 셀 content가 하단에서 clip되는 직접 구조 신호. |
| body `TextLine` ↔ `FootnoteArea` | p43, p54, p67, p85 등 | 사용자가 지적한 p43·54·67·85를 후보로 재포착했다. p9·26·91 등은 새 후보이며 아직 결함으로 승격하지 않았다. |
| Body `Table` ↔ `Footer` | p106, p158 | p106의 기존 사용자 관측을 재포착했다. p158은 table bottom이 page frame 밖으로 나가 p157 overflow와 함께 다음 원인 분석의 최우선 대상이다. |

기존 사용자 대조에서 고정된 page는 text report도 명확하게 다시 올렸다: p52 `reference_only=83`,
p53 `73/21`, p66 `153/0`, p67 `0/153`, p83 `25/0`, p84 `136/2`, p85 `0/159`, p90 `61/0`,
p94 `0/534`, p106 `0/106`, p107 `67/75`, p108 `78/58` (각 값은 `reference_only/svg_only`).
p25는 `0/0`이므로 이 text-only signal이 그림 25 누락을 놓친다는 사실도 확인된다. 이 때문에
`layout-candidates.tsv`와 후속 pixel/visual sweep을 반드시 결합한다.

## 도구 검증

- `python3 -m py_compile tools/fidelity_compare/fidelity_compare.py` 통과
- 등록 fixture positional (`plan 0 9`)과 direct pair positional (`0 214 --source ...`) parse 계약 통과
- synthetic render-tree에서 body/footnote=1, table/footer=1, image frame=1 후보 판정 통과
- 실제 direct pair 1쪽 probe 및 215쪽 full text-only+layout-ledger run 모두 complete
- `git diff --check` 통과

이 단계에서는 Rust renderer 코드를 바꾸지 않았으므로 cargo/WASM을 새로 실행하지 않았다.

## 후속

다음 Stage는 p157–158의 clipped table과 page-frame 이탈이 220/215 page-map 차이를 만드는 최초/주요
분기인지 source para·RowBreak fragment·PDF p157–158 기준으로 분석한다. 그 결과에 따라 p43·54·67·85·106의
개별 P0 reservation 결함을 같은 원인으로 묶을지, 별도 수정으로 다룰지를 결정한다.
